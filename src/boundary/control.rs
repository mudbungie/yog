//! The capability boundary's one executor (VISION §4.11, DESIGN §8.6):
//! **answering a parked invocation**.
//!
//! The control itself writes nothing — it is re-consulted on every drive, so a
//! consult with a side effect would answer differently the second time. The
//! writer is here, and it writes exactly one `ops.jsonl` row:
//!
//! ```text
//! ["yog-control","answer",<key>,"pass"|"hold"|"refuse",<scope>]
//! ```
//!
//! which is at once the audit and the fold's memory ([`crate::control::judge`]
//! reads it back). No fourth durable artifact; the §4.9 monitor's own pattern.
//!
//! **Three moves, in this order, and each one earns its place.**
//!
//! 1. *Read the mark, live.* The held `tool_use` id is never typed and never
//!    carried from a snapshot: it is read off `refs/litany/held/<agent>` at
//!    fire time, so the answer names what is parked now. Nothing parked is a
//!    refusal — a gesture is an instruction, and an answer aimed at nothing
//!    must say so rather than report a silent success. **An answer wider than
//!    the call reads the mark for its CLASS too** (bl-94a5): the sentence yog
//!    wrote into that mark says which class was parked
//!    ([`crate::control::reason::class_of`]), and the class is half the key a
//!    standing answer stands over. A mark whose class cannot be read takes
//!    `--scope call` only — fail-closed, because a key nobody can compute is a
//!    grant nobody can bound.
//! 2. *Write the row.* Durable before anything is launched, so a driver that
//!    re-consults a microsecond later already sees the answer. The reverse
//!    order would race the very thing it is trying to release.
//! 3. *Advance, detached* — but only when the answer **releases**. `pass` and
//!    `refuse` both move the branch (one executes, one declines in band); a
//!    `hold` answer is the operator saying *stay parked*, and launching a
//!    driver to re-park would spend a process to reach the state it is already
//!    in. The launch is detached for the reason every driver launch is: an
//!    `advance` runs the conversation until it goes quiet, and no gesture may
//!    block a frame or a consumer thread on that.
//!
//! **No enforcement path calls stop.** `litany stop` mid-tool-window wedges the
//! branch permanently (litany bl-b98d), so declining is in-band and parking is
//! a park — never a kill.

use std::path::Path;

use crate::control::hold;
use crate::control::judge::{Answer, Ruling, Scope, class_key};
use crate::opslog::{self, OpEntry, Origin, YOG_CONTROL};

use super::dispatch::Deps;
use super::reply::Reply;

/// The releasing driver launch, shared with the §8.2 nudge — its own file
/// because it is a *launch*, not a judgment: nothing in it reads a mark, a row
/// or a policy.
mod drive;
pub(super) use drive::advance;

/// The family's other writer — the §4.9 fifth rung's per-conversation floor
/// (bl-94b4). Its own file on a real seam: this one answers **one invocation**
/// off a live mark and drives the branch on; that one writes **standing
/// policy** for a whole descent and launches nothing.
mod floor;
pub(super) use floor::set_floor;

/// The ops-row verb naming an answer to a held call. Mirrored from the fold
/// that reads it (`crate::control::judge::answers`); the words are held equal
/// by a test rather than by a shared const, because the reader deliberately
/// owns its grammar.
const ANSWER: &str = "answer";

/// **What one answered park is**: the invocation the answer landed on, read
/// live off the mark rather than typed; the tool it named; the answer written
/// — verdict and the scope it now stands over; and whether the releasing
/// `litany advance` was launched. Its own named type rather than four fields
/// on [`Reply`], the [`Acknowledged`](crate::boundary::answer::queue::Acknowledged)
/// shape: a receipt is a thing, and this one is minted here and spelled in
/// exactly two other places.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Answered {
    pub tool_use: String,
    pub tool: String,
    pub answer: Answer,
    pub advanced: bool,
}

