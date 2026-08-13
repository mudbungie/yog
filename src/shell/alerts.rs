//! **The attention strip reaching a buried window** (§6 as amended, bl-e160) —
//! the shell's whole face of [`crate::alert`], which is one call and one
//! thread.
//!
//! Its own file for §12's budget, and the split is a real seam: everything the
//! decision needs is pure and lives in `crate::alert`; what is left here is the
//! two things only a window can supply — the OS's answer to *do I have focus*,
//! and a thread to spawn the notifier on so the frame never waits for a
//! desktop.

use super::{ShellState, clock};
use crate::AppModel;

/// Fold this frame's §6 queue into what the window has already announced and,
/// when something arrived that the operator has not seen, tell the desktop.
///
/// The fold runs **every** frame, focused or not, so the baseline advances on
/// what the operator is actually looking at; only the announcing is gated
/// ([`crate::alert::announce`]). That is what stops a burst of stale news the
/// moment a window loses focus.
///
/// **The frame never waits on the desktop** (bl-ee0a): the spawn and its wait
/// go to a thread of their own, and the handle is dropped — a notification is
/// output, so nothing downstream depends on its fate. The only cost on the
/// frame thread is the queue derivation, a pure read of the snapshot this
/// frame is already rendering.
pub(super) fn escalate(ctx: &egui::Context, model: &AppModel, state: &mut ShellState) {
    let arrived = crate::alert::announce(
        &mut state.alerts,
        &model.decision_queue(clock::now_unix()),
        ctx.input(|i| i.focused),
        model.notify_unfocused(),
    );
    if arrived.is_empty() {
        return;
    }
    drop(std::thread::spawn(move || {
        crate::alert::send::deliver(std::path::Path::new(crate::alert::send::NOTIFIER), &arrived);
    }));
}
