//! The capability boundary's one executor (VISION §4.11, DESIGN §8.6):
//! **answering a parked invocation**.
//!
//! The control itself writes nothing — it is re-consulted on every drive, so a
//! consult with a side effect would answer differently the second time. The
//! writer is here, and it writes exactly one `ops.jsonl` row:
//!
//! ```text
//! ["yog-control","answer",<tool_use id>,"pass"|"hold"|"refuse"]
//! ```
//!
//! which is at once the audit and the fold's memory ([`crate::control::judge`]
//! reads it back). No fourth durable artifact; the §4.9 monitor's own pattern.
//!
//! **Three moves, in this order, and each one earns its place.**
//!
//! 1. *Read the mark, live.* The held `tool_use` id is never typed and never
//!    carried from a snapshot: it is read off `refs/lernie/held/<agent>` at
//!    fire time, so the answer names what is parked now. Nothing parked is a
//!    refusal — a gesture is an instruction, and an answer aimed at nothing
//!    must say so rather than report a silent success.
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
//! **No enforcement path calls stop.** `lernie stop` mid-tool-window wedges the
//! branch permanently (lernie bl-b98d), so declining is in-band and parking is
//! a park — never a kill.

use std::path::Path;

use crate::control::hold;
use crate::control::judge::Ruling;
use crate::opslog::{self, DETACHED_EXIT, OpEntry, Origin, YOG_CONTROL};

use super::dispatch::Deps;
use super::reply::Reply;

/// The family's other writer — the §4.9 fifth rung's per-conversation floor
/// (bl-94b4). Its own file on a real seam: this one answers **one invocation**
/// off a live mark and drives the branch on; that one writes **standing
/// policy** for a whole descent and launches nothing.
mod floor;
pub(super) use floor::set_floor;

/// The ops-row verb naming a once-answer. Mirrored from the fold that reads it
/// ([`crate::control::judge`]); the two words are held equal by a test rather
/// than by a shared const, because the reader deliberately owns its grammar.
const ANSWER: &str = "answer";

/// lernie's re-drive verb — `lernie advance <ws> <agent>` (its ARCH §6): one
/// hop of the workflow chain, which re-enters the tool window under the mark
/// and re-consults the control. That re-consult *is* the release.
const ADVANCE: &str = "advance";

/// Answer the invocation parked at `(workspace, agent)`.
pub(super) fn answer_hold(
    deps: &Deps,
    ts: &str,
    workspace: &Path,
    agent: &str,
    ruling: Ruling,
) -> Result<Reply, String> {
    let held = hold::read(workspace, agent).ok_or_else(|| {
        format!("nothing is held on {agent:?} in {} — the capability boundary is parking no invocation there", workspace.display())
    })?;
    let row = OpEntry {
        ts: ts.to_owned(),
        argv: vec![
            YOG_CONTROL.to_owned(),
            ANSWER.to_owned(),
            held.tool_use_id.clone(),
            ruling.word().to_owned(),
        ],
        cwd: crate::nav::ws_key(workspace),
        exit: 0,
        stdout: held.reason.clone(),
        stderr: String::new(),
        // The subject is a conversation, which is what §7.3 attribution names.
        origin: Origin::Conversation,
    };
    opslog::append(&deps.state_root, &row).map_err(|e| e.to_string())?;
    let advanced = ruling != Ruling::Hold && advance(deps, ts, workspace, agent).is_ok();
    Ok(Reply::Answered {
        tool_use: held.tool_use_id,
        tool: held.tool,
        ruling,
        advanced,
    })
}

/// Fire `lernie advance <ws> <agent>` detached, logging the launch exactly as
/// the §8.1 fire logs its own: [`DETACHED_EXIT`] for a handoff that happened, a
/// §4.2 synthetic-failure line for a fork that never landed. The row is the
/// receipt — the answer's own reply says only whether the launch was made.
///
/// **Two callers, one body** (bl-9bef): the release above, and the §8.2 nudge —
/// the operator's own "run it again from here", which is this launch and
/// nothing else, since lernie derives what is due from the transcript tail
/// (ARCH §6). Shared rather than re-written, so a driver launch has one home.
///
/// **The spawn is workspace-bound** ([`Deps::bound`], bl-bf79): what this
/// launches is a *driver*, which makes model calls, so it owes its workspace
/// the §16.2 wall — without it the driver's first `bz` dies with `no workspace
/// in this environment` and the turn produces an empty reply. That is the same
/// fold every §8.2 lernie verb takes, and it was missing here.
pub(super) fn advance(deps: &Deps, ts: &str, workspace: &Path, agent: &str) -> Result<(), String> {
    let ws_s = workspace.to_string_lossy();
    let sink = opslog::detached::sink(&deps.state_root, ts, workspace);
    let bound = deps.bound(workspace);
    let spawn =
        bound
            .cli()
            .spawn_detached(Some(workspace), &sink, &[ADVANCE, ws_s.as_ref(), agent]);
    let argv = vec![
        deps.lernie.binary().display().to_string(),
        ADVANCE.to_owned(),
        ws_s.into_owned(),
        agent.to_owned(),
    ];
    let cwd = crate::nav::ws_key(workspace);
    let entry = match spawn.as_ref().err() {
        Some(e) => OpEntry::synthetic_failure(
            ts.to_owned(),
            argv,
            cwd,
            e.to_string(),
            Origin::Conversation,
        ),
        None => OpEntry {
            ts: ts.to_owned(),
            argv,
            cwd,
            exit: DETACHED_EXIT,
            stdout: String::new(),
            stderr: String::new(),
            origin: Origin::Conversation,
        },
    };
    opslog::append(&deps.state_root, &entry).map_err(|e| e.to_string())?;
    spawn.map(|_pid| ()).map_err(|e| e.to_string())
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
