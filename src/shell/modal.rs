//! **A modal owns the frame while it is up** (DESIGN §11, bl-d921) — the one
//! home of that invariant for yog's modals: §3.1's `new workspace` name form
//! and the two §3.6 delete confirmations (workspace, and one conversation
//! deep, bl-f17a).
//!
//! Coverage-excluded glue like the rest of `shell/*`. The two halves it wires
//! are tested elsewhere: the keyboard half is the pure
//! [`Held::Modal`](crate::keymap::Held::Modal) plane in `src/keymap`, and both
//! halves are driven end to end by the acceptance harness
//! (`shell::acceptance::modal`).
//!
//! **The pointer half is a hit test, not a picture.** egui 0.29 has no
//! `egui::Modal`, so the backdrop is a screen-sized [`egui::Area`] in
//! `Order::Middle` — above the panels' `Order::Background`, below the dialog
//! windows that are shown after it. egui's hit test picks the *topmost layer*
//! under the pointer and discards every widget below it, so a click at the left
//! panel's Config entry lands on the backdrop's layer and reaches nothing.
//! `interactable(false)` is load-bearing twice over: it makes the backdrop's
//! own widget sense hover rather than click (so the click is swallowed, not
//! delivered), and it keeps the backdrop out of `Areas::layer_id_at` — which is
//! what stops a press on it from calling `move_to_top` and hoisting the
//! backdrop *above* the dialog it is meant to sit under.

use crate::theme;

use super::ShellState;

/// Is a modal up? The §11 keyboard plane and the backdrop read this one
/// predicate, so the two halves of "owns the frame" can never disagree.
pub(super) fn open(state: &ShellState) -> bool {
    state.new_ws.open || state.delete.target.is_some() || state.delete_agent.target.is_some()
}

/// Dismiss whichever modal owns the frame — Escape's whole meaning on the
/// [`Held::Modal`](crate::keymap::Held::Modal) plane. The draft dies with it
/// (§5.3: unsubmitted input is RAM, and this is the operator saying no), and
/// the keyboard goes back to the composer (§11 focus discipline). Written as
/// both modals' default rather than a branch on which one is up: there is no
/// state to preserve either way, so "clear the transients" is the whole verb.
pub(super) fn dismiss(state: &mut ShellState) {
    state.new_ws = super::NewWsState::default();
    state.delete = super::DeleteState::default();
    state.delete_agent = super::DeleteAgentState::default();
    super::focus::request(state);
}

/// The scrim that makes everything beneath a modal inert — painted only while
/// one is up, immediately before the dialogs so it lands under them.
pub(super) fn backdrop(ctx: &egui::Context, state: &ShellState) {
    if !open(state) {
        return;
    }
    let screen = ctx.screen_rect();
    egui::Area::new(egui::Id::new("modal-backdrop"))
        .order(egui::Order::Middle)
        .interactable(false)
        .fixed_pos(screen.min)
        // The sizing pass has no content yet, so the *first* frame's interact
        // rect comes from here — without it the frame a modal opens on would be
        // the one frame that still leaks clicks through.
        .default_size(screen.size())
        .show(ctx, |ui| {
            ui.set_min_size(screen.size());
            ui.painter().rect_filled(screen, 0.0, theme::SCRIM);
        });
}
