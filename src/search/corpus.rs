//! What search reads, and in what order (§8.5).
//!
//! Two halves, and the split is the whole cancellation story. The **snapshot
//! half** is free — balls (live and closed) and workspaces are already in the
//! published derivation, so searching them costs a scan of memory yog was
//! holding anyway. The **conversation half** is disk: a goal and a transcript
//! per agent, re-read at ask time because the bytes are the authority (I1), and
//! therefore the half the asker's liveness is consulted between.
//!
//! Order is derived, never incidental: workspaces in enumeration order,
//! projects sorted, balls in listing order, conversations in the workspace's
//! own agent order. Two runs over one world enumerate identically.

use super::{Address, Field};
use crate::app::Snapshot;
use crate::naming::leaf as leaf_name;
use crate::projects::balls::Ball;
use crate::projects::join::JoinState;
use std::path::{Path, PathBuf};

/// The goal file inside an agent's worktree (§3.3) — the conversation's
/// [`Summary`](Field::Summary).
const GOAL: &str = "goal.md";

/// One subject's searchable fields, tagged with the tier each sits in.
type Fields = Vec<(Field, String)>;

/// Everything the published snapshot already holds: every ball of every listed
/// project (live *and* closed — the same on-demand corpus the §3.5 Delivered
/// rows are drawn from, never a second fetch), and every enumerated workspace
/// under its §3.1 name.
pub(super) fn from_snapshot(snap: &Snapshot) -> Vec<(Address, Fields)> {
    let mut projects: Vec<&PathBuf> = snap
        .balls_by_project
        .keys()
        .chain(snap.closed_by_project.keys())
        .collect();
    projects.sort();
    projects.dedup();
    let mut rows: Vec<(Address, Fields)> = Vec::new();
    for project in projects {
        let live = snap.balls_by_project.get(project).into_iter().flatten();
        let closed = snap.closed_by_project.get(project).into_iter().flatten();
        rows.extend(live.chain(closed).map(|ball| ball_row(project, ball)));
    }
    rows.extend(snap.workspaces.iter().map(|w| {
        (
            Address::Workspace {
                path: w.path.clone(),
            },
            vec![(Field::Name, leaf_name(&w.path))],
        )
    }));
    rows
}

/// One ball as a subject: its id is what it **is**, its title what it is
/// **for**, its body what it **says**.
fn ball_row(project: &Path, ball: &Ball) -> (Address, Fields) {
    (
        Address::Ball {
            project: project.to_path_buf(),
            id: ball.id.clone(),
        },
        vec![
            (Field::Name, ball.id.clone()),
            (Field::Summary, ball.title.clone()),
            (Field::Text, ball.body.clone()),
        ],
    )
}

/// Every derived conversation with the identity fields the snapshot already
/// carries — its agent id and, when it wears one, its §3.3 name. The disk half
/// is [`read_conversation`], asked per subject so a superseded search stops
/// before paying for it.
pub(super) fn conversations(snap: &Snapshot) -> Vec<(PathBuf, String, Fields)> {
    let mut rows: Vec<(PathBuf, String, Fields)> = Vec::new();
    for ws in &snap.workspaces {
        let Some(tree) = snap.trees.get(&ws.path) else {
            continue;
        };
        rows.extend(tree.agents.iter().map(|agent| {
            let mut fields = vec![(Field::Name, agent.agent_id.clone())];
            fields.extend(agent.name_fact().map(|name| (Field::Name, name)));
            (ws.path.clone(), agent.agent_id.clone(), fields)
        }));
    }
    rows
}

/// One conversation's bytes, re-read now: its goal, then every committed
/// `messages/` entry verbatim ([`Entry::raw`](crate::transcript::Entry::raw)).
/// A goal that exists but cannot be read is **named** rather than skipped —
/// that gap is exactly what an operator must not have to guess at. An absent
/// goal is not a gap: most agents never had one.
///
/// **A compacted span is named the same way** (bl-fde5). lernie's compactor
/// deletes entries out of `messages/` (§5.1 #12), so a search over such a
/// conversation is a search over a **rewritten** record: the deleted text is
/// unrecoverable and no hit can come from it. The spliced marker's `raw` is
/// the compaction summary, so that replacement prose *is* searched — but the
/// answer must not read as the whole conversation having been, which is
/// exactly the `unreadable` channel's job: a source the search could not read,
/// named with why.
pub(super) fn read_conversation(
    workspace: &Path,
    agent: &str,
    unreadable: &mut Vec<String>,
) -> Fields {
    let goal = crate::files_view::agent_worktree(workspace, agent).join(GOAL);
    let mut fields: Fields = Vec::new();
    match std::fs::read(&goal) {
        Ok(bytes) => fields.push((Field::Summary, String::from_utf8_lossy(&bytes).into_owned())),
        Err(e) if goal.exists() => unreadable.push(format!("{}: {e}", goal.display())),
        Err(_) => {}
    }
    let transcript = crate::transcript::build(workspace, agent);
    for entry in &transcript.entries {
        if let crate::transcript::EntryKind::Compacted { first, last, .. } = entry.kind {
            unreadable.push(format!(
                "{}/{agent}: entries {first:03}\u{2013}{last:03} compacted away — searched the \
                 surviving record and the compaction summary",
                workspace.display()
            ));
        }
    }
    fields.extend(transcript.entries.into_iter().map(|entry| {
        (
            Field::Text,
            String::from_utf8_lossy(&entry.raw).into_owned(),
        )
    }));
    fields
}

/// The sources the derivation itself already reports as unreachable: a
/// workspace whose tree failed to derive (absent from
/// [`trees`](Snapshot::trees)) and a project whose balls are unlistable (the
/// §3.5 orphan row). Search names them rather than quietly searching a smaller
/// world.
pub(super) fn unreadable(snap: &Snapshot) -> Vec<String> {
    let workspaces = snap
        .workspaces
        .iter()
        .filter(|w| !snap.trees.contains_key(&w.path))
        .map(|w| format!("{}: no derived tree", w.path.display()));
    let projects = snap
        .join_rows
        .iter()
        .filter(|r| r.state == JoinState::OrphanedProject)
        .map(|r| format!("{}: balls unlistable", r.project));
    workspaces.chain(projects).collect()
}
