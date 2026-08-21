//! **Retention** — the one policy that turns a released candidate into a
//! discarded one (VISION §4.10 items 4 and 6; DESIGN §3.8).
//!
//! §4.10 item 6, verbatim: *"Losers stay inspectable until yog's retention
//! policy (world config, severable) asks balls to clean them."* This is that
//! policy, and it is an entry in the file yog's clock already owns — a sibling
//! of `cadence:`, `fleet:` and the monitor's block, a row rather than a
//! rebuild:
//!
//! ```text
//! retention:
//!   /home/u/dev/yog:
//!     keep_min: 1440
//! ```
//!
//! **Absence is never discard**, and that default is deliberate for exactly the
//! reason the armed loop's `lease_min` is absent by default
//! ([`crate::fleet::arming`]: *"a reap releases a claim and yog must not do that
//! on an opinion"*). Deleting a source ref destroys the only remaining record of
//! a rejected candidate — balls itself never sweeps attempts — so yog will not
//! do it on a default it invented. **Severability is deleting the entry**: what
//! goes is config, never code.
//!
//! The entry key is the **project** path, not a workspace: retention is about
//! refs in a project repo, and every workspace that fans that project is
//! spending the same ref namespace.

use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use balls::delivery_path::attempt_branch;

use crate::model_pick::grammar::entry_field;

/// The column-0 block key, beside [`fleet`](crate::fleet::arming::BLOCK).
pub const BLOCK: &str = "retention";
/// How long a candidate's source ref is kept, in whole minutes.
pub const KEEP_MIN: &str = "keep_min";

/// How long this project keeps a candidate's source ref. `None` — the default,
/// and the answer for every project with no entry — keeps it forever.
pub fn keep(text: &str, project: &Path) -> Option<Duration> {
    entry_field(text, BLOCK, &project.to_string_lossy(), KEEP_MIN)
        .filter(|v| !v.is_empty())
        .and_then(|m| m.parse::<u64>().ok())
        .map(|m| Duration::from_secs(m.saturating_mul(60)))
}

/// Whether a candidate of this `age` has outlived `keep`. An undeclared
/// retention never expires, and an unreadable age
/// ([`age`] answering `None`) never expires either — yog discards on a fact or
/// not at all.
pub fn expired(keep: Option<Duration>, age: Option<Duration>) -> bool {
    match (keep, age) {
        (Some(keep), Some(age)) => age >= keep,
        _ => false,
    }
}

/// How long ago this candidate's source ref last moved — the committer time of
/// `attempt/<handle>`'s tip, against `now`. The ref name is balls'
/// ([`attempt_branch`]), never a literal here.
///
/// `None` when the ref does not resolve, when git will not run, or when the tip
/// is in the future (an unusable clock): in each the caller keeps the ref, which
/// is the standing default.
pub fn age(project: &Path, handle: &str, now: SystemTime) -> Option<Duration> {
    let out = crate::git_env::output(crate::git_env::git().arg("-C").arg(project).args([
        "log",
        "-1",
        "--format=%ct",
        &attempt_branch(handle),
    ]))
    .ok()
    .filter(|out| out.status.success())?;
    let secs: u64 = String::from_utf8_lossy(&out.stdout).trim().parse().ok()?;
    now.duration_since(UNIX_EPOCH + Duration::from_secs(secs))
        .ok()
}

#[cfg(test)]
mod tests;
