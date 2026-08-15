//! The §3.5 join-state table (DESIGN §3.2, §3.5, §5.1 #7): the ball × workspace
//! product enumerated once, every combination a row state, never an ad-hoc
//! branch. The binding is the claimant equality (§3.2): a ball is **bound** to a
//! workspace iff its claimant equals that workspace's name — no operator
//! identity, no stored fact.
//!
//! [`classify`] is the total function at its heart — (status, bound?) ⇒ exactly
//! one ball-driven [`JoinState`]. [`join`] enumerates the whole product over the
//! pre-fetched live balls ([`super::balls`]), the on-demand closed listing, and
//! the enumerated workspaces ([`crate::binding`]): a row per live ball, a
//! delivered row per closed ball whose claimant names a local workspace, an
//! unassigned row per named workspace no ball claims, and an orphaned row per
//! cloned-but-unlistable project.

use super::balls::{Ball, Status, ladder};
use crate::binding::{Workspace, WorkspaceKind};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};

/// The left axis of the §3.5 table: the ball's derived status (§5.1 #3),
/// including `Closed` (absence from the live set — the on-demand closed listing).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JoinStatus {
    Ready,
    Blocked,
    Claimed,
    Closed,
}

/// The live [`Status`] ladder lifted into the join's left axis; a closed ball
/// enters as [`JoinStatus::Closed`] directly, never through this.
fn live_status(status: Status) -> JoinStatus {
    match status {
        Status::Ready => JoinStatus::Ready,
        Status::Blocked => JoinStatus::Blocked,
        Status::Claimed => JoinStatus::Claimed,
    }
}

/// The rendered row state for one (ball × workspace) cell (§3.5). The seven §3.5
/// rows: five ball-driven ([`classify`]), plus the workspace-driven
/// [`UnassignedWorkspace`](JoinState::UnassignedWorkspace) and the project-driven
/// [`OrphanedProject`](JoinState::OrphanedProject).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JoinState {
    /// Ready ball, unclaimed: ▶ Start (ball rung) or Assign to a workspace.
    ReadyStartable,
    /// Blocked ball, unclaimed: blocker edges shown from bedrock JSON.
    Blocked,
    /// Claimed by a local workspace name — the normal working row, grouped under
    /// its workspace (§3.2).
    Bound,
    /// Claimed by a name that is **not** a local workspace — a human, another
    /// machine, or a deleted workspace; the badge shows the claimant verbatim.
    ClaimedElsewhere,
    /// Closed ball whose claimant names a local workspace — grouped under it
    /// (on demand, §3.4/§5.1 #4).
    Delivered,
    /// A named workspace no ball claims — the bare/path-rung general case: full
    /// rendering, no ball column (§3.5).
    UnassignedWorkspace,
    /// The project's clone is gone / its balls unlistable, marked missing;
    /// workspaces are unaffected (they encode no project path, §3.5).
    OrphanedProject,
}

/// The §3.5 table as a total function over the ball-driven cells: (status,
/// bound?) ⇒ the row state. Ready/Blocked ignore boundness (an unclaimed ball
/// names no workspace); a claim splits on the binding; a closed ball is
/// delivered (its `workspace` grouping is set by [`join`], not decided here).
pub fn classify(status: JoinStatus, bound: bool) -> JoinState {
    match (status, bound) {
        (JoinStatus::Ready, _) => JoinState::ReadyStartable,
        (JoinStatus::Blocked, _) => JoinState::Blocked,
        (JoinStatus::Claimed, true) => JoinState::Bound,
        (JoinStatus::Claimed, false) => JoinState::ClaimedElsewhere,
        (JoinStatus::Closed, _) => JoinState::Delivered,
    }
}

/// The short roster badge for a join state (§3.5); `None` where the row needs
/// none (a startable/bound/unassigned workspace row). `claimant` fills the
/// "claimed by <who>" text.
pub fn badge(state: JoinState, claimant: Option<&str>) -> Option<String> {
    Some(match state {
        JoinState::ReadyStartable | JoinState::Bound | JoinState::UnassignedWorkspace => {
            return None;
        }
        JoinState::Blocked => "blocked".to_owned(),
        JoinState::ClaimedElsewhere => format!("claimed by {}", claimant.unwrap_or("?")),
        JoinState::Delivered => "delivered".to_owned(),
        JoinState::OrphanedProject => "project missing".to_owned(),
    })
}

/// The workspace name a bound ball's close / release / move-from stamps `--as`
/// (§3.2, §8.2 rider): the ball's claimant — which, for a [`Bound`](JoinState::Bound)
/// row, **is** the local workspace's name (the join binds on that equality). The
/// claimant delivers its own ball; empty for a row that names no claimant (never a
/// Bound one). Covered derivation, so the shell paints without deciding names.
pub fn owner_name(row: &JoinRow) -> String {
    row.claimant.clone().unwrap_or_default()
}

