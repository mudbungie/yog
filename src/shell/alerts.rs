//! **The attention strip reaching a buried window** (§6 as amended, bl-e160) —
//! the shell's whole face of [`crate::alert`], which is one call and one
//! thread.
//!
//! Its own file for §12's budget, and the split is a real seam: everything the
//! decision needs is pure and lives in `crate::alert`; what is left here is the
//! two things only a window can supply — the OS's answer to *do I have focus*,
//! and a thread to spawn the notifier on so the frame never waits for a
//! desktop.

use super::ShellState;
use crate::AppModel;
use crate::boundary::Query;
use crate::boundary::reply::Reply;

/// Fold the §6 queue **the wire answered** into what the window has already
/// announced and, when something arrived that the operator has not seen, tell
/// the desktop.
///
/// The queue is [`Query::Attention`] since bl-f297 — the same list a headless
/// reader takes, standing on the asker like every other migrated read, so the
/// window's escalation and `/attention` cannot describe one moment differently.
///
/// **An unanswered frame is not a reading of the queue** — and that is the rule
/// this migration turned on. The fold used to run on every frame, because every
/// frame held a derivation; a frame the wire has not answered holds nothing, and
/// folding *that* would read as everything having departed and then, on the next
/// answer, as everything arriving at once. So the baseline moves only on a frame
/// that was told something, which is the same rule that makes a freshly-opened
/// window announce nothing: no observation, no arrival. A refusal is not a
/// reading either — it is the engine declining to say, and this seat has no
/// surface to paint that on (a notification is output, not a pane), so the
/// window stays quiet rather than announcing on a guess.
///
/// What that costs is stated rather than discovered: the fold now runs at the
/// asker's cadence rather than the frame's, and the focus gate is read on the
/// frame the answer lands. A window buried and re-focused inside one ask period
/// folds once instead of thirty times, which is a difference no difference
/// detector can feel.
///
/// **The frame never waits on the desktop** (bl-ee0a): the spawn and its wait
/// go to a thread of their own, and the handle is dropped — a notification is
/// output, so nothing downstream depends on its fate.
pub(super) fn escalate(ctx: &egui::Context, model: &mut AppModel, state: &mut ShellState) {
    let landed = super::wire::ask(model, Query::Attention, |reply| match reply {
        Reply::Attention(rows) => Some(rows),
        _ => None,
    });
    let Some(rows) = landed.value else {
        return;
    };
    let arrived = crate::alert::announce(
        &mut state.alerts,
        &rows,
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
