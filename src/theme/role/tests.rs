//! The §11 role vocabulary: the byte-derivation, the hue-and-words mapping,
//! and the stripe's paint.

use super::{Role, message_role, role_badge, role_stripe};
use crate::paint_probe::paint_fills;
use crate::theme::{BRAZEN, BRAZEN_DIM, GATE, SPECTRE};

#[test]
fn the_reserved_user_token_is_the_operator_and_nothing_else_is() {
    assert_eq!(message_role("user", false), Role::User);
    assert_eq!(message_role("p1-worker", false), Role::Peer);
    // An absent/unknown sender reads as third-party mail, never the operator.
    assert_eq!(message_role("", false), Role::Peer);
}

#[test]
fn an_epitaph_marks_a_result_deposit_before_any_sender_reading() {
    assert_eq!(message_role("some-child-id", true), Role::Ended);
    // A kind before a sender: even the reserved token yields to the ending.
    assert_eq!(message_role("user", true), Role::Ended);
}

#[test]
fn each_role_wears_its_own_palette_hue_and_words() {
    let cases = [
        (Role::User, GATE),
        (Role::Model, SPECTRE),
        (Role::Peer, BRAZEN),
        (Role::Ended, BRAZEN_DIM),
    ];
    let mut words_seen = Vec::new();
    for (role, hue) in cases {
        let (got_hue, words) = role_badge(role);
        assert_eq!(got_hue, hue, "{role:?}");
        assert!(!words.is_empty(), "{role:?} must not ship wordless");
        words_seen.push(words);
    }
    // The peer and ended hues are one bronze family, but the words differ —
    // and every role says something the others don't.
    words_seen.sort_unstable();
    words_seen.dedup();
    assert_eq!(words_seen.len(), 4, "each role must be worded apart");
}

#[test]
fn a_role_paints_its_stripe_and_no_role_paints_none() {
    for (role, hue) in [
        (Role::User, GATE),
        (Role::Model, SPECTRE),
        (Role::Peer, BRAZEN),
        (Role::Ended, BRAZEN_DIM),
    ] {
        let fills = paint_fills(|ui| role_stripe(ui, Some(role)));
        assert!(fills.contains(&hue), "{role:?} must paint {hue:?}");
    }
    let blank = paint_fills(|ui| role_stripe(ui, None));
    for hue in [GATE, SPECTRE, BRAZEN, BRAZEN_DIM] {
        assert!(!blank.contains(&hue), "an empty seat paints no role hue");
    }
}
