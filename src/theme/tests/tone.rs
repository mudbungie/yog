//! The §11 **pending solidity** (bl-915e): a send is shown in-memory in faded
//! colour, brightening when it is actually locked in as a statement. Split from the sibling badge corpus at
//! §12's budget — the badge tests ask what a fact *looks* like, this one asks
//! how much of it there is.

use crate::transcript::Tone;

/// Two facts, and both matter: `Weak` really fades, and every other tone is a
/// fact the derivation already asserts, so it paints solid — a mapping that
/// dimmed anything else would fade rows nobody sent.
#[test]
fn only_the_weak_tone_fades_and_it_really_does() {
    let faded = crate::theme::tone_solidity(Tone::Weak);
    assert!(
        (0.0..1.0).contains(&faded),
        "a pending row is visibly less than solid: {faded}"
    );
    for solid in [
        Tone::Plain,
        Tone::Good,
        Tone::Bad,
        Tone::Live,
        Tone::InFlight,
    ] {
        assert!(
            (crate::theme::tone_solidity(solid) - 1.0).abs() < f32::EPSILON,
            "{solid:?} is a derived fact and paints at full strength"
        );
    }
}
