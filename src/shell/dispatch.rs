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
//! **All five cross the wire** (bl-4841, completed bl-1747). Stop, Scan, Nudge
//! and the §8.6 hold answer read no receipt and never did — their durable
//! record is the `ops.jsonl` line — so they post and hold nothing. `Message`
//! was the last one held back, because its reply gated two frame-side facts in
//! the same breath: the draft clearing and the §3.4 pending echo. Both now hang
//! off the **receipt** ([`super::acting`]) rather than a synchronous `Ok`, so
//! this file constructs the variant and hands it over like every other verb,
//! and the `Cli` pair went with the last dispatch.
//!
//! No error is ever printed and dropped (INV-2): every outcome is the durable
//! `ops.jsonl` line the activity pane and the §7.3 banner read back per frame.

use crate::AppModel;
use crate::boundary::Action;
use std::path::Path;

use super::ShellState;

/// Stop the **selected conversation** (§8.2) — the §11 `x` binding and the Stop
/// button's one implementation. Re-derives its target from the focus, so it is
/// refused exactly where the button is disabled: no workspace, no selection, or
/// an agent the §11 seat's own `stoppable` gate says is not stoppable.
pub(super) fn stop_selected(model: &mut AppModel, state: &mut ShellState) {
    let (Some(ws), Some(seat)) = (model.focused_workspace(), super::seat::selection(model)) else {
        return;
    };
    // The gate the button paints is the gate this runs (REMOTE §9.4, bl-1eb0;
    // §9.7, bl-48ae): one fact off the **landed forest**, read in the frame the
    // click happened, not a second reading of the tree and not an answer that
    // arrives an ask period after the key.
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
    let Some(ws) = model.focused_workspace() else {
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

/// Send one message (§8.2's resume gesture) — the composer's Message button
/// and Enter, one body: the boundary variant, posted, with the draft `key` it
/// was composed in held for the receipt. **Whether the words leave the screen
/// is the engine's answer, not the click's** (§5.3, REMOTE §9.8): a clean
/// deposit clears the draft and raises the §3.4 echo; anything else leaves them
/// where they can be fixed.
pub(super) fn message(
    model: &mut AppModel,
    state: &mut ShellState,
    key: &crate::actions::DraftKey,
    ws: &Path,
    agent: &str,
    content: &str,
) {
    let action = Action::Message {
        workspace: model.snap.ws_name(ws),
        agent: agent.to_owned(),
        content: content.to_owned(),
    };
    super::acting::deposit(model, state, key, ws, &action);
}

/// Send **and interrupt** (§8.2, bl-a33d) — the composer's Interrupt button and
/// Ctrl+Enter, one body. The seat says only which conversation and what to say;
/// that the stop goes first, and that the deposit's own driver-start is the
/// trigger, are the executor's ([`crate::boundary::interrupt`]). The aftermath
/// is `message`'s exactly, because this deposits too: a clean landing clears
/// **this** draft and raises the §3.4 echo.
pub(super) fn interrupt(
    model: &mut AppModel,
    state: &mut ShellState,
    key: &crate::actions::DraftKey,
    ws: &Path,
    agent: &str,
    content: &str,
) {
    let action = Action::Interrupt {
        workspace: model.snap.ws_name(ws),
        agent: agent.to_owned(),
        content: content.to_owned(),
    };
    super::acting::deposit(model, state, key, ws, &action);
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
