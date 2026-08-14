//! The mutating fan's two boundary executors (VISION §4.10, bl-8746): spread a
//! prepared start over N candidates, and retire one of them. Split out of
//! [`dispatch`](super::dispatch) for the reason every other family is — the
//! chokepoint is a table, and a table stops being one when arms carry bodies.
//!
//! **Both are in-process calls into the linked balls crate, not spawns**, and
//! that is upstream's own ruling rather than a yog shortcut: balls' attempt
//! capability has no `bl` verb and *must not* have one, because a verb would be
//! a second entry point to a capability whose whole point is that the N = 1 ball
//! path and the N > 1 candidate paths are one mechanism. So the §4.2 trail
//! records them as **steps** (`["yog-step","fan"]` / `["yog-step","retire"]`)
//! rather than as argv rows — the same shape the arming writes and the start
//! flow's mint use, and the same rule: an attempted action always leaves a
//! durable line, whichever kind it was.

use std::time::SystemTime;

use crate::fan::{self, Obligation, retention};
use crate::opslog::{self, OpEntry, Origin};
use crate::start::Prepared;

use super::dispatch::Deps;
use super::reply::Reply;

/// The step names the two gestures log under.
const FAN_STEP: &str = "fan";
const RETIRE_STEP: &str = "retire";

/// Materialize N candidates and answer with the prepared start rebound to each
/// (§4.10 item 1). Nothing is fired here: each element is spent by the ordinary
/// [`Prompt`](super::Action::Prompt) gesture, so the §3.5 spend ceiling gates
/// every birth exactly as it gates a single start and this door needs no gate
/// of its own.
pub(super) fn spread(
    deps: &Deps,
    ts: &str,
    prepared: &Prepared,
    obligation: &Obligation,
    n: usize,
) -> Result<Reply, String> {
    let xdg = deps.world.balls_layout();
    // The one resolution (REMOTE §8): the obligation names its project, the
    // chokepoint turns that name into the repo everything below works in.
    let repo = deps.snapshot.project_path(&obligation.project)?;
    let spread = fan::spread(prepared, obligation, &repo, &xdg, n).map_err(|e| e.to_string());
    logged(deps, ts, &repo, FAN_STEP, spread).map(Reply::Fanned)
}

/// Retire one candidate (§4.10 items 4 and 6): release the worktree always, and
/// discard the source ref **only** when this project's declared retention has
/// expired. The two are separate balls calls and the policy is world config, so
/// deleting the `retention:` entry restores the standing default — keep the ref
/// — without touching a line of this.
pub(super) fn retire(
    deps: &Deps,
    ts: &str,
    obligation: &Obligation,
    handle: &str,
) -> Result<Reply, String> {
    let xdg = deps.world.balls_layout();
    let repo = deps.snapshot.project_path(&obligation.project)?;
    let keep = retention::keep(&cadence(deps), &repo);
    let discarded = retention::expired(keep, retention::age(&repo, handle, SystemTime::now()));
    let spent = if discarded {
        fan::discard(obligation, &repo, &xdg, handle)
    } else {
        fan::release(obligation, &repo, &xdg, handle)
    };
    logged(
        deps,
        ts,
        &repo,
        RETIRE_STEP,
        spent.map_err(|e| e.to_string()),
    )
    .map(|()| Reply::Retired { discarded })
}

/// The world's clock-settings file, which is where every yog policy block lives
/// (§7.2) — absent reads as empty, and an empty file declares nothing.
fn cadence(deps: &Deps) -> String {
    std::fs::read_to_string(deps.state_root.join(crate::app::cadence::CADENCE_YAML))
        .unwrap_or_default()
}

/// Run one in-process act onto the §4.2 trail: a completed step either way, its
/// error carried in the failure line's `stderr` — so a fan that balls refused
/// is as visible on the trail as a `bl` verb that exited non-zero.
fn logged<T>(
    deps: &Deps,
    ts: &str,
    repo: &std::path::Path,
    step: &str,
    outcome: Result<T, String>,
) -> Result<T, String> {
    let cwd = repo.display().to_string();
    let entry = match outcome.as_ref().err() {
        Some(err) => OpEntry::step_failure(ts.to_owned(), step, cwd, err.clone(), Origin::Balls),
        None => OpEntry::step_done(ts.to_owned(), step, cwd, Origin::Balls),
    };
    opslog::append(&deps.state_root, &entry).map_err(|e| e.to_string())?;
    outcome
}

#[cfg(test)]
mod tests;
