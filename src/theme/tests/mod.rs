//! Theme tests: the visuals derivation, the apply seam, the pulse animation and
//! the badge mappings. The two mark seats are `theme/mark/tests.rs`, beside the
//! module that paints them.

mod tone;

use super::{
    ASH, BRAZEN, BRAZEN_DIM, GATE, HYDRA, ICHOR, SIGIL, SPECTRE, apply, ball_hue, integration_hue,
    pulse, visuals,
};

#[test]
fn integration_hues_key_the_three_driven_tools() {
    assert_eq!(integration_hue("lernie"), HYDRA);
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
fn verdict_badges_are_distinct_and_name_their_verdict() {
    use crate::monitor::Verdict;
    let mut carriers = std::collections::HashSet::new();
    for verdict in [Verdict::Aligned, Verdict::Drifting, Verdict::Diverged] {
        let (glyph, color, phrase) = super::verdict_badge(verdict);
        assert!(carriers.insert(glyph), "duplicate glyph {glyph:?}");
        assert!(carriers.insert(phrase), "duplicate phrase {phrase:?}");
        assert!(
            phrase.starts_with(verdict.token()),
            "{phrase:?} does not say its verdict"
        );
        assert!(color.to_array()[3] > 0, "an invisible badge says nothing");
    }
    // The quiet one is the calm hue: a monitor that shouts when nothing is
    // wrong teaches the operator to stop reading it.
    assert_eq!(super::verdict_badge(Verdict::Aligned).1, super::MOONLIT);
    assert_eq!(super::verdict_badge(Verdict::Diverged).1, super::ICHOR);
}

#[test]
fn flight_badges_are_distinct_in_all_three_carriers() {
    use crate::nav::convs::Flight;
    // The operator's ask is that the three "look different" — so glyph, hue and
    // words must each differ, not merely the tuple as a whole. A shared carrier
    // would make two classes tell apart only by the one that is left.
    let classes = [Flight::Inference, Flight::Tools, Flight::Subagents];
    let mut glyphs = std::collections::HashSet::new();
    let mut colors = std::collections::HashSet::new();
    let mut phrases = std::collections::HashSet::new();
    for f in classes {
        let (glyph, color, phrase) = super::flight_badge(f);
        assert!(glyphs.insert(glyph), "duplicate glyph {glyph:?} for {f:?}");
        assert!(colors.insert(color.to_array()), "duplicate hue for {f:?}");
        assert!(!phrase.is_empty(), "unsaid class {f:?}");
        assert!(phrases.insert(phrase), "duplicate phrase for {f:?}");
    }
    // The lore mapping: a model call is the spectral in-transit hue, a running
    // tool wears the driver's hydra green, a child wears the other-agent bronze.
    assert_eq!(super::flight_badge(Flight::Inference).1, SPECTRE);
    assert_eq!(super::flight_badge(Flight::Tools).1, HYDRA);
    assert_eq!(super::flight_badge(Flight::Subagents).1, BRAZEN);
}

#[test]
fn state_badges_are_distinct_per_state() {
    use crate::git_tree::AgentState;
    let states = [
        AgentState::Live,
        AgentState::InFlight,
        AgentState::Quiescent,
        AgentState::Stopped,
    ];
    let mut glyphs = std::collections::HashSet::new();
    let mut colors = std::collections::HashSet::new();
    let mut phrases = std::collections::HashSet::new();
    for s in states {
        let (glyph, color, phrase) = super::state_badge(s);
        assert!(glyphs.insert(glyph), "duplicate glyph {glyph:?} for {s:?}");
        assert!(
            colors.insert(color.to_array()),
            "duplicate colour for {s:?}"
        );
        // The glyph doctrine (§11): every state says itself in words, so the
        // badge is never the state's only carrier. A phrase that is empty — or
        // shared with another state — puts the load back on the glyph.
        assert!(!phrase.is_empty(), "unsaid state {s:?}");
        assert!(
            phrases.insert(phrase),
            "duplicate phrase {phrase:?} for {s:?}"
        );
    }
    // The lore mapping itself: live = hydra, streaming = spectre.
    let (glyph, color, phrase) = super::state_badge(AgentState::Live);
    assert_eq!((glyph, color), ("●", HYDRA));
    assert!(phrase.starts_with("live —"));
    let (glyph, color, phrase) = super::state_badge(AgentState::InFlight);
    assert_eq!((glyph, color), ("◐", SPECTRE));
    assert!(phrase.starts_with("in flight —"));
    let (glyph, color, phrase) = super::state_badge(AgentState::Quiescent);
    assert_eq!((glyph, color), ("○", BRAZEN_DIM));
    assert!(phrase.starts_with("quiescent —"));
    let (glyph, color, phrase) = super::state_badge(AgentState::Stopped);
    assert_eq!((glyph, color), ("■", ASH));
    assert!(phrase.starts_with("stopped —"));
    // The §10 sigil's own words, the fourth carrier the seats pair with "?".
    assert!(super::STATE_UNCERTAIN.starts_with("uncertain —"));
    // bl-8068: the legacy-rung name warns it is not a message target, and
    // points at the address that always works.
    assert!(super::NAME_DISPLAY_ONLY.starts_with("display-only name —"));
    assert!(super::NAME_DISPLAY_ONLY.contains("agent id"));
}

#[test]
fn every_agent_mark_says_itself_in_a_label_and_a_sentence() {
    use crate::git_tree::AgentMark;
    let marks = [
        AgentMark::Notified,
        AgentMark::BudgetExhausted,
        AgentMark::Conflicted,
        AgentMark::Held,
        AgentMark::Abandoned,
    ];
    let mut labels = std::collections::HashSet::new();
    let mut phrases = std::collections::HashSet::new();
    for m in marks {
        let (label, _, phrase) = super::mark_badge(m);
        assert!(labels.insert(label), "duplicate label {label:?} for {m:?}");
        // §11 glyph doctrine, and §6's "the fact survives the ack": the mark's
        // seat must be able to say what it means, not merely name it — so the
        // sentence leads with the label and then explains it.
        assert!(
            phrase.starts_with(label) && phrase.len() > label.len(),
            "unexplained mark {m:?}: {phrase:?}"
        );
        assert!(phrases.insert(phrase), "duplicate phrase for {m:?}");
    }
    // The hues: a summons and a ceiling read brazen, a declined transfer is a
    // wound, and the abandonment that *quiets* a stop reads ash like the stop.
    assert_eq!(super::mark_badge(AgentMark::Notified).1, BRAZEN);
    assert_eq!(super::mark_badge(AgentMark::BudgetExhausted).1, BRAZEN);
    assert_eq!(super::mark_badge(AgentMark::Conflicted).1, ICHOR);
    assert_eq!(super::mark_badge(AgentMark::Held).1, BRAZEN);
    assert_eq!(super::mark_badge(AgentMark::Abandoned).1, ASH);
}

#[test]
fn op_badges_say_their_outcome_in_words() {
    use crate::opslog::OpOutcome;
    let mut colors = std::collections::HashSet::new();
    let mut phrases = std::collections::HashSet::new();
    for o in [
        OpOutcome::Clean,
        OpOutcome::Failed,
        OpOutcome::Retired,
        OpOutcome::Detached,
    ] {
        let (glyph, color, phrase) = super::op_badge(o);
        assert!(!glyph.is_empty(), "unmarked outcome {o:?}");
        assert!(
            colors.insert(color.to_array()),
            "duplicate colour for {o:?}"
        );
        // The glyph doctrine (§11): every outcome says itself in words, so the
        // marker is never its only carrier. A phrase that is empty — or shared
        // with another outcome — puts the load back on the glyph.
        assert!(!phrase.is_empty(), "unsaid outcome {o:?}");
        assert!(
            phrases.insert(phrase),
            "duplicate phrase {phrase:?} for {o:?}"
        );
    }
    // The two failure outcomes share ⚠ on purpose (§6 keeps the retired row's
    // mark, dropping only ichor) — so the words, not the glyph, tell them apart.
    let (failed_glyph, failed_hue, failed_phrase) = super::op_badge(OpOutcome::Failed);
    assert_eq!(
        (failed_glyph, failed_hue, failed_phrase),
        ("⚠", ICHOR, "failed")
    );
    let (retired_glyph, retired_hue, retired_phrase) = super::op_badge(OpOutcome::Retired);
    assert_eq!((retired_glyph, retired_hue), ("⚠", ASH));
    assert!(retired_phrase.starts_with("failed, retired by"));
    assert_eq!(
        super::op_badge(OpOutcome::Clean),
        ("·", super::MOONLIT, "ran clean")
    );
    // bl-8433: a handed-off spawn is neither clean nor failed — its own glyph
    // and hue, and its phrase matches `opslog::exit::ExitKind::Detached`'s own
    // wording verbatim, so the badge and the expanded row never disagree.
    let (detached_glyph, detached_hue, detached_phrase) = super::op_badge(OpOutcome::Detached);
    assert_ne!(detached_glyph, "·", "must not read as clean");
    assert_ne!(detached_glyph, "⚠", "must not read as failed");
    assert_eq!(detached_hue, BRAZEN);
    assert_eq!(detached_phrase, "detached — handed off, no exit to observe");
}

#[test]
fn tool_result_badges_say_the_outcome() {
    let ok = super::tool_result_badge(false);
    let err = super::tool_result_badge(true);
    // The lore mapping: an ok result is the liveness green, an error the wound.
    assert_eq!((ok.0, ok.1), ("✔", HYDRA));
    assert_eq!((err.0, err.1), ("✖", ICHOR));
    // The glyph doctrine (§11): both outcomes say themselves in words, so ✔/✖
    // is never the only carrier. A phrase that is empty — or shared with the
    // other outcome — puts the whole load back on the glyph.
    assert!(!ok.2.is_empty(), "unsaid ok outcome");
    assert!(!err.2.is_empty(), "unsaid error outcome");
    assert_ne!(ok.2, err.2, "one phrase for both outcomes says neither");
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