/// One classified row of the ball × workspace join (§3.5). `title`/`claimant`
/// carry live-ball detail; both are absent for a delivered/unassigned/orphaned
/// row. `workspace` is the bound workspace's name when one matches, else `None`.
///
/// **Both addresses are NAMES** (REMOTE §8.1, bl-b4b5). They were a project
/// `PathBuf` and a workspace `PathBuf` — the last two path-typed fields on a
/// reply payload after `Prepared::binding`, and the ones §8.1's own list did
/// not reach. `Reply::Balls` therefore answered a thin seat two absolute paths
/// under the engine's home: unusable there and a disclosure besides. They are
/// the §5.1 #1 project name and the §3.1 workspace leaf now — the *same* two
/// words `Action::Close`/`Assign`/`Move` already take — so a seat holding a
/// name can select its own workspace's rows out of the answer without joining
/// it back against the engine's table (the shape bl-7407 refused). The engine
/// resolves either back through [`Snapshot::project_path`](crate::app::Snapshot)
/// / [`ws_path`](crate::app::Snapshot) at the one seam that owns the round trip.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JoinRow {
    pub project: String,
    pub ball_id: String,
    pub state: JoinState,
    pub workspace: Option<String>,
    pub claimant: Option<String>,
    pub title: Option<String>,
}

/// The local named workspaces as `name -> path` (§3.1); only named workspaces
/// bind — foreign and replay workspaces never carry a claimant identity. A
/// `BTreeMap` so the [`join`] emission of the trailing UnassignedWorkspace rows
/// (its iteration) is name-ordered — deterministic across instances (I9).
fn named_map(workspaces: &[Workspace]) -> BTreeMap<&str, &Path> {
    workspaces
        .iter()
        .filter_map(|w| match &w.kind {
            WorkspaceKind::Named { name } => Some((name.as_str(), w.path.as_path())),
            _ => None,
        })
        .collect()
}

/// Enumerate the §3.5 join (§5.1 #7), pure over the pre-fetched caches. Per
/// cloned, listable project: a row per live ball (bound to its named workspace
/// when the claimant matches) and a Delivered row per closed ball whose claimant
/// names a local workspace. Then an UnassignedWorkspace row per named workspace
/// no ball engages, and an OrphanedProject row per cloned project absent from
/// `live_by_project` (unlistable). `closed_by_project` is on-demand (§5.1 #4):
/// empty on the fetch cadence, populated per project after a `bl close`.
///
/// `projects` is the **naming set** (§5.1 #1, the snapshot's whole enumeration,
/// internal clones included) that each row's project name is derived over —
/// which is a superset of `cloned`, because a name is unique against everything
/// that exists and not merely against what listed cleanly.
pub fn join(
    projects: &[PathBuf],
    cloned: &[PathBuf],
    live_by_project: &HashMap<PathBuf, Vec<Ball>>,
    closed_by_project: &HashMap<PathBuf, Vec<Ball>>,
    workspaces: &[Workspace],
) -> Vec<JoinRow> {
    let names = named_map(workspaces);
    let mut rows = Vec::new();
    // Local names engaged by any live or closed ball — the rest are unassigned.
    let mut engaged: HashSet<String> = HashSet::new();

    for project in cloned {
        let name = crate::naming::name_of(projects, project);
        let Some(live) = live_by_project.get(project) else {
            rows.push(orphaned_row(name));
            continue;
        };
        let live_ids: HashSet<&str> = live.iter().map(|b| b.id.as_str()).collect();
        for ball in live {
            let bound = bind(&names, ball, &mut engaged);
            let js = live_status(ladder(ball, &live_ids));
            rows.push(JoinRow {
                project: name.clone(),
                ball_id: ball.id.clone(),
                state: classify(js, bound.is_some()),
                workspace: bound,
                claimant: ball.claimant.clone(),
                title: Some(ball.title.clone()),
            });
        }
        for ball in closed_by_project.get(project).into_iter().flatten() {
            // Live is authoritative: a ball reopened since the on-demand closed
            // fetch already has its live row, so skip the stale closed entry.
            if live_ids.contains(ball.id.as_str()) {
                continue;
            }
            // A closed ball is a roster row only when it groups under a local
            // workspace; otherwise it lives in the raw on-demand closed listing.
            if let Some(bound) = bind(&names, ball, &mut engaged) {
                rows.push(JoinRow {
                    project: name.clone(),
                    ball_id: ball.id.clone(),
                    state: classify(JoinStatus::Closed, true),
                    workspace: Some(bound),
                    claimant: ball.claimant.clone(),
                    title: None,
                });
            }
        }
    }

    for name in names.keys() {
        if !engaged.contains(*name) {
            rows.push(JoinRow {
                project: String::new(),
                ball_id: String::new(),
                state: JoinState::UnassignedWorkspace,
                workspace: Some((*name).to_owned()),
                claimant: None,
                title: None,
            });
        }
    }
    rows
}

/// Resolve a ball's binding: the local workspace **name** equal to the ball's
/// claimant (§3.2), marking it engaged. `None` when the ball is unclaimed or
/// claimed by a non-local name. The claimant *is* the workspace's §3.1 name —
/// the join binds on that equality — so the binding and the address it answers
/// are one string rather than a path derived from it.
fn bind(
    names: &BTreeMap<&str, &Path>,
    ball: &Ball,
    engaged: &mut HashSet<String>,
) -> Option<String> {
    let claimant = ball.claimant.as_deref()?;
    names.get(claimant)?;
    engaged.insert(claimant.to_owned());
    Some(claimant.to_owned())
}

/// A project-driven orphaned row: its clone is present but unlistable, so no
/// ball detail is available (§3.5).
fn orphaned_row(project: String) -> JoinRow {
    JoinRow {
        project,
        ball_id: String::new(),
        state: JoinState::OrphanedProject,
        workspace: None,
        claimant: None,
        title: None,
    }
}

#[cfg(test)]
mod tests;
