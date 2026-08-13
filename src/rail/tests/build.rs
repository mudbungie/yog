//! **S10-T1 rail-spine** and **S10-T2 two-edges**: one notch per step at that
//! step's read-state commit, and a child's two edges derived apart — context
//! from git ancestry, provenance from the descent id and the dispatch notch.
//! Since bl-1802 the context edge's whole rendering is the fork label's
//! wording, so that is what these assert.

use super::{chat, child, commit, seat, step, steps};
use crate::rail::{Rail, build};

/// The spine: one notch per step, each carrying that step's `meta.json`
/// commit, and a step that recorded none is still a notch — it says so.
#[test]
fn one_notch_per_step_carries_that_steps_read_state_commit() {
    let rail = build(
        "root",
        &[commit("aaaa1111", 10), commit("bbbb2222", 20)],
        &steps(vec![
            step("001", Some("aaaa1111"), 5),
            step("002", Some("bbbb2222"), 7),
            step("003", None, 0),
        ]),
        &chat(3),
        &[],
    );
    let seqs: Vec<&str> = rail.notches.iter().map(|n| n.seq.as_str()).collect();
    assert_eq!(seqs, ["001", "002", "003"]);
    let shorts: Vec<String> = rail
        .notches
        .iter()
        .map(super::super::Notch::short)
        .collect();
    assert_eq!(shorts, ["aaaa111", "bbbb222", "—"]);
    assert_eq!(rail.notches.get(2).and_then(|n| n.commit.clone()), None);
}

/// The burden check, mechanical: nothing dispatched — no cards and no edges,
/// so the chat carries exactly the faint rules bl-929d already drew, one per
/// commit boundary, and an operator who never clicks one sees today's
/// transcript exactly. There is no gutter left to withhold.
#[test]
fn nothing_dispatched_leaves_the_chat_exactly_as_it_was() {
    let one = build(
        "root",
        &[commit("aaaa1111", 10)],
        &steps(vec![step("001", Some("aaaa1111"), 5)]),
        &chat(1),
        &[],
    );
    assert!(one.cards.is_empty());
    assert_eq!(one.rules().get(&seat(0)), Some(&0));
    assert!(Rail::default().rules().is_empty());
}

/// A **fork** child shares its parent's commit prefix, so it wears both edges:
/// the solid context edge at the fork point and the dashed provenance edge at
/// the dispatch notch. Forked at the notch it was dispatched from, the label
/// is "from here".
#[test]
fn a_fork_child_wears_both_edges_and_reads_from_here() {
    let parent = vec![commit("aaaa1111", 10), commit("bbbb2222", 20)];
    let forked = child(
        "hare",
        vec![
            commit("aaaa1111", 10),
            commit("bbbb2222", 20),
            commit("cccc3333", 30),
        ],
    );
    let rail = build(
        "storeroom",
        &parent,
        &steps(vec![
            step("001", Some("aaaa1111"), 0),
            step("002", Some("bbbb2222"), 0),
        ]),
        &chat(2),
        &[forked],
    );
    let card = rail.cards.first().expect("the fork has a card");
    assert_eq!(card.provenance_notch, 1);
    assert_eq!(card.fork, "from here");
}

/// Forked further back than the notch it was dispatched at, the two edges land
/// on different notches and the label names the parent and the oid — the whole
/// point of keeping them apart.
#[test]
fn a_fork_from_an_older_notch_splits_the_two_edges() {
    let parent = vec![
        commit("aaaa1111", 10),
        commit("bbbb2222", 20),
        commit("cccc3333", 30),
    ];
    let forked = child("hare", vec![commit("aaaa1111", 10), commit("dddd4444", 40)]);
    let rail = build(
        "storeroom",
        &parent,
        &steps(vec![
            step("001", Some("aaaa1111"), 0),
            step("002", Some("bbbb2222"), 0),
            step("003", Some("cccc3333"), 0),
        ]),
        &chat(3),
        &[forked],
    );
    let card = rail.cards.first().expect("the fork has a card");
    assert_eq!(card.provenance_notch, 2);
    assert_eq!(card.fork, "from storeroom@aaaa111");
}

/// A **clean** child shares no commit with its parent, so it has provenance
/// and no ancestry: one edge, and a label naming the config branch it started
/// from. The dispatch notch is still located, by birth time.
#[test]
fn a_clean_child_has_provenance_only_and_names_its_config_branch() {
    let parent = vec![commit("aaaa1111", 10), commit("bbbb2222", 20)];
    let mut clean = child("mole", vec![commit("ffff9999", 25)]);
    clean.config_label = Some("default".to_owned());
    let rail = build(
        "storeroom",
        &parent,
        &steps(vec![
            step("001", Some("aaaa1111"), 0),
            step("002", Some("bbbb2222"), 0),
        ]),
        &chat(2),
        &[clean],
    );
    let card = rail.cards.first().expect("the clean child has a card");
    assert_eq!(card.provenance_notch, 1);
    assert_eq!(card.fork, "from config/default");
}

/// A clean child whose governing commit is no branch's tip still says what it
/// is, plainly, rather than naming a branch that no longer points there.
#[test]
fn a_clean_child_with_no_named_branch_says_so() {
    let rail = build(
        "storeroom",
        &[commit("aaaa1111", 10), commit("bbbb2222", 20)],
        &steps(vec![
            step("001", Some("aaaa1111"), 0),
            step("002", Some("bbbb2222"), 0),
        ]),
        &chat(2),
        &[child("mole", vec![commit("ffff9999", 5)])],
    );
    let card = rail.cards.first().expect("the clean child has a card");
    assert_eq!(card.fork, "from a config commit");
    // Born before any notch's commit: the rail's head, never nowhere.
    assert_eq!(card.provenance_notch, 0);
}

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
