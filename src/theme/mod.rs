//! The congeries palette — yog's single colour authority (DESIGN §11).
//!
//! Yog-Sothoth manifests as *a congeries of iridescent globes* — and yog's
//! visual identity is exactly that: luminous sphere-hues against the void.
//! Every colour the UI paints is named here, once; renderers import the hue,
//! never restate an RGB triple. The naming keys each hue to the platform's
//! lore so the vocabulary is stable across the tools yog drives: **lernie**
//! (the hydra) owns the liveness green, **brazen** (the brazen head) owns the
//! oracle bronze, and yog itself — the key and the gate — owns the violet.
//!
//! [`icon`] is the same congeries as the application mark — one orb table
//! rendered two ways: rasterized for the eframe window icon, emitted as the
//! checked-in `assets/yog.svg` for the desktop entry.
//!
//! [`visuals`] derives the whole egui [`egui::Visuals`] (void backgrounds,
//! gate-violet selection, moonlit text), [`fonts`] the glyph coverage both
//! families share, and [`apply`] installs both at eframe bring-up; [`pulse`]
//! is the shared in-flight animation colour; [`wordmark`] is the congeries
//! mark both wordmark seats render.
//!
//! The `badges` submodule holds the §11 badge mappings — glyph + hue +
//! **words** for one fact each ([`state_badge`], [`mark_badge`], [`op_badge`],
//! [`tool_result_badge`]) — re-exported here so a seat imports one module.

pub mod icon;

mod mark;
pub use mark::{WORDMARK_LEAD, live_mark, wordmark};

mod badges;

// The §11 badge vocabulary, split from this file at §12's cap but addressed
// through it: `theme::state_badge` and its siblings stay one import, because the
// palette and the words a badge says are one authority (§11).
pub use badges::{
    ELBOW, NAME_DISPLAY_ONLY, ROW_ATTENTION, STATE_UNCERTAIN, doing_badge, flight_badge,
    mark_badge, op_badge, state_badge, tool_result_badge, verdict_badge,
};

mod role;

// The §11 role stripe (bl-3acb): the one role vocabulary and its one
// hue-and-words mapping, worn by the transcript rows and the inbox-composer's
// pending queue alike — addressed through the palette for the same reason the
// badges are.
pub use role::{Role, message_role, role_badge, role_stripe};

/// Hydra green — lernie's liveness hue: the `Live` state badge, ✔ ok tool
/// results, `Complete` framing, a settled login outcome.
pub const HYDRA: egui::Color32 = egui::Color32::from_rgb(110, 222, 148);

/// Spectral blue — light mid-transit through the gate: the `InFlight` badge,
/// the live streaming tail, the pulsing tool chip, hyperlinks.
pub const SPECTRE: egui::Color32 = egui::Color32::from_rgb(118, 188, 242);

/// Brazen bronze — the oracle head's metal: pending-mail ✉, warnings,
/// budget-exhausted marks, attention counts.
pub const BRAZEN: egui::Color32 = egui::Color32::from_rgb(232, 176, 96);

/// Tarnished brazen — the `Quiescent` (finished-for-now) badge: still warm,
/// no longer bright.
pub const BRAZEN_DIM: egui::Color32 = egui::Color32::from_rgb(184, 152, 104);

/// Ichor red — the wound hue: ✖ error results, `Failed` framing,
/// declined-transfer marks, fatal ops rows.
pub const ICHOR: egui::Color32 = egui::Color32::from_rgb(242, 108, 120);

/// Ash grey — `Stopped` / `Killed`: nothing behind the gate.
pub const ASH: egui::Color32 = egui::Color32::from_rgb(150, 150, 166);

/// Sigil magenta — the "?" uncertainty suffix (probe-degraded, DESIGN
/// §10/§11): never blends into a definite state's hue.
pub const SIGIL: egui::Color32 = egui::Color32::from_rgb(230, 118, 214);

/// Gate violet — yog's own hue: selection, focus, the wordmark. "It is the
/// key and the gate."
pub const GATE: egui::Color32 = egui::Color32::from_rgb(160, 112, 240);

/// The tagline both wordmark seats may set beneath the mark.
pub const TAGLINE: &str = "the key and the gate";

/// The modal scrim — the void laid over everything a modal has made inert
/// (§11, bl-d921). Translucent, not opaque: what the dialog is about stays
/// legible behind it, which is the whole reason a modal floats rather than
/// replaces. It is the *visible* half of a fact the hit test already enforces.
pub const SCRIM: egui::Color32 = egui::Color32::from_rgba_premultiplied(6, 4, 10, 150);

