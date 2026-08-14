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
//! **This seat carries the characteristics, never the class in words**
//! (bl-3f70, QUALITY H1). It used to print `flight_badge`'s whole sentence in
//! front of them — *"◐ inference — a model call is streaming · growing · 320
//! chars streamed · 5s"* — which is the identical run the §11 header paints two
//! lines above, on the same surface, in the same hue. Two things decided which
//! of the two seats keeps the words. The header is **unconditional**: this
//! panel asks the §11 rule 5 budget for its share and paints nothing when the
//! answer is `None` ([`super::pane`]), so a fact seated only here is a fact
//! that disappears at the documented 420x320 minimum. And this line is
//! **width-bound** — it truncates at the pane's edge by its own rule below — so
//! the duplicated prefix was pushing the one thing this seat exists to add off
//! the right-hand end of it. What is left is the §11 badge-seat pattern the
//! width-bound list row already follows: the glyph and its pulse state the
//! class, and the words ride the hover.
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
    // (§11 glyph doctrine). This seat paints the first two and hovers the third
    // (see the module doc): the glyph is never the words' only carrier here —
    // the characteristics beside it are the class stated concretely (`320 chars
    // streamed` is a model call streaming, `Read` is a tool executing, `2
    // children running` is a descent working), the hue is the class's own, and
    // the sentence is one hover away.
    let (glyph, hue, says) = theme::flight_badge(strip.class);
    egui::TopBottomPanel::bottom("flight-strip").show_inside(ui, |ui| {
        // One line, always: a long name truncates at the pane's edge rather
        // than wrapping, which would grow the panel's height frame by frame
        // (§11 rule 1, the same defect on the other axis).
        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);
        let time = ui.ctx().input(|i| i.time);
        ui.colored_label(theme::pulse(hue, time), format!("{glyph} {}", strip.facts))
            .on_hover_text(format!("{says} — {STRIP_HOVER}"));
    });
    ui.ctx().request_repaint_after(theme::PULSE_REPAINT_DELAY);
}
