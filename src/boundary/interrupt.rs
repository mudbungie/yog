//! **Send-and-interrupt** (§8.2, bl-a33d): the one executor behind
//! [`Action::Interrupt`](super::Action::Interrupt) — stop whatever is running
//! this conversation, then deposit the operator's text, and let the deposit's
//! own driver-start be the trigger.
//!
//! **No new substrate verb.** It is `lernie stop` followed by `lernie message`,
//! in that order, through the two §8.2 short verbs that already exist
//! ([`verbs::stop`], [`verbs::message`]) and the workspace-bound spawn they
//! already take (bl-bf79). The trigger is lernie's standing law rather than
//! anything yog adds: ARCH §2.9 has no resume verb, so a deposit into a
//! quiescent branch *is* a driver start — which is exactly the state the stop
//! just put the branch in.
//!
//! **Two ops rows, one gesture** (§4.2). Each short verb logs its own attempt,
//! so the trail carries the interrupt and the deposit separately and a reader
//! can see that a stop fired and what it answered. A composite row would hide
//! precisely the half that can fail on its own, which is the ruling this
//! gesture was filed under. Nothing here logs a third row.
//!
//! **What each failure means.** A stop whose spawn never happened aborts the
//! gesture (`?`): the handle is broken, the deposit would fail the same way, and
//! the synthetic failure row is already on the trail. A stop that *ran* and was
//! declined — nothing in flight, no such conversation — carries on to the
//! deposit, because a declined stop is the answer "there was no work to cut
//! short" and the deposit is then the whole gesture. So the reply is the
//! **deposit's** outcome: it is the act that resumes the conversation, and its
//! exit is the one the operator is waiting on.
//!
//! **Why this could not be built until lernie bl-b98d landed.** A stop landing
//! in a *tool window* used to leave the branch tip at lernie's ARCH §6 one
//! non-replayable state: the assistant entry with its `tool_use` blocks was
//! already committed, the felled invocations had no `tool_result`, and the next
//! `lernie advance` — which is exactly what this gesture's deposit starts —
//! declined with `Error::UnpairedToolUse`. The gesture would have bricked any
//! conversation it happened to catch mid-tool-call. The pinned lernie settles
//! that window on the stopped exit (`prompt::dispatch::tool_step::settle`): one
//! in-band `is_error` `tool_result` per unanswered `tool_use`, so the tail
//! warrants an ordinary model call and the deposit revives the branch. yog
//! carries no guess about which window a stop landed in — a yog-side read of
//! lernie's step state would be a race, not a mechanism.

use std::path::Path;

use crate::actions::verbs;

use super::dispatch::Deps;
use super::reply::Reply;

/// Interrupt `agent` in `workspace` and deposit `content` (§8.2). `ts` is the
/// §4.2 stamp both rows carry, so the pair reads as one gesture on the trail.
pub(crate) fn interrupt(
    deps: &Deps,
    ts: &str,
    workspace: &Path,
    agent: &str,
    content: &str,
) -> Result<Reply, String> {
    let bound = deps.bound(workspace);
    let root = deps.state_root.as_path();
    // The children cascade is deliberately not offered: this gesture's subject
    // is the conversation the operator is talking to, and `/stop children` is
    // the gesture for a subtree. One knob fewer, and no seat has to decide.
    verbs::stop(&bound, root, ts, agent, false).map_err(|e| e.to_string())?;
    verbs::message(&bound, root, ts, agent, content)
        .map(Reply::Outcome)
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests;
