//! **What one tick decides** (VISION §4.3) — the level-triggered plan, cut off
//! [`super`] at §12's per-file budget (bl-b4b5) along the seam that module's own
//! doc already draws: the thread and its acts live there, and this is the pure
//! function that says which act a tick makes.
//!
//! Everything here is a fold over the published snapshot and the board built
//! from it, which is what lets the whole decision be a table test — no clock of
//! its own, no disk, and no state between ticks.

use super::super::{Facts, facts, row as fleet_row};
use super::Move;
use crate::app::Snapshot;
use crate::board::{BoardRow, Column};
use crate::git_tree::{AgentState, GitTree};

/// One workspace's move, or `None` when its loop has nothing to do. Pure over
/// the published snapshot and the board built from it — which is what lets the
/// whole decision be a table test.
pub fn plan(snap: &Snapshot, fleet: &Facts, rows: &[BoardRow], now: i64) -> Option<Move> {
    stillborn(snap, fleet, rows)
        .or_else(|| reap(snap, fleet, rows, now))
        .or_else(|| spawn(snap, fleet, rows))
}

/// **The stillbirth** (bl-ab13): a claim this loop holds that has no
/// conversation at all, and whose own spawn row is answered on the trail by a
/// driver that died in the handoff. The claim comes back.
///
/// This is a reap and not a new move — the act is the same `bl unclaim`, and
/// the reason is the same kind of sentence: what the loop recorded set against
/// what the world has, never a diagnosis of the driver's death.
///
/// **It is not lease-gated, and the default lease must not reach it.** The
/// lease is the operator's number for how long a *quiet worker* may hold a
/// ball, and [`reap`] rightly refuses to guess one. A stillbirth is not a quiet
/// worker: there is nobody to be quiet, so no number can ever expire, and
/// before this the slot and the ball were consumed forever while the trail said
/// the spawn had succeeded. Undoing an act the loop itself made, on evidence
/// the loop itself wrote, is not a judgement about a drone.
///
/// **Two conditions keep it off a healthy birth.** The driver must have said
/// something dying ([`OpRow::detached_died`](crate::opslog::OpRow)), so a slow
/// but healthy launch is never touched; and the loop must have re-derived the
/// world *since* it spawned (`derived_at_unix` past the spawn row), so a
/// conversation missing only because this snapshot predates it is yog's own
/// latency rather than a fact about the world.
///
/// A driver that dies **silently** leaves no evidence and is out of reach here;
/// that claim is still the lease's to reap once one is set.
fn stillborn(snap: &Snapshot, fleet: &Facts, rows: &[BoardRow]) -> Option<Move> {
    let key = crate::nav::ws_key(&fleet.workspace);
    let acts = fleet_row::of_rows(&snap.ops);
    rows.iter()
        .filter(|r| held_here(r, fleet) && r.drones.is_empty())
        .find_map(|row| {
            let claimant = row.claimant.clone()?;
            // The loop's OWN newest birth on this ball — a claim yog did not
            // make is not yog's to unmake.
            let act = acts
                .iter()
                .rev()
                .find(|a| a.verb == fleet_row::SPAWN && a.workspace == key && a.ball == row.id)?;
            (act.ts < snap.derived_at_unix && died(snap, &key, act.ts)).then(move || Move::Reap {
                row: row.clone(),
                claimant,
                since: format!("spawn {} left no conversation", act.subject),
            })
        })
}

/// Whether the driver the loop handed off at `ts` in `ws` died there. The join
/// is exact and needs no field of its own: the detached `lernie prompt` row and
/// the loop's own spawn row are written inside one tick, from one stamp, with
/// the workspace as both their `cwd` (§4.2).
fn died(snap: &Snapshot, ws: &str, ts: i64) -> bool {
    snap.ops
        .iter()
        .any(|r| r.cwd == ws && r.ts.parse::<i64>().ok() == Some(ts) && r.detached_died())
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

/// The spawn: the board's top ready ball in this loop's project **that the loop
/// has not already given back**, when the workspace has room under its cap and
/// the ceiling would not refuse the birth anyway. **Ready only** — a gated ball
/// can be started but cannot be delivered, and a loop that fills its cap with
/// undeliverable work has stopped being a fleet.
fn spawn(snap: &Snapshot, fleet: &Facts, rows: &[BoardRow]) -> Option<Move> {
    if !fleet.has_room() {
        return None;
    }
    // The armed entry names a clone directory and the row names the project
    // (bl-b4b5), so the entry is put into the row's vocabulary rather than the
    // row into the entry's — the one direction the naming set can answer.
    let project = snap.project_name(&fleet.project);
    let acts = fleet_row::of_rows(&snap.ops);
    let key = crate::nav::ws_key(&fleet.workspace);
    rows.iter()
        .find(|r| r.column == Column::Ready && r.project == project && !given_back(&acts, &key, r))
        .map(|row| Move::Spawn { row: row.clone() })
}

/// **Whether this loop has already given this ball back** (bl-3988): its newest
/// act on the ball, in this workspace, was a reap.
///
/// A reap returns a ball to *ready*, and the board's first ready ball is the
/// pick — so without this the highest-priority ball the loop cannot finish is
/// re-taken by the very next tick, forever, and no lower ball ever runs. That
/// is a board state making the loop storm, which §4.3's own law says cannot
/// happen; it does not, once the loop declines to undo its own decision.
///
/// **Nothing is stored and no number is invented.** The bound is the loop's own
/// trail: it took this ball, it gave it back, and taking it again would be a
/// bet that the same fire has a different outcome — which is the diagnosing
/// §4.3 forbids. The ball stays on the board, ready, for an operator who *can*
/// judge it; the loop simply moves to the next one.
///
/// Its reach is the ops tail's reach and no further, which is the honest bound
/// rather than a promised one: a reap that has scrolled off the trail is a fact
/// yog no longer has, and the loop may take the ball again.
fn given_back(acts: &[fleet_row::Act], workspace: &str, row: &BoardRow) -> bool {
    acts.iter()
        .rev()
        .find(|a| a.workspace == workspace && a.ball == row.id)
        .is_some_and(|a| a.verb == fleet_row::REAP)
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
