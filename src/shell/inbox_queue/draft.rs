//! **The queue's last item**: the multiline draft box, split off [`super`] at
//! §12's budget on the seam the region is built from — above the box sit the
//! pending deposits, which are the snapshot's and are only read; the box is the
//! one thing in the region the operator writes, and everything peculiar to it
//! lives here.
//!
//! Three of those peculiarities are the file's whole reason: the ↑/↓ recall
//! keys are taken off the frame's input **before** the widget is added, so a
//! step never also moves the caret (bl-f908); the caret's painted row is read
//! back off the galley, so the next frame's gate reads what was shown rather
//! than the pre-recall cursor; and the widget's own newline is bound to
//! Shift+Enter, which is what leaves a bare Enter whole for the caller's send
//! read (bl-4515).

use crate::actions::Drafts;
use crate::composer::{self, Caret, Step};

use super::QueueCtx;

/// The one box (§11): what Enter in it does, either way the target falls.
const BOX_HINT: &str = "Say what you want done. Enter sends it — as a message to the selected \
     conversation, or as a new conversation when nothing is selected. Shift+Enter \
     inserts a newline instead. A draft starting with `/` is a command instead — \
     type `/` alone to see them all, `//` to say a literal slash. ↑ on the box's \
     top line brings back what you already said here, newest first; ↓ on its \
     bottom line comes forward again, and past the newest hands your draft back.";

/// The queue's last item: the multiline draft box (bl-4515 key contract — the
/// widget newlines only on Shift+Enter, so a plain Enter stays whole for the
/// caller's send read). The buffer is the target's own (bl-a69a), read in and
/// written back every frame; it is RAM until sent (§5.3).
///
/// The recall's two keys (bl-f908) are taken **before** the widget is added,
/// so a step never also moves the caret; a step the caret gate or the history
/// declines is left alone and the arrow does what it always did.
pub(super) fn input_box(
    ui: &mut egui::Ui,
    recall: &mut composer::Recall,
    caret: &mut Caret,
    drafts: &mut Drafts,
    ctx: &QueueCtx,
) -> egui::Response {
    // A **stable** widget id: the box sits after a queue whose row count
    // changes frame to frame, and egui's auto-id would shift with it — moving
    // the keyboard's focus target out from under the operator mid-typing
    // exactly when an item lands.
    let id = egui::Id::new("inbox-composer-box");
    let mut buffer = drafts.text(&ctx.key);
    recall.settle(&buffer, &ctx.prompts);
    let mut stepped = false;
    if ui.memory(|m| m.has_focus(id)) {
        for (key, dir) in [
            (egui::Key::ArrowUp, Step::Back),
            (egui::Key::ArrowDown, Step::Forward),
        ] {
            if !ui.input(|i| i.modifiers.is_none() && i.key_pressed(key)) {
                continue;
            }
            if let Some(text) = recall.step(dir, *caret, &buffer, &ctx.prompts) {
                ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, key));
                buffer = text;
                stepped = true;
            }
        }
    }
    let out = egui::TextEdit::multiline(&mut buffer)
        .id(id)
        .desired_rows(1)
        .desired_width(f32::INFINITY)
        .return_key(egui::KeyboardShortcut::new(
            egui::Modifiers::SHIFT,
            egui::Key::Enter,
        ))
        .hint_text(&ctx.hint)
        .show(ui);
    let rows = out.galley.rows.len();
    *caret = if stepped {
        // A recall parks the caret at the end of what it brought back — the
        // galley just painted IS that text, so the row is known here and the
        // next frame's gate reads it rather than the pre-recall cursor.
        park_at_end(ui, id, out.state, buffer.chars().count());
        Caret {
            row: rows.saturating_sub(1),
            rows,
        }
    } else {
        Caret {
            row: out.cursor_range.map_or(0, |r| r.primary.rcursor.row),
            rows,
        }
    };
    drafts.set(ctx.key.clone(), buffer);
    out.response.on_hover_text(BOX_HINT)
}

/// Seat the caret past the last character of a recalled prompt — the shell
/// idiom for "you are looking at this one now", and what makes a one-row
/// prompt sit on the top row and the bottom row at once (§11, bl-f908).
fn park_at_end(
    ui: &egui::Ui,
    id: egui::Id,
    mut state: egui::text_edit::TextEditState,
    chars: usize,
) {
    state
        .cursor
        .set_char_range(Some(egui::text::CCursorRange::one(
            egui::text::CCursor::new(chars),
        )));
    state.store(ui.ctx(), id);
}
