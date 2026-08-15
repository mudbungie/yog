//! **What one tick decides** (VISION §4.3) — the level-triggered plan, cut off
//! [`super`] at §12's per-file budget (bl-b4b5) along the seam that module's own
//! doc already draws: the thread and its acts live there, and this is the pure
//! function that says which act a tick makes.
//!
//! Everything here is a fold over the published snapshot and the board built
//! from it, which is what lets the whole decision be a table test — no clock of
//! its own, no disk, and no state between ticks.

use super::super::{Facts, facts};
use super::Move;
use crate::app::Snapshot;
use crate::board::{BoardRow, Column};
use crate::git_tree::{AgentState, GitTree};

/// One workspace's move, or `None` when its loop has nothing to do. Pure over
/// the published snapshot and the board built from it — which is what lets the
/// whole decision be a table test.
pub fn plan(snap: &Snapshot, fleet: &Facts, rows: &[BoardRow], now: i64) -> Option<Move> {
    reap(snap, fleet, rows, now).or_else(|| spawn(snap, fleet, rows))
}

/// The reap: the first claimed ball of this workspace whose conversations have
/// all been quiet past the lease. No lease, no reap — releasing a claim is not
/// something yog does on a default (see [`arming`]).
fn reap(snap: &Snapshot, fleet: &Facts, rows: &[BoardRow], now: i64) -> Option<Move> {
    let lease = i64::try_from(fleet.lease?.as_secs()).ok()?;
    let tree = snap.trees.get(&fleet.workspace)?;
    rows.iter().filter(|r| held_here(r, fleet)).find_map(|row| {
        // A row that names nobody cannot be released from anyone.
        let claimant = row.claimant.clone()?;
        let idle = quiet_for(tree, row, now)?;
        (idle >= lease).then(move || Move::Reap {
            row: row.clone(),
            claimant,
            // The comparison itself, never a diagnosis (§4.3): how far past
            // the operator's own number this ball has gone, and nothing
            // about why.
            since: format!("lease expired {} ago", facts::secs_label(idle - lease)),
        })
    })
}

/// The spawn: the board's top ready ball in this loop's project, when the
/// workspace has room under its cap and the ceiling would not refuse the birth
/// anyway. **Ready only** — a gated ball can be started but cannot be
/// delivered, and a loop that fills its cap with undeliverable work has stopped
/// being a fleet.
fn spawn(snap: &Snapshot, fleet: &Facts, rows: &[BoardRow]) -> Option<Move> {
    if !fleet.has_room() {
        return None;
    }
    // The armed entry names a clone directory and the row names the project
    // (bl-b4b5), so the entry is put into the row's vocabulary rather than the
    // row into the entry's — the one direction the naming set can answer.
    let project = snap.project_name(&fleet.project);
    rows.iter()
        .find(|r| r.column == Column::Ready && r.project == project)
        .map(|row| Move::Spawn { row: row.clone() })
}

/// Whether this row is a ball the armed workspace holds right now.
fn held_here(row: &BoardRow, fleet: &Facts) -> bool {
    row.column == Column::Claimed
        && row.workspace.as_deref() == Some(crate::naming::leaf(&fleet.workspace).as_str())
}

/// How long every conversation on this ball has been quiet, or `None` when one
/// of them is still running (or the ball has no conversation at all).
///
/// **Nothing running is ever reaped**, whatever its age: the ceiling's own
/// ruling — killing mid-ball destroys uncommitted work — applies to a claim as
/// much as to a spend, and a live drone's claim is not idle by any reading.
fn quiet_for(tree: &GitTree, row: &BoardRow, now: i64) -> Option<i64> {
    let mut newest: Option<i64> = None;
    for agent in &tree.agents {
        let root = crate::nav::convs::root_of(&tree.agents, &agent.agent_id)?;
        if !row.drones.iter().any(|d| d.root_id == root) {
            continue;
        }
        if matches!(agent.state, AgentState::Live | AgentState::InFlight) {
            return None;
        }
        newest = Some(newest.unwrap_or(i64::MIN).max(agent.last_action_unix));
    }
    newest.map(|last| now.saturating_sub(last))
}