// The void — background strata, darkest at the deepest inset. Private: only
// `visuals()` composes backgrounds; renderers never paint their own.
const VOID_WINDOW: egui::Color32 = egui::Color32::from_rgb(15, 12, 22);
const VOID_PANEL: egui::Color32 = egui::Color32::from_rgb(19, 15, 27);
const VOID_DEEP: egui::Color32 = egui::Color32::from_rgb(10, 8, 15);
const VOID_FAINT: egui::Color32 = egui::Color32::from_rgb(30, 24, 43);
const VOID_EDGE: egui::Color32 = egui::Color32::from_rgb(54, 43, 78);

// Moonlit text ramp — lavender-cast greys, brightening with interactivity.
const MOONLIT: egui::Color32 = egui::Color32::from_rgb(198, 190, 216);
const MOONLIT_BRIGHT: egui::Color32 = egui::Color32::from_rgb(216, 208, 234);
const MOONLIT_FULL: egui::Color32 = egui::Color32::from_rgb(240, 234, 250);

/// Repaint cadence for pulsing in-flight indicators — ~30 fps, smooth enough
/// for the eye and cheap enough for an idle UI. One cadence, so the whole UI
/// pulses in step.
pub const PULSE_REPAINT_DELAY: std::time::Duration = std::time::Duration::from_millis(33);

/// Pulse frequency in radians per second — ~0.6 Hz, slow enough to read as
/// "alive" rather than "blinking error".
const PULSE_RATE_RAD_PER_SEC: f64 = 4.0;

/// The in-flight pulse colour for `base` at animation-clock `time` (seconds):
/// the hue breathing between dim and full. One animation and one cadence for
/// every pulsing indicator (tool rows, transcript chips, the §11 live-activity
/// classes) so they beat as one; the *hue* is the caller's, which is how the
/// three classes read apart while sharing the beat.
pub fn pulse(base: egui::Color32, time: f64) -> egui::Color32 {
    let alpha = (0.5 + 0.5 * (time * PULSE_RATE_RAD_PER_SEC).sin()).clamp(0.0, 1.0);
    base.gamma_multiply(0.35 + 0.65 * alpha as f32)
}

/// How solid a row painted in `tone` is (bl-915e) — the palette's answer for a
/// statement that is not yet a statement. A send is shown in-memory in faded
/// colour, brightening when it is actually locked in as a statement.
///
/// One number, applied to the whole row rather than a second hue per element:
/// what is provisional about a §7.2 pending echo is not *which* colour it wears
/// — it wears exactly the colours it will wear once it lands — but how much of
/// it there is. So no seat gains a parallel palette, and brightening is the
/// same row at full strength rather than a repaint into different hues.
/// [`Tone::Weak`](crate::transcript::Tone::Weak) is the existing vocabulary's
/// word for that, and every other tone is a fact the derivation already
/// asserts, so it paints solid.
pub fn tone_solidity(tone: crate::transcript::Tone) -> f32 {
    match tone {
        crate::transcript::Tone::Weak => 0.55,
        _ => 1.0,
    }
}

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

/// The hue of a driven integration, keyed by its CLI name: `lernie` the
/// hydra (green), `bz` the brazen head (bronze), `bl` the balls — the
/// congeries itself, so it wears the gate's violet. Anything else reads in
/// the moonlit text default. The one authoritative home for the tool → hue
/// branding (config-editor headings).
pub fn integration_hue(tool: &str) -> egui::Color32 {
    match tool {
        "lernie" => HYDRA,
        "bz" => BRAZEN,
        "bl" => GATE,
        _ => MOONLIT,
    }
}

/// The hue of a conversation's start-flow ball badge, keyed by its §3.5 join
/// state (DESIGN §3.5, §11): the derived status coloured at a glance. `Bound` is
/// the healthy working ball (hydra green); `Delivered` has gone quiet (ash);
/// `Blocked`/`ClaimedElsewhere` want a look (brazen); an `OrphanedProject` ball
/// is a wound (ichor). The startable/unassigned states never head a conversation
/// ball, so they read in the neutral moonlit default. One home for the mapping;
/// a stamp the join could not resolve (`None` state) the caller paints moonlit.
pub fn ball_hue(state: crate::projects::join::JoinState) -> egui::Color32 {
    use crate::projects::join::JoinState;
    match state {
        JoinState::Bound => HYDRA,
        JoinState::Delivered => ASH,
        JoinState::Blocked | JoinState::ClaimedElsewhere => BRAZEN,
        JoinState::OrphanedProject => ICHOR,
        JoinState::ReadyStartable | JoinState::UnassignedWorkspace => MOONLIT,
    }
}

#[cfg(test)]
mod tests;
