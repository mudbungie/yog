//! **S10-T2**'s other half: the child **card** itself, as against the spine and
//! the two edges it hangs by ([`build`](super::build)). Two things a card
//! cannot hang from, the anchor it falls back to when a notch's commit is off
//! the branch, the streaming tail clipped to a glance, and the child's own
//! identity and spend — the per-agent fold, so a fork's shared prefix cost
//! stays with the ancestor.
//!
//! Split from [`build`](super::build) at §12's budget on the seam §12 states
//! for the production modules: `mod` is the spine's shape, `cards` a child's
//! placement on it.

use super::{chat, child, commit, step, steps};
use crate::rail::build;

/// Two things a card cannot hang from, both values rather than arms: a parent
/// with no steps has no rail, and a child with no commit has no birth to place.
#[test]
fn a_card_needs_a_notch_and_a_birth() {
    let no_steps = build(
        "root",
        &[commit("aaaa1111", 10)],
        &steps(vec![]),
        &chat(0),
        &[child("mole", vec![commit("ffff9999", 20)])],
    );
    assert!(no_steps.cards.is_empty());
    let no_commits = build(
        "root",
        &[commit("aaaa1111", 10)],
        &steps(vec![step("001", Some("aaaa1111"), 0)]),
        &chat(1),
        &[child("mole", vec![])],
    );
    assert!(no_commits.cards.is_empty());
}

/// A notch whose commit is not on the parent's list positions nowhere, so it
/// anchors no edge — the derivation declines rather than guessing a neighbour.
#[test]
fn a_notch_commit_off_the_branch_anchors_nothing() {
    let rail = build(
        "storeroom",
        &[commit("aaaa1111", 10)],
        &steps(vec![
            step("001", Some("aaaa1111"), 0),
            step("002", Some("9999zzzz"), 0),
        ]),
        &chat(2),
        &[child(
            "hare",
            vec![commit("aaaa1111", 10), commit("cccc3333", 30)],
        )],
    );
    let card = rail.cards.first().expect("the fork has a card");
    assert_eq!(card.provenance_notch, 0);
}

/// The card's streaming tail is the last line or two of the child's in-flight
/// text, clipped — a glance at whether it is moving, not a reader.
#[test]
fn the_card_tail_is_the_last_lines_of_in_flight_text() {
    let base = || {
        vec![
            step("001", Some("aaaa1111"), 0),
            step("002", Some("bbbb2222"), 0),
        ]
    };
    let parent = || vec![commit("aaaa1111", 10), commit("bbbb2222", 20)];
    let mut talking = child("mole", vec![commit("ffff9999", 25)]);
    talking.streaming_text = Some("one\n\ntwo\nthree\n".to_owned());
    let rail = build("storeroom", &parent(), &steps(base()), &chat(0), &[talking]);
    assert_eq!(
        rail.cards.first().and_then(|c| c.tail.clone()),
        Some("two three".to_owned())
    );

    let mut blank = child("mole", vec![commit("ffff9999", 25)]);
    blank.streaming_text = Some("   \n\n".to_owned());
    let quiet = build("storeroom", &parent(), &steps(base()), &chat(0), &[blank]);
    assert_eq!(quiet.cards.first().and_then(|c| c.tail.clone()), None);

    let mut long = child("mole", vec![commit("ffff9999", 25)]);
    long.streaming_text = Some("x".repeat(400));
    let clipped = build("storeroom", &parent(), &steps(base()), &chat(0), &[long]);
    assert_eq!(
        clipped
            .cards
            .first()
            .and_then(|c| c.tail.as_ref())
            .map(|t| t.chars().count()),
        Some(140)
    );
}

/// The card carries the child's own identity and spend — the per-agent fold,
/// so a fork's shared prefix cost stays with the ancestor.
#[test]
fn the_card_carries_the_childs_own_id_and_spend() {
    let mut spent = child("mole", vec![commit("ffff9999", 25)]);
    spent.tokens = 4242;
    let rail = build(
        "storeroom",
        &[commit("aaaa1111", 10), commit("bbbb2222", 20)],
        &steps(vec![
            step("001", Some("aaaa1111"), 0),
            step("002", Some("bbbb2222"), 0),
        ]),
        &chat(2),
        &[spent],
    );
    let card = rail.cards.first().expect("a card");
    assert_eq!(card.agent_id, "root-mole");
    assert_eq!(card.name, "mole");
    assert_eq!(card.tokens, 4242);
}
