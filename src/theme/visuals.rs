//! The congeries **installed** — the palette (`super`) derived into the egui
//! context yog actually paints through.
//!
//! Split from the palette at §12's budget on the seam the two already had: the
//! parent names every hue once, and this is the one place those hues become an
//! [`egui::Visuals`], a [`egui::FontDefinitions`] and the bring-up seam that
//! seats both. Nothing here mints a colour; nothing there touches a context.

use super::{
    BRAZEN, GATE, ICHOR, MOONLIT, MOONLIT_BRIGHT, MOONLIT_FULL, SPECTRE, VOID_DEEP, VOID_EDGE,
    VOID_FAINT, VOID_PANEL, VOID_WINDOW,
};

/// The whole-app [`egui::Visuals`]: egui's dark theme re-grounded in the
/// void strata, with gate-violet selection and the moonlit text ramp. Pure —
/// testable without a window; [`apply`] installs it.
pub fn visuals() -> egui::Visuals {
    let mut v = egui::Visuals::dark();
    v.panel_fill = VOID_PANEL;
    v.window_fill = VOID_WINDOW;
    v.extreme_bg_color = VOID_DEEP;
    v.code_bg_color = VOID_DEEP;
    v.faint_bg_color = VOID_FAINT;
    v.window_stroke = egui::Stroke::new(1.0, VOID_EDGE);
    v.selection.bg_fill = GATE.gamma_multiply(0.35);
    v.selection.stroke = egui::Stroke::new(1.0, GATE);
    v.hyperlink_color = SPECTRE;
    v.warn_fg_color = BRAZEN;
    v.error_fg_color = ICHOR;
    v.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(24, 19, 34);
    v.widgets.noninteractive.weak_bg_fill = VOID_FAINT;
    v.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, VOID_EDGE);
    v.widgets.noninteractive.fg_stroke.color = MOONLIT;
    v.widgets.inactive.bg_fill = egui::Color32::from_rgb(36, 29, 52);
    v.widgets.inactive.weak_bg_fill = egui::Color32::from_rgb(28, 22, 40);
    v.widgets.inactive.fg_stroke.color = MOONLIT_BRIGHT;
    v.widgets.hovered.bg_fill = egui::Color32::from_rgb(48, 38, 70);
    v.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, GATE.gamma_multiply(0.6));
    v.widgets.hovered.fg_stroke.color = MOONLIT_FULL;
    v.widgets.active.bg_fill = egui::Color32::from_rgb(60, 47, 88);
    v.widgets.active.bg_stroke = egui::Stroke::new(1.0, GATE);
    v.widgets.active.fg_stroke.color = MOONLIT_FULL;
    v.widgets.open.bg_fill = egui::Color32::from_rgb(36, 29, 52);
    v.widgets.open.fg_stroke.color = MOONLIT_BRIGHT;
    v
}

/// The font families yog paints with: egui's bundled defaults, with the
/// **Monospace font list folded into Proportional** as a fallback tail.
///
/// egui ships Proportional as `[Ubuntu-Light, NotoEmoji, emoji-icon-font]` and
/// Monospace as the same plus `Hack` at the head — so Hack's box-drawing and
/// geometric coverage (`●◐◈⋯▼→⇒`) was reachable only in mono seats, and every
/// proportional seat painting one of those got a tofu box. Folding mono's list
/// in gives the two families the same font *set*, differing only in priority
/// (the proportional face still leads Proportional, Hack still leads
/// Monospace). Coverage is then identical by construction: no seat has to know
/// which family will paint its glyph, and the `tests/glyph_coverage.rs` guard
/// is one invariant over both families rather than a per-site attribution.
/// Derived from the mono list rather than naming "Hack", so an egui bump that
/// adds a mono fallback is picked up without an edit here.
pub fn fonts() -> egui::FontDefinitions {
    let mut defs = egui::FontDefinitions::default();
    let mono = defs
        .families
        .get(&egui::FontFamily::Monospace)
        .cloned()
        .unwrap_or_default();
    if let Some(prop) = defs.families.get_mut(&egui::FontFamily::Proportional) {
        for name in mono {
            if !prop.contains(&name) {
                prop.push(name);
            }
        }
    }
    defs
}

/// Install the congeries fonts and visuals on an egui context — called once
/// from the eframe creation closure (main.rs, coverage-excluded), so theming
/// rides two covered pure functions plus this seam.
pub fn apply(ctx: &egui::Context) {
    ctx.set_fonts(fonts());
    ctx.set_visuals(visuals());
    // egui's built-in Ctrl+±/0 handler would be a second, RAM-only authority
    // for text size — lost at exit and free to drift from the durable §4.1
    // `zoom`. yog binds those keys itself (§11) and derives the live factor
    // from `ui.json` every frame, so the context is a projection, never a home.
    ctx.options_mut(|o| o.zoom_with_keyboard = false);
}
