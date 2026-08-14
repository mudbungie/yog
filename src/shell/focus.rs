//! **Composer focus discipline** (DESIGN §11): the one mechanism deciding when
//! the keyboard lands in the message box, so the operator can just type.
//!
//! Not to be confused with [`crate::app::focus`], which owns the *selection* —
//! which conversation is picked. This module owns the **keyboard**,
//! and it owns it through exactly one piece of state: the deferred request bit
//! [`ShellState::focus_composer`]. Nothing else in `shell/*` may call
//! `request_focus` on a composer; a gesture states its intent by setting the
//! bit, and whichever composer paints next frame — the bottom one or the
//! empty-world bootstrap — consumes it in [`take`]. egui focus is per-frame and
//! a gesture is handled before any widget exists, so "next frame" is not a
//! delay bolted on: it is the only frame that has a box to hand the focus to.
//!
//! [`take`] also schedules a **one-frame repeat** of the bit it just spent
//! (`refocus_composer`, bl-58e4). That is not a second request path — no gesture
//! reaches it and nothing else reads it — but the bit's own delivery, because a
//! request that rode an arrow key does not actually land on the first frame.
//! The reason is ruled at [`take`].
//!
//! It also carries the §11 **list gestures** that move the selection since the
//! unfold (bl-fa82) — the ↑/↓ walk over the *visible* rows, the ←/→ that fold
//! one, and the field's click — because each of them ends by handing the
//! keyboard over under the rules below, and each is a thin call onto
//! `nav::convs::expand`'s tested derivations rather than a decision made here.
//!
//! Two rules decide who sets it, and there is no third:
//!
//! 1. **A pointer gesture hands the keyboard back.** Opening a conversation,
//!    switching workspace, launching, sending, and dismissing a modal all end
//!    with the cursor in the box. The mouse said *where*; the keyboard's only
//!    remaining job is *what to say*.
//! 2. **A selection lands the composer no matter the plane it rode.** Opening
//!    the app focuses the chat prompt, and so does selecting an agent, which
//!    supersedes the older rule that a keyboard gesture left the keyboard plane alone. A
//!    bare ↓ therefore surrenders the very plane it was pressed on, and that
//!    cost is accepted: the walk's continuation is spelled on the combo plane
//!    (Ctrl+↑/↓), which survives text focus. `i` / Ctrl+I stays the explicit
//!    request from anywhere and Escape the release — and Escape with nothing
//!    pending must still not re-grab, since with every selection landing here it
//!    is the one door back to the bare plane.

use crate::AppModel;
use crate::keymap::CenterTab;
use std::path::{Path, PathBuf};

use super::ShellState;

/// Ask that the composer take the keyboard on the next frame it is painted
/// (§11) — and seat the center on the tab that *has* one.
///
/// Still one request bit. The seat beside it is not a second mechanism but the
/// bit's precondition: since bl-1ca2 the center is a strip of tab focuses and
/// only the Conversation tab carries a composer, so a request made while
/// Config is up would wait for a box that never paints. Asking for the
/// keyboard **is** asking for the surface that takes it, which is why this is
/// one call and not two at every site.
pub(super) fn request(state: &mut ShellState) {
    state.center = CenterTab::Conversation;
    state.focus_composer = true;
}

/// Focus a §11 center tab — the left-panel entries, the strip itself, the
/// keyboard's Command+Shift+digit, and Escape's way back.
///
/// Landing on the conversation is a selection like any other and hands the
/// keyboard back (rule 1); the other tabs hold no composer, so they take the
/// seat and leave the keyboard where it is — which is what keeps Escape the
/// door back to the bare plane rather than a re-grab.
pub(super) fn center(state: &mut ShellState, tab: CenterTab) {
    match tab {
        CenterTab::Conversation => request(state),
        other => state.center = other,
    }
}

