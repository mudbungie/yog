//! The §8.2 short-verb invocations the shell shares across its three carriers:
//! the composer's buttons ([`super::input_bar`]), the §11 key bindings
//! ([`super::keys`]), and the conversation-row accelerator menu
//! ([`super::menus`]). Each verb comes in two spellings — one that resolves the
//! *focused* selection, and one that names its target outright, so the
//! pointer path never touches the focus.
//!
//! Coverage-excluded glue like the rest of `src/shell/*`: every body here
//! **constructs a boundary [`Action`] and posts it** (§8.5, REMOTE §9.8) — the
//! chokepoint and the enablement predicates it honours are tested; this file
//! only routes.
//!
//! **Four of the five cross the wire** (bl-4841). Stop, Scan, Nudge and the §8.6
//! hold answer read no receipt and never did — their durable record is the
//! `ops.jsonl` line — so they post and hold nothing, and the `Cli` pair went
//! with the dispatch. [`message`] is the one that stays in process for now: its
//! reply gates two frame-side facts in the same breath, the draft clearing and
//! the §3.4 pending echo, and moving those onto a receipt is REMOTE §9.8's
//! named residual.
//!
//! No error is ever printed and dropped (INV-2): every outcome is the durable
//! `ops.jsonl` line the activity pane and the §7.3 banner read back per frame.

use crate::AppModel;
use crate::boundary::{Action, reply};
use crate::cli_outbound::Cli;
use std::path::Path;

use super::ShellState;

/// Stop the **selected conversation** (§8.2) — the §11 `x` binding and the Stop
/// button's one implementation. Re-derives its target from the focus, so it is
/// refused exactly where the button is disabled: no workspace, no selection, or
/// an agent the §11 seat's own `stoppable` gate says is not stoppable.
pub(super) fn stop_selected(model: &mut AppModel, state: &mut ShellState) {
    let (Some(ws), Some(seat)) = (
        model.focused_workspace().map(Path::to_path_buf),
        model.focused_conversation(),
    ) else {
        return;
    };
    // The gate the button paints is the gate this runs (REMOTE §9.4, bl-1eb0):
    // one fact off the seat's own view, not a second reading of the tree.
    if !seat.stoppable {
        return;
    }
    stop_agent(model, &ws, &seat.agent_id, state.actions.stop_children);
}

/// Stop **one named agent** (§8.2) — the body [`stop_selected`] runs once it has
/// resolved the selection, and the same call the §11 conversation-row menu makes
/// on the row under the pointer (`super::menus`). The target is a parameter, not
/// a re-derivation, so the pointer path never touches the focus.
pub(super) fn stop_agent(model: &mut AppModel, ws: &Path, agent: &str, children: bool) {
    super::act::fire(
        model,
        &Action::Stop {
            workspace: model.snap.ws_name(ws),
            agent: agent.to_owned(),
            children,
        },
    );
}

/// Flush the focused workspace's inbox — `lernie scan` (§8.2): the §11 `f`
/// binding and the Scan button's one implementation.
pub(super) fn scan_focused(model: &mut AppModel) {
    let Some(ws) = model.focused_workspace().map(Path::to_path_buf) else {
        return;
    };
    scan_ws(model, &ws);
}

/// Flush **one named workspace's** inbox (§8.2) — [`scan_focused`]'s body, shared
/// with the conversation-row menu's Flush, which names the row's workspace rather
/// than the focus.
pub(super) fn scan_ws(model: &mut AppModel, ws: &Path) {
    super::act::fire(
        model,
        &Action::Scan {
            workspace: model.snap.ws_name(ws),
        },
    );
}

/// Send one message (§8.2's resume gesture) — the composer's Message button and
/// Enter, one body: the boundary variant, dispatched, then the ordinary
/// lernie-verb aftermath. Returns whether the draft clears (a clean send only).
pub(super) fn message(
    model: &mut AppModel,
    lernie: &Cli,
    bl: &Cli,
    ws: &Path,
    agent: &str,
    content: &str,
) -> bool {
    let deps = model.boundary_deps(lernie, bl);
    let action = Action::Message {
        workspace: model.snap.ws_name(ws),
        agent: agent.to_owned(),
        content: content.to_owned(),
    };
    let sent = model.dispatch(&deps, &super::now_ts(), &action);
    let cleared = after_lernie(&sent, model);
    // A clean send holds §7.2's pending echo — the deposit is piped and its
    // `NNN-user.md` only appears on the driver's next step boundary, so without
    // this the operator's own words leave the screen with the draft. Gated on
    // the same predicate the draft clear is: a refused send has nothing to echo.
    if cleared {
        model.await_message(ws, agent, content);
    }
    cleared
}

/// Fire inference on one conversation from where it already stands (§8.2,
/// bl-9bef) — the composer's Nudge button, one body. It carries no payload at
/// all: the target is the parameter and the conversation's own state is the
/// prompt, so there is nothing here to clear and nothing to echo.
pub(super) fn nudge(model: &mut AppModel, ws: &Path, agent: &str) {
    super::act::fire(
        model,
        &Action::Nudge {
            workspace: model.snap.ws_name(ws),
            agent: agent.to_owned(),
        },
    );
}

/// Answer the invocation parked at one conversation's capability boundary
/// (§8.6) — the composer's two hold buttons, one body. The held `tool_use` id
/// is the executor's to derive; this seat says only *which conversation* and
/// *which verdict*, exactly as the line does.
pub(super) fn answer_hold(
    model: &mut AppModel,
    ws: &Path,
    agent: &str,
    ruling: crate::control::judge::Ruling,
) {
    super::act::fire(
        model,
        &Action::AnswerHold {
            workspace: model.snap.ws_name(ws),
            agent: agent.to_owned(),
            ruling,
        },
    );
}

/// Fold a `lernie` verb's aftermath: refresh the ops tail; return whether it
/// cleanly succeeded ([`reply::cleared`], the covered predicate) so a draft
/// clears only on a clean send. The banner is not touched here — it is derived
/// per frame from the refreshed tail (§7.3).
pub(super) fn after_lernie(result: &Result<reply::Reply, String>, model: &mut AppModel) -> bool {
    model.after_lernie_verb();
    reply::cleared(result)
}
