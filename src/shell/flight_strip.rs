//! The §11 bottom in-flight strip (bl-905f): one pulsing line at the bottom
//! of the conversation pane saying what the open conversation is doing right
//! now.
//!
//! **Why a third seat for one fact.** The chat is tail-anchored (bl-5cdb), so
//! the operator's eyes live at the bottom of the screen; the altitude-1
//! header's chip — the same class off the same derivation — is out of view
//! while they read the tail. The inference status at the top is right as far as
//! it goes, but an in-flight call's characteristics belong at the bottom of the
//! screen while it runs, so that an operator looking down at the chat sees that
//! it is working.
//!
//! Coverage-excluded glue: the strip's whole content is
//! [`AppModel::flight_strip`] and [`theme::flight_badge`], both tested — this
//! file only chooses the seat and asks for the pulse.

use crate::AppModel;
use crate::nav::convs::STRIP_HOVER;
use crate::theme;

/// Paint the strip, if anything is in flight. **The panel itself is
/// conditional**, not its content: an idle conversation costs no strip, no
/// pixel row, and no repaint (§7.2 — `None` is one decision at one site).
///
/// The seat is the **innermost** bottom panel of the conversation pane: hard
/// against the chat tail, above whichever goal box holds the composer's seat
/// and above the settings rows below it (§11 bottom accessories, bl-c038,
/// bl-2e18 as amended by bl-58e4 — the band-order ruling moved the settings
/// rows to the far side of the input box and did **not** touch this seat). The
/// strip is a fact about the open conversation and belongs inside its pane,
/// while the window-level activity accessory below is world-level ops chrome.
/// Called last of the pane's bottom stack for exactly that stacking, with its
/// §11 rule 5 share already checked by the stack that owns the arithmetic
/// ([`super::pane`]).
pub(super) fn render(ui: &mut egui::Ui, model: &AppModel) {
    let Some(strip) = model.flight_strip(super::now_unix()) else {
        return;
    };
    // Glyph, hue and the class said in words all come from the one badge home
    // (§11 glyph doctrine); this seat has a full pane row, so — like the
    // altitude-1 header and unlike the width-bound list row — it states them
    // outright and hovers only what the strip *is*.
    let (glyph, hue, says) = theme::flight_badge(strip.class);
    egui::TopBottomPanel::bottom("flight-strip").show_inside(ui, |ui| {
        // One line, always: a long name truncates at the pane's edge rather
        // than wrapping, which would grow the panel's height frame by frame
        // (§11 rule 1, the same defect on the other axis).
        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);
        let time = ui.ctx().input(|i| i.time);
        ui.colored_label(
            theme::pulse(hue, time),
            format!("{glyph} {says} · {}", strip.facts),
        )
        .on_hover_text(STRIP_HOVER);
    });
    ui.ctx().request_repaint_after(theme::PULSE_REPAINT_DELAY);
}
