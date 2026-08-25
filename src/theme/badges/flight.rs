//! The two §11 **live-activity** badges — what is in flight in a conversation
//! right now, keyed by [`crate::nav::convs`]'s two liveness vocabularies.
//!
//! Split from [`super`] at §12's cap on the seam `op` was split on: the parent
//! is the vocabulary of what a thing *is* — an agent's state, a `refs/lernie/*`
//! mark, a verdict, a tool's outcome — and these two are the vocabulary of what
//! it is *doing* this instant, read off the live snapshot and repainted every
//! frame. Both mappings are total over their enum, so a new class cannot ship
//! wordless; both pulse through the one shared [`super::super::pulse`].

use super::super::{BRAZEN, HYDRA, ICHOR, SIGIL, SPECTRE};

/// Glyph + colour + **the class said in words** for each §11 live-activity
/// class ([`crate::nav::convs::Flight`]) — what is in flight in a conversation,
/// worn by both indicator seats (the conversation row's pulsing name, the
/// altitude-1 header's chip).
///
/// The three must read **apart at a glance**, so each differs in all three
/// carriers at once, and each hue is the one this palette already keys to that
/// fact — nothing new is minted: `◐` spectral blue is the `InFlight` state's own
/// glyph and hue (a model call streaming); `⚙` hydra green is the tool glyph in
/// the hue of `Live`, which is exactly the state an agent is in while its tool
/// runs; `↳` brazen bronze is the descent arrow in the hue yog already wears for
/// *another agent's* doing (pending mail, attention counts). Each pulses in its
/// own hue through the one shared [`super::super::pulse`] animation, so they beat
/// together and still say three different things.
pub fn flight_badge(
    flight: crate::nav::convs::Flight,
) -> (&'static str, egui::Color32, &'static str) {
    use crate::nav::convs::Flight;
    match flight {
        Flight::Inference => ("◐", SPECTRE, "inference — a model call is streaming"),
        Flight::Tools => ("⚙", HYDRA, "tools — a tool call is executing"),
        Flight::Subagents => ("↳", BRAZEN, "subagents — a dispatched child is running"),
    }
}

/// Hue + **what the agent is doing said in words** for each §5.1 #28b
/// per-agent state — the one home of the §11 live mark's vocabulary.
///
/// **A pair, not the usual triple, and that is the seat's shape rather than an
/// exemption.** The glyph doctrine forbids a glyph from being a fact's only
/// carrier; on the mark the carrier *is* the circle, and these words are what
/// backs it — the mark hovers a roster naming every seat and saying what it is
/// doing. Minting five glyphs for a surface that paints none would add five
/// tofu risks to say nothing new.
///
/// **The set is chosen for legibility at 3 px, not for its names.** Every hue
/// here is driven through `icon::deep` onto a node circle about three pixels
/// across, where hue angle and brightness are the only channels that survive —
/// so the five are picked to be maximally separable *against each other*, and
/// two of them are borrowed from facts they name elsewhere. Ichor is the wound
/// hue everywhere else; sigil is the §10 uncertainty suffix. Nothing on the
/// mark is ever an error or an uncertainty, so neither reuse can be ambiguous
/// at this seat, and minting two more palette entries to dodge a collision that
/// cannot occur would cost the palette its one-hue-per-fact discipline instead.
///
/// **What the spread costs, measured** (bl-c16f, amending bl-b768's set). Thinking wore gate violet and moved to sigil: violet is the
/// *dimmest* hue in the palette (luminance 67 against a void of 17) and it is
/// the wordmark's own hue, painted two pixels to the mark's right — so the
/// state the operator most wanted to see was both the hardest to see and
/// indistinguishable from brand furniture. Tools then kept ichor rather than
/// taking the freed violet, on the whole-set numbers: min ΔE **65** with red,
/// **49** with violet. Red↔orange sit close in hue but 79 against 175 in
/// luminance, and that 2.2× carries them apart where hue does not;
/// violet↔magenta are close in *both* (67 against 97), so nothing rescues them
/// — and that pair is thinking↔tools, the two an operator most needs apart.
pub fn doing_badge(doing: crate::nav::convs::Doing) -> (egui::Color32, &'static str) {
    use crate::nav::convs::Doing;
    match doing {
        // The mark at rest is hydra green, so idle is the logo, not a state
        // painted over it — and an empty seat reads the same as an idle one.
        Doing::Idle => (HYDRA, "idle — nothing in flight"),
        // The request is out and nothing has come back: brazen, the hue yog
        // already wears for waiting on something else to act.
        Doing::Waiting => (BRAZEN, "waiting — the call is open, nothing back yet"),
        // Reasoning, which displays nothing — the segment that used to look
        // identical to a stalled call. Sigil, whose stated job in the palette
        // is already to never blend into a definite state's hue.
        Doing::Thinking => (SIGIL, "thinking — reasoning, no text yet"),
        // Answering: the same spectral blue the InFlight badge and the live
        // streaming tail already wear for a model call producing text.
        Doing::Inference => (SPECTRE, "inference — the answer is streaming"),
        Doing::Tools => (ICHOR, "tools — a tool call is executing"),
    }
}
