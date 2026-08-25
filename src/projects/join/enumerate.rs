//! **Enumerating the product** — the ball × workspace join itself, pure over
//! the pre-fetched caches. Split from the table at §12's budget on the seam the
//! module's own doc draws: [`super`] is the total function at its heart —
//! (status, bound?) ⇒ exactly one state — and this is the walk that asks it
//! once per combination, so no row anywhere is an ad-hoc branch.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};

use super::super::balls::{Ball, ladder};
use super::{JoinRow, JoinState, JoinStatus, classify, live_status};
use crate::binding::{Workspace, WorkspaceKind};

/// The local named workspaces as `name -> path` (§3.1); only named workspaces
/// bind — foreign and replay workspaces never carry a claimant identity. A
/// `BTreeMap` so the [`join`] emission of the trailing UnassignedWorkspace rows
/// (its iteration) is name-ordered — deterministic across instances (I9).
pub(super) fn named_map(workspaces: &[Workspace]) -> BTreeMap<&str, &Path> {
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
