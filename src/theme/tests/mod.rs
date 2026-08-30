//! Theme tests: the palette itself — the lore hues, their pairwise distinctness,
//! the derived ball hue — plus the visuals derivation, the apply seam and the
//! pulse animation. [`badges`] holds the §11 badge mappings' half, split on the
//! same seam the production modules are (`theme/badges.rs` against the palette
//! and `theme/visuals.rs`); the two mark seats are `theme/mark/tests.rs`,
//! beside the module that paints them.

mod badges;
mod tone;

use super::{
    ASH, BRAZEN, BRAZEN_DIM, GATE, HYDRA, ICHOR, SIGIL, SPECTRE, apply, ball_hue, integration_hue,
    pulse, visuals,
};

#[test]
fn integration_hues_key_the_three_driven_tools() {
    assert_eq!(integration_hue("litany"), HYDRA);
    assert_eq!(integration_hue("bz"), BRAZEN);
    assert_eq!(integration_hue("bl"), GATE);
    // An unknown tool reads in the moonlit text default, not a lore hue.
    assert_eq!(integration_hue("cthulhu"), super::MOONLIT);
}

#[test]
fn palette_hues_are_pairwise_distinct() {
    // The badge/mark vocabulary only works if no two hues collapse — a
    // regression here would make e.g. "quiescent" read as "pending".
    let hues = [HYDRA, SPECTRE, BRAZEN, BRAZEN_DIM, ICHOR, ASH, SIGIL, GATE];
    for (i, a) in hues.iter().enumerate() {
        for b in hues.iter().skip(i + 1) {
            assert_ne!(a, b);
        }
    }
}

#[test]
fn ball_hues_key_the_derived_join_status() {
    use crate::projects::join::JoinState;
    // The lore mapping: a bound ball is the healthy working hue, delivered has
    // gone quiet, blocked/claimed-elsewhere want a look, an orphan is a wound.
    assert_eq!(ball_hue(JoinState::Bound), HYDRA);
    assert_eq!(ball_hue(JoinState::Delivered), ASH);
    assert_eq!(ball_hue(JoinState::Blocked), BRAZEN);
    assert_eq!(ball_hue(JoinState::ClaimedElsewhere), BRAZEN);
    assert_eq!(ball_hue(JoinState::OrphanedProject), ICHOR);
    // States that never head a conversation ball read in the neutral default.
    assert_eq!(ball_hue(JoinState::ReadyStartable), super::MOONLIT);
    assert_eq!(ball_hue(JoinState::UnassignedWorkspace), super::MOONLIT);
}

#[test]
fn visuals_reground_the_dark_theme_in_the_void() {
    let v = visuals();
    assert!(v.dark_mode);
    // The void strata: window < panel, extreme deepest, faint the lift.
    assert_eq!(v.panel_fill, egui::Color32::from_rgb(19, 15, 27));
    assert_eq!(v.window_fill, egui::Color32::from_rgb(15, 12, 22));
    assert_eq!(v.extreme_bg_color, egui::Color32::from_rgb(10, 8, 15));
    assert_eq!(v.code_bg_color, v.extreme_bg_color);
    assert_eq!(v.faint_bg_color, egui::Color32::from_rgb(30, 24, 43));
    // Gate-violet selection; semantic hues on the egui-native slots.
    assert_eq!(v.selection.stroke.color, GATE);
    assert_eq!(v.selection.bg_fill, GATE.gamma_multiply(0.35));
    assert_eq!(v.hyperlink_color, SPECTRE);
    assert_eq!(v.warn_fg_color, BRAZEN);
    assert_eq!(v.error_fg_color, ICHOR);
    // The moonlit text ramp brightens with interactivity.
    let base = v.widgets.noninteractive.fg_stroke.color;
    let hover = v.widgets.hovered.fg_stroke.color;
    assert_eq!(base, egui::Color32::from_rgb(198, 190, 216));
    assert_eq!(hover, egui::Color32::from_rgb(240, 234, 250));
    assert_eq!(v.widgets.active.bg_stroke.color, GATE);
}

#[test]
fn apply_installs_the_visuals_on_a_context() {
    let ctx = egui::Context::default();
    apply(&ctx);
    assert_eq!(ctx.style().visuals.panel_fill, visuals().panel_fill);
    assert_eq!(ctx.style().visuals.selection.stroke.color, GATE);
}

#[test]
fn pulse_breathes_its_hue_between_dim_and_full() {
    use std::f64::consts::PI;
    // sin(4t) = 1 at t = π/8 (full); sin(4t) = -1 at t = 3π/8 (dim).
    let full = pulse(SPECTRE, PI / 8.0);
    let dim = pulse(SPECTRE, 3.0 * PI / 8.0);
    assert_eq!(full, SPECTRE.gamma_multiply(1.0));
    assert_eq!(dim, SPECTRE.gamma_multiply(0.35));
    assert!(dim.r() < full.r());
    // The hue is the caller's — one animation, three §11 classes reading apart.
    assert_eq!(pulse(HYDRA, PI / 8.0), HYDRA.gamma_multiply(1.0));
    assert_ne!(pulse(HYDRA, PI / 8.0), full);
}