/// Honour a pending request on the composer just painted, consuming it — the
/// **only** `request_focus` on a composer in the tree. Called unconditionally
/// by every composer; the bit decides.
///
/// **A request that rode an arrow key is asked again next frame** (bl-58e4).
/// egui walks the *focus floor* on a bare arrow (its own cardinal navigation),
/// and a widget that GAINS focus during a frame has not yet installed the event
/// filter that would claim the arrow for itself — so the very ↑/↓ that made the
/// selection is also read as "step the floor one control on", and the keyboard
/// lands on whatever sits under the box rather than in it. Rule 2 was therefore
/// only ever nearly true: `wants_keyboard_input` said yes while the cursor was
/// on the Send button. Re-asking on the next frame — which no longer carries
/// the key — makes it true outright, and a second request on a box that already
/// has the keyboard is a no-op.
///
/// The band reorder is what made "nearly" not good enough: the
/// control under the box is now the settings band, whose height settles a frame
/// late, so the control the floor stepped onto could vanish mid-settle and take
/// the keyboard down with it.
///
/// **Escape outranks a carried request**, exactly as rule 2 says it outranks a
/// standing one — it is the one door back to the bare plane — so a request still
/// in flight from the frame before must not re-grab on the frame that puts the
/// keyboard down. An Escape frame that *also* asks is a different thing and is
/// honoured: that is a dismissed modal handing the keyboard back (rule 1).
pub(super) fn take(state: &mut ShellState, ui: &egui::Ui, edit: &egui::Response) {
    let (escape, arrow) = ui.input(|i| {
        (
            i.key_pressed(egui::Key::Escape),
            i.modifiers.is_none()
                && (i.key_pressed(egui::Key::ArrowDown) || i.key_pressed(egui::Key::ArrowUp)),
        )
    });
    // Escape outranks the **repeat**, never the ask itself.
    let repeat = std::mem::take(&mut state.refocus_composer) && !escape;
    if !std::mem::take(&mut state.focus_composer) && !repeat {
        return;
    }
    edit.request_focus();
    state.refocus_composer = arrow;
}

/// Select a conversation and hand the keyboard over (rules 1 and 2) — a list
/// row, a descent-tree member, a followed card. The selection is the §6
/// acknowledgement gesture ([`AppModel::focus_agent`]) and it re-targets the
/// composer, so the box the operator lands in is already aimed at what they
/// just picked.
///
/// Revealing what was selected is **not** here — [`reveal_selection`] is the
/// one home of that, read by the list itself, so a selection made anywhere
/// (including outside this module) lands on a row the operator can see.
pub(super) fn conversation(model: &mut AppModel, state: &mut ShellState, ws: &Path, agent: &str) {
    model.focus_agent(ws, agent);
    request(state);
}

/// **§11's visible-selection invariant** (bl-fa82): the selected agent's row is
/// painted. Landing the operator on a row they cannot see is the *why am I
/// here* §6 already forbids of a jump's landing, and the gestures that can do
/// it are many — a §8.5 address, a start's adoption, the attention jump, a
/// pointer on any row — so the invariant is kept once, by the list,
/// rather than by a clause at each of their sites.
///
/// Opening the agent's ancestor chain is exactly enough: a visible row's parent
/// is visible by construction. A depth-0 root's chain is empty, so the ordinary
/// case adds nothing. It cannot fight a fold, either, because both collapsing
/// gestures ([`collapse_row`], [`toggle_row`]) carry the selection up to the row
/// they shut — so by the time this runs the selection is never under it.
pub(super) fn reveal_selection(model: &mut AppModel, state: &mut ShellState) {
    state.expanded.extend(ancestors(model));
}

/// The descent-id chain above the **selected** agent, outermost first — read
/// off the §11 seat's own view (REMOTE §9.4, bl-1eb0), which folds the same
/// `nav::convs` derivation the list itself renders. Empty for a root and for an
/// id this snapshot has not got.
fn ancestors(model: &AppModel) -> Vec<String> {
    model
        .focused_conversation()
        .map(|seat| seat.ancestors)
        .unwrap_or_default()
}

