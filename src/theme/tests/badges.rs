//! The §11 badge mappings' tests — every one of them a claim that a fact
//! reaches the operator in **all** its carriers: the glyph, the hue, and the
//! words the glyph doctrine forbids it from shipping without. Each mapping is
//! total over its enum, so these walk the enum rather than sampling it.
//!
//! Split from [`super`] at §12's budget on the seam the production side is cut
//! on: `theme/badges.rs` (and `badges/flight.rs`, `badges/op.rs`) against the
//! palette and its egui derivation, which is what the parent asserts.

use crate::theme::{ASH, BRAZEN, BRAZEN_DIM, HYDRA, ICHOR, SPECTRE};

#[test]
fn verdict_badges_are_distinct_and_name_their_verdict() {
    use crate::monitor::Verdict;
    let mut carriers = std::collections::HashSet::new();
    for verdict in [Verdict::Aligned, Verdict::Drifting, Verdict::Diverged] {
        let (glyph, color, phrase) = crate::theme::verdict_badge(verdict);
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
    assert_eq!(
        crate::theme::verdict_badge(Verdict::Aligned).1,
        crate::theme::MOONLIT
    );
    assert_eq!(
        crate::theme::verdict_badge(Verdict::Diverged).1,
        crate::theme::ICHOR
    );
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
        let (glyph, color, phrase) = crate::theme::flight_badge(f);
        assert!(glyphs.insert(glyph), "duplicate glyph {glyph:?} for {f:?}");
        assert!(colors.insert(color.to_array()), "duplicate hue for {f:?}");
        assert!(!phrase.is_empty(), "unsaid class {f:?}");
        assert!(phrases.insert(phrase), "duplicate phrase for {f:?}");
    }
    // The lore mapping: a model call is the spectral in-transit hue, a running
    // tool wears the driver's hydra green, a child wears the other-agent bronze.
    assert_eq!(crate::theme::flight_badge(Flight::Inference).1, SPECTRE);
    assert_eq!(crate::theme::flight_badge(Flight::Tools).1, HYDRA);
    assert_eq!(crate::theme::flight_badge(Flight::Subagents).1, BRAZEN);
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
        let (glyph, color, phrase) = crate::theme::state_badge(s);
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
    let (glyph, color, phrase) = crate::theme::state_badge(AgentState::Live);
    assert_eq!((glyph, color), ("●", HYDRA));
    assert!(phrase.starts_with("live —"));
    let (glyph, color, phrase) = crate::theme::state_badge(AgentState::InFlight);
    assert_eq!((glyph, color), ("◐", SPECTRE));
    assert!(phrase.starts_with("in flight —"));
    let (glyph, color, phrase) = crate::theme::state_badge(AgentState::Quiescent);
    assert_eq!((glyph, color), ("○", BRAZEN_DIM));
    assert!(phrase.starts_with("quiescent —"));
    let (glyph, color, phrase) = crate::theme::state_badge(AgentState::Stopped);
    assert_eq!((glyph, color), ("■", ASH));
    assert!(phrase.starts_with("stopped —"));
    // The §10 sigil's own words, the fourth carrier the seats pair with "?".
    assert!(crate::theme::STATE_UNCERTAIN.starts_with("uncertain —"));
    // bl-8068: the legacy-rung name warns it is not a message target, and
    // points at the address that always works.
    assert!(crate::theme::NAME_DISPLAY_ONLY.starts_with("display-only name —"));
    assert!(crate::theme::NAME_DISPLAY_ONLY.contains("agent id"));
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
        let (label, _, phrase) = crate::theme::mark_badge(m);
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
    assert_eq!(crate::theme::mark_badge(AgentMark::Notified).1, BRAZEN);
    assert_eq!(
        crate::theme::mark_badge(AgentMark::BudgetExhausted).1,
        BRAZEN
    );
    assert_eq!(crate::theme::mark_badge(AgentMark::Conflicted).1, ICHOR);
    assert_eq!(crate::theme::mark_badge(AgentMark::Held).1, BRAZEN);
    assert_eq!(crate::theme::mark_badge(AgentMark::Abandoned).1, ASH);
}

/// The two marks a **live** branch wears must each name the act that moves it
/// on, and name it in the words the control itself wears — a remedy the
/// operator has to translate is a remedy they have to guess at (bl-d710).
///
/// Held has said *approve or decline it* since it shipped; budget-exhausted
/// said only what had happened, which is the wound this ball closes. The
/// sentence is checked against [`RETARGET_EXIT`] rather than against a copy of
/// its text, so relabelling the button that lifts the ceiling fails here
/// instead of leaving the wound pointing at a control by a name nothing wears.
#[test]
fn a_live_mark_names_the_act_that_moves_it_on() {
    use crate::git_tree::AgentMark;
    use crate::model_pick::RETARGET_EXIT;
    let budget = crate::theme::mark_badge(AgentMark::BudgetExhausted).2;
    assert!(
        budget.contains(RETARGET_EXIT),
        "the budget wound names no way out: {budget:?}"
    );
    // And says what the ceiling IS, since the way out only works because the
    // figure is re-derived per call against the frozen config rather than
    // banked against the branch (lernie ARCH §6).
    assert!(budget.contains("never stored"), "{budget:?}");
    assert!(
        crate::theme::mark_badge(AgentMark::Held)
            .2
            .contains("decline")
    );
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
        let (glyph, color, phrase) = crate::theme::op_badge(o);
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
    let (failed_glyph, failed_hue, failed_phrase) = crate::theme::op_badge(OpOutcome::Failed);
    assert_eq!(
        (failed_glyph, failed_hue, failed_phrase),
        ("⚠", ICHOR, "failed")
    );
    let (retired_glyph, retired_hue, retired_phrase) = crate::theme::op_badge(OpOutcome::Retired);
    assert_eq!((retired_glyph, retired_hue), ("⚠", ASH));
    assert!(retired_phrase.starts_with("failed, retired by"));
    assert_eq!(
        crate::theme::op_badge(OpOutcome::Clean),
        ("·", crate::theme::MOONLIT, "ran clean")
    );
    // bl-8433: a handed-off spawn is neither clean nor failed — its own glyph
    // and hue, and its phrase matches `opslog::exit::ExitKind::Detached`'s own
    // wording verbatim, so the badge and the expanded row never disagree.
    let (detached_glyph, detached_hue, detached_phrase) =
        crate::theme::op_badge(OpOutcome::Detached);
    assert_ne!(detached_glyph, "·", "must not read as clean");
    assert_ne!(detached_glyph, "⚠", "must not read as failed");
    assert_eq!(detached_hue, BRAZEN);
    assert_eq!(detached_phrase, "detached — handed off, no exit to observe");
    // bl-b95e: the vocabulary is four words again. bl-1296's fifth — a dimmed
    // handoff for a driver whose sink held only benign lines — went with the
    // phrase table it existed to serve: such a driver is now an ordinary
    // handoff, because its sink is never folded in at all.
}

#[test]
fn tool_result_badges_say_the_outcome() {
    let ok = crate::theme::tool_result_badge(false);
    let err = crate::theme::tool_result_badge(true);
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
