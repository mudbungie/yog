//! The check's ops row (VISION §4.9, DESIGN §4.2) — audit trail, level-trigger
//! memory and tuning dataset in one line.
//!
//! A check writes exactly one `ops.jsonl` row, in the shape §4.2 already
//! defines, with no field added to the schema — the same discipline
//! `["yog-step",…]` and `["yog-drift",…]` follow:
//!
//! ```text
//! argv[0]  yog-monitor            the pseudo-binary naming a monitor row
//! argv[1]  aligned|drifting|diverged
//! argv[2]  the agent id
//! argv[3]  the branch tip sha the verdict read
//! argv[4]  the model that answered
//! argv[5]  input tokens, or `-` when the provider reported none
//! argv[6]  output tokens, or `-`
//! cwd      the workspace the agent lives in
//! stdout   the one-sentence reason
//! exit     0 — a check that completed; a check that did NOT is a `yog-step`
//!          failure line and names no sha, which is what makes the next tick
//!          re-fire (the anti-reinvention law's retry)
//! ```
//!
//! **Nothing derived is stored.** The last-checked sha is not a field anywhere:
//! it is [`latest`] over the rows. Neither is a standing verdict — same query,
//! same row. That is the whole reason the monitor needs no durable of its own.
//!
//! Reading is forgiving, like every other `ops.jsonl` read: a row from a future
//! yog with more tokens, or a hand-mangled one, simply is not a check.

use crate::opslog::{OpEntry, OpRow, Origin};
use std::path::Path;

/// `argv[0]` of a monitor row — the pseudo-binary, beside `yog-step` and
/// `yog-drift`. No process is involved; the call was made in yog's own address
/// space through the embedded brazen adapter.
pub const YOG_MONITOR: &str = "yog-monitor";

/// The logical step name a *failed* check logs under
/// ([`OpEntry::step_failure`]). It names no sha by construction, so the level
/// trigger still sees the tip as unchecked and the next tick re-fires.
pub const STEP: &str = "monitor";

/// How many `char`s of the model's sentence the row keeps. The §4.2 line cap
/// (4096 bytes) would truncate a runaway reason anyway; clipping here keeps the
/// truncation the monitor's own, stated decision rather than the log's accident.
const REASON_MAX: usize = 400;

/// The token an absent token-counter writes. A provider that reported no count
/// leaves it absent — `0` would be a lie (brazen's own rule for its counters).
const ABSENT: &str = "-";

/// One completed check, as a row carries it. This is also the **standing
/// verdict**: V6's "aligned / drifting / diverged, with the reason and the
/// checked sha" is this value, read off the ops tail rather than stored.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Check {
    /// The workspace the checked agent lives in — the row's `cwd`.
    pub workspace: String,
    pub agent: String,
    pub verdict: super::Verdict,
    /// The branch tip this verdict read: what makes it replayable, and what the
    /// level trigger compares the live tip against.
    pub sha: String,
    pub reason: String,
    pub model: String,
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
}

/// The ops entry one completed check appends.
pub fn entry(ts: String, check: &Check) -> OpEntry {
    OpEntry {
        ts,
        argv: vec![
            YOG_MONITOR.to_owned(),
            check.verdict.token().to_owned(),
            check.agent.clone(),
            check.sha.clone(),
            check.model.clone(),
            count(check.input_tokens),
            count(check.output_tokens),
        ],
        cwd: check.workspace.clone(),
        exit: 0,
        stdout: clip(&check.reason),
        stderr: String::new(),
        // A check is yog's own observation about the world, made by no operator
        // gesture — the same attribution a drift observation carries, and the
        // one that raises no §7.3 banner on a surface that did not ask.
        origin: Origin::World,
    }
}

/// The row a *failed* check appends: a `["yog-step","monitor"]` synthetic
/// failure carrying why. It deliberately names no sha — see [`STEP`].
pub fn failure(ts: String, workspace: &Path, agent: &str, why: &str) -> OpEntry {
    OpEntry::step_failure(
        ts,
        STEP,
        crate::nav::ws_key(workspace),
        format!("{agent}: {why}"),
        Origin::World,
    )
}

/// Read one durable line's fields as a check, or `None` when it is not one.
fn read(argv: &[&str], cwd: &str, stdout: &str) -> Option<Check> {
    let &[head, verdict, agent, sha, model, input, output] = argv else {
        return None;
    };
    if head != YOG_MONITOR {
        return None;
    }
    Some(Check {
        workspace: cwd.to_owned(),
        agent: agent.to_owned(),
        verdict: super::Verdict::parse(verdict)?,
        sha: sha.to_owned(),
        reason: stdout.to_owned(),
        model: model.to_owned(),
        input_tokens: tokens(input),
        output_tokens: tokens(output),
    })
}

/// The checks in a durable tail, oldest first — the sentry's reading, over the
/// entries [`crate::opslog::tail`] hands back.
pub fn of_entries(entries: &[OpEntry]) -> Vec<Check> {
    entries
        .iter()
        .filter_map(|e| {
            let argv: Vec<&str> = e.argv.iter().map(String::as_str).collect();
            read(&argv, &e.cwd, &e.stdout)
        })
        .collect()
}

/// The checks in a published snapshot's ops tail — the render side's reading.
/// [`OpRow`] pre-joins `argv` for display, and every field a monitor row puts
/// there is space-free by construction (a verdict token, an agent id, a sha, a
/// model id, two counts), so the split is lossless.
pub fn of_rows(rows: &[OpRow]) -> Vec<Check> {
    rows.iter()
        .filter_map(|r| read(&r.argv.split(' ').collect::<Vec<_>>(), &r.cwd, &r.stdout))
        .collect()
}

/// The newest check for one agent — the standing verdict, and the last-checked
/// sha the level trigger compares against. `workspace` is the §4.1 workspace
/// key ([`crate::nav::ws_key`]), the same spelling the row's `cwd` carries.
/// `checks` is in file order, so the last match wins.
pub fn latest(checks: &[Check], workspace: &str, agent: &str) -> Option<Check> {
    checks
        .iter()
        .rev()
        .find(|c| c.workspace == workspace && c.agent == agent)
        .cloned()
}

/// The standing verdict of a whole conversation (rung V6): the **worst** among
/// its members' latest checks, or `None` when none of them has been checked.
/// Worst, not newest — a diverged child is the conversation's fact however
/// calmly its root is behaving, and it is the same subtree aggregation the §11
/// row already does for state and attention.
pub fn worst(checks: &[Check], workspace: &str, agents: &[String]) -> Option<Check> {
    agents
        .iter()
        .filter_map(|a| latest(checks, workspace, a))
        .max_by_key(|c| c.verdict)
}

fn count(n: Option<u64>) -> String {
    n.map_or_else(|| ABSENT.to_owned(), |n| n.to_string())
}

fn tokens(token: &str) -> Option<u64> {
    token.parse().ok()
}

pub(super) fn clip(reason: &str) -> String {
    let flat = reason.replace(['\n', '\r'], " ");
    if flat.chars().count() > REASON_MAX {
        flat.chars().take(REASON_MAX).collect()
    } else {
        flat
    }
}

#[cfg(test)]
mod tests;