/// The list as the frame paints it — the one derivation every gesture below
/// reads, so the walk, the fold and the paint can never disagree about which
/// rows exist (§11, `nav::convs::expand`).
fn visible(model: &AppModel, state: &ShellState) -> Vec<crate::nav::convs::ConvRow> {
    model.visible_conversations(super::now_unix(), &state.expanded)
}

/// The focused agent, owned — the id every unfold gesture acts on.
fn selected(model: &AppModel) -> Option<String> {
    model.focused_agent_id()
}

/// The focused workspace, owned — every selection below is made inside it,
/// since the walk stopped crossing walls with bl-fa82.
fn wall(model: &AppModel) -> Option<PathBuf> {
    model.focused_workspace().map(Path::to_path_buf)
}

/// ↑/↓ — step the selection ±`delta` through the **visible** list rows in paint
/// order and hand the keyboard over (rule 2). A collapsed subtree contributes
/// one row to that list, so the step skips it whole without the walk knowing
/// anything about folding — the ruling's *"don't automatically expand just from
/// going down"* with no branch implementing it. The request is unconditional:
/// with nothing to select, the composer is where the keyboard belongs anyway.
pub(super) fn list_step(model: &mut AppModel, state: &mut ShellState, delta: isize) {
    let rows = visible(model, state);
    let here = selected(model);
    if let (Some(ws), Some(next)) = (
        wall(model),
        crate::nav::convs::step(&rows, here.as_deref(), delta),
    ) {
        model.focus_agent(&ws, &next);
    }
    request(state);
}

/// → — unfold the selected row. Fires no verb and takes no keyboard: it
/// repaints a viewport (§11 rule 3), so the plane it was pressed on is still
/// live under the operator's hand.
pub(super) fn expand_row(model: &AppModel, state: &mut ShellState) {
    if let Some(id) = selected(model) {
        state.expanded.insert(id);
    }
}

/// ← — fold the selected row shut, or, with nothing to fold, page the selection
/// up to its parent row (§11). The two are one gesture and this is the whole of
/// it: a row that *was* open closes, and one that was not walks out a level, so
/// `←` held down leaves a descent the way `↑` walks up a list. Paging is a
/// selection, so it lands the composer like every other; closing a fold is not,
/// and does not.
pub(super) fn collapse_row(model: &mut AppModel, state: &mut ShellState) {
    let Some(id) = selected(model) else { return };
    if state.expanded.remove(&id) {
        return;
    }
    let rows = visible(model, state);
    if let (Some(ws), Some(parent)) = (wall(model), crate::nav::convs::parent_of(&rows, &id)) {
        conversation(model, state, &ws, &parent);
    }
}

/// The subagent field's click: flip one id in the expanded set through
/// [`toggle_path`](crate::jsonview::toggle_path) — the crate's one disclosure
/// toggle, shared with the transcript and queue folds — then keep §11's
/// visible-selection invariant by carrying a selection the fold just hid up to
/// the row that hid it. Reads the ancestor chain, so a selection three
/// generations down is caught by the same one check.
pub(super) fn toggle_row(model: &mut AppModel, state: &mut ShellState, ws: &Path, agent: &str) {
    crate::jsonview::toggle_path(&mut state.expanded, agent);
    let hidden = !state.expanded.contains(agent)
        && selected(model).is_some()
        && ancestors(model).iter().any(|a| a == agent);
    if hidden {
        conversation(model, state, ws, agent);
    }
}

/// Select a workspace and hand the keyboard over — the tab bar, the overflow
/// menu, and `new conversation`, which is the same move with the agent
/// selection cleared (the keyboard's `n` rides here, rule 2).
pub(super) fn workspace(model: &mut AppModel, state: &mut ShellState, ws: &Path) {
    model.focus_workspace(ws);
    request(state);
}
