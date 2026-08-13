//! The loop's ops rows (VISION §5 V4 item 2, DESIGN §4.2): **every spawn and
//! every reap, and nothing else.**
//!
//! One `ops.jsonl` line per action the loop completes, in the shape §4.2
//! already defines, with no field added to the schema — the same discipline
//! `["yog-step",…]`, `["yog-drift",…]` and `["yog-monitor",…]` follow:
//!
//! ```text
//! argv[0]  yog-fleet              the pseudo-binary naming a loop row
//! argv[1]  spawn|reap
//! argv[2]  the ball id the action was about
//! argv[3]  what it produced: the conversation a spawn minted, or the
//!          claimant name a reap released the ball from
//! cwd      the armed workspace
//! stdout   a reap's COMPARISON; empty for a spawn
//! exit     0 — the loop writes a row for what it DID
//! ```
//!
//! **A reap reason is the comparison itself** (V4 item 2, verbatim: *"Reap
//! reasons are the comparisons themselves ('lease expired 14m ago'), never
//! diagnoses"*). [`reaped`] takes an already-formed comparison and stores it
//! verbatim; there is deliberately no way to hand this module a judgement. The
//! loop spawns and reaps; it never diagnoses (§4.3).
//!
//! **A failed action writes nothing here.** Every executor the loop composes —
//! the start flow's `bl` steps, the §3.5 ceiling gate, `bl unclaim` — already
//! leaves its own §4.2 failure row, and the loop is level-triggered, so the
//! next tick simply re-fires against whatever it finds. A second row saying
//! "and the loop wanted that" would double every failure on the trail and, at
//! one tick per full sweep, would do it forever.

use crate::opslog::{OpEntry, OpRow, Origin};
use std::path::Path;

/// `argv[0]` of a loop row — the pseudo-binary, beside `yog-step`, `yog-drift`
/// and `yog-monitor`. No process is involved: the loop's own decision is what
/// the row records, the verbs it drove having logged themselves.
pub const YOG_FLEET: &str = "yog-fleet";

/// `argv[1]` — what the loop did. Two words, because the loop does two things.
pub const SPAWN: &str = "spawn";
pub const REAP: &str = "reap";

/// One row, read back. The render side's value and the tests' assertion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Act {
    /// The armed workspace — the row's `cwd`, as [`crate::nav::ws_key`] spells
    /// it.
    pub workspace: String,
    /// [`SPAWN`] or [`REAP`].
    pub verb: String,
    pub ball: String,
    /// The conversation a spawn minted, or the claimant a reap released.
    pub subject: String,
    /// A reap's comparison, verbatim; empty for a spawn.
    pub reason: String,
    /// The row's own wall stamp (§4.2 unix seconds), or `0` when unreadable.
    pub ts: i64,
}

/// The row a completed **spawn** appends: the ball it took and the conversation
/// it minted on it.
pub fn spawned(ts: String, workspace: &Path, ball: &str, conversation: &str) -> OpEntry {
    entry(ts, workspace, SPAWN, ball, conversation, String::new())
}

/// The row a completed **reap** appends: the ball it released, the claimant it
/// released it from, and the comparison that decided it — never a diagnosis.
pub fn reaped(ts: String, workspace: &Path, ball: &str, claimant: &str, since: &str) -> OpEntry {
    entry(ts, workspace, REAP, ball, claimant, since.to_owned())
}

fn entry(
    ts: String,
    workspace: &Path,
    verb: &str,
    ball: &str,
    subject: &str,
    stdout: String,
) -> OpEntry {
    OpEntry {
        ts,
        argv: vec![
            YOG_FLEET.to_owned(),
            verb.to_owned(),
            ball.to_owned(),
            subject.to_owned(),
        ],
        cwd: crate::nav::ws_key(workspace),
        exit: 0,
        stdout,
        stderr: String::new(),
        // The board is the surface the loop acts on, so a loop row belongs to
        // the same §7.3 attribution a ▶ Start from that section carries.
        origin: Origin::Balls,
    }
}

/// Read one durable line as a loop act, or `None` when it is not one. Forgiving
/// like every other `ops.jsonl` read: a row from a future yog with more fields,
/// or a hand-mangled one, simply is not an act.
fn read(argv: &[&str], cwd: &str, stdout: &str, ts: &str) -> Option<Act> {
    let &[head, verb, ball, subject] = argv else {
        return None;
    };
    if head != YOG_FLEET || (verb != SPAWN && verb != REAP) {
        return None;
    }
    Some(Act {
        workspace: cwd.to_owned(),
        verb: verb.to_owned(),
        ball: ball.to_owned(),
        subject: subject.to_owned(),
        reason: stdout.to_owned(),
        ts: ts.parse().unwrap_or(0),
    })
}

/// The acts in a published snapshot's ops tail, oldest first. [`OpRow`] joins
/// `argv` for display and every field a loop row puts there is space-free by
/// construction (a word, a ball id, a conversation or claimant name), so the
/// split is lossless.
pub fn of_rows(rows: &[OpRow]) -> Vec<Act> {
    rows.iter()
        .filter_map(|r| {
            read(
                &r.argv.split(' ').collect::<Vec<_>>(),
                &r.cwd,
                &r.stdout,
                &r.ts,
            )
        })
        .collect()
}

/// When this workspace's loop last **did** something — the newest act's stamp,
/// or `None` when it has never acted. The board's "last tick" fact: a
/// level-triggered loop's quiet tick leaves no trace by design, so the last
/// tick yog can name is the last one that changed the world.
pub fn last_act(acts: &[Act], workspace: &str) -> Option<i64> {
    acts.iter()
        .rev()
        .find(|a| a.workspace == workspace)
        .map(|a| a.ts)
}

#[cfg(test)]
mod tests;