/// Answer the invocation parked at `(workspace, agent)`.
pub(super) fn answer_hold(
    deps: &Deps,
    ts: &str,
    workspace: &Path,
    agent: &str,
    answer: Answer,
) -> Result<Reply, String> {
    let held = hold::read(workspace, agent).ok_or_else(|| {
        // The §3.1 name, never the path (REMOTE §8.1, bl-ef16): a refusal is a
        // reply body, it crosses to whatever seat asked, and it names the very
        // workspace that seat addressed the gesture by.
        format!(
            "nothing is held on {agent:?} in {:?} — the capability boundary is parking no \
             invocation there",
            crate::naming::leaf(workspace)
        )
    })?;
    let key = key(&held, agent, answer.scope)?;
    let row = OpEntry {
        ts: ts.to_owned(),
        argv: vec![
            YOG_CONTROL.to_owned(),
            ANSWER.to_owned(),
            key,
            answer.ruling.word().to_owned(),
            answer.scope.word().to_owned(),
        ],
        cwd: crate::nav::ws_key(workspace),
        exit: 0,
        stdout: held.reason.clone(),
        stderr: String::new(),
        // The subject is a conversation, which is what §7.3 attribution names.
        origin: Origin::Conversation,
        client: deps.caller.client.clone(),
    };
    opslog::append(&deps.state_root, &row).map_err(|e| e.to_string())?;
    let advanced = answer.ruling != Ruling::Hold && advance(deps, ts, workspace, agent).is_ok();
    Ok(Reply::Answered(Answered {
        tool_use: held.tool_use_id,
        tool: held.tool,
        answer,
        advanced,
    }))
}

/// **What this answer stands over**, in one word for the row: the held
/// `tool_use` id at [`Scope::Call`], and the class of the held call — the same
/// tool at the same reach — at either wider scope, prefixed by the answering
/// conversation where the scope is that conversation's descent.
///
/// Two refusals, and both are the fail-closed direction. A class the mark's
/// sentence does not name cannot bound a standing grant, so the answer is
/// narrowed to the call by refusing outright rather than by silently answering
/// something else; and loss and credentials take the call alone
/// ([`Answer::permits`]), which is the shipped table's own line answering a new
/// question rather than a new floor.
fn key(held: &hold::Held, agent: &str, scope: Scope) -> Result<String, String> {
    if scope == Scope::Call {
        return Ok(held.tool_use_id.clone());
    }
    let effect = crate::control::reason::class_of(&held.reason).ok_or_else(|| {
        format!(
            "the hold on {agent:?} does not say which class it parked, so only --scope call can \
             be answered there: a standing answer stands over a class of calls and this one \
             cannot be named"
        )
    })?;
    scope.permits(effect)?;
    let class = class_key(&held.tool, effect);
    match scope {
        Scope::Conversation => Ok(format!("{agent} {class}")),
        // The workspace is the row's own `cwd`, never a second copy of it in
        // the argv: one fact, one home, and the fold reads it back from there.
        _ => Ok(class),
    }
}

/// The §4.11 item-8 **confinement gate**: a workspace whose live policy
/// declares `confinement: required` fires a drone only where the platform's
/// one backend proves itself at this very birth — the derivation, the probe
/// and the refusal all live in [`crate::control::confine`]; this is the doors'
/// name for them. On Linux the backend is bubblewrap and a passing probe means
/// the fired spawn runs wrapped (the doors fold the wrapper on); everywhere
/// else, and wherever the probe fails, the standing refusal names exactly why.
/// Never a silent fallback, and no UI affordance for an absent layer — the
/// only surface it earns is the refusal.
///
/// Severable in both directions: absent, the gate is a no-op with nothing
/// configured; present, removing the line removes the policy, not the code.
pub(super) fn confinement_gate(workspace: &Path) -> Result<(), String> {
    crate::control::confine::gate(workspace)
}

#[cfg(test)]
mod tests;
