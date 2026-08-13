//! **S12-T3 cohort-one-path** (the derivation half): the fan is a grouping of
//! V1's cards, and a lone child is that grouping with one member. Nothing here
//! reads a registry, because there is none to read.

use super::{chat, child, commit, step, steps};
use crate::rail::{ChildInput, Rail, build, cohorts};

fn parent() -> Vec<crate::git_tree::StepCommit> {
    vec![commit("aaaa1111", 10), commit("bbbb2222", 20)]
}

fn rail(children: Vec<ChildInput>) -> Rail {
    build(
        "storeroom",
        &parent(),
        &steps(vec![
            step("001", Some("aaaa1111"), 5),
            step("002", Some("bbbb2222"), 7),
        ]),
        &chat(2),
        &children,
    )
}

/// A child forked at the second notch: it carries the parent's whole prefix
/// and one commit of its own.
fn forked(name: &str, at: i64) -> ChildInput {
    child(
        name,
        vec![
            commit("aaaa1111", 10),
            commit("bbbb2222", 20),
            commit(name, at),
        ],
    )
}

/// Three candidates fired from one mark are one cohort, and the ancestry they
/// share is stated once — that is the whole of what "cohort" means here.
#[test]
fn candidates_born_at_one_notch_are_one_cohort() {
    let out = cohorts(&rail(vec![
        forked("hare", 30),
        forked("mole", 31),
        forked("wren", 32),
    ]));
    assert_eq!(out.len(), 1);
    let fan = &out[0];
    assert_eq!(fan.notch, 1, "anchored at the birth notch (VISION V1.7)");
    assert_eq!(fan.members.len(), 3);
    assert!(fan.fanned());
    assert_eq!(fan.common.as_deref(), Some("from here"));
}

/// **One attempt is the same path**: a lone child is a cohort of one, sharing
/// its ancestry with itself. No arm anywhere branches on the count.
#[test]
fn a_lone_child_is_a_cohort_of_one() {
    let out = cohorts(&rail(vec![forked("hare", 30)]));
    assert_eq!(out.len(), 1);
    assert!(!out[0].fanned());
    assert_eq!(out[0].common.as_deref(), Some("from here"));
}

/// Candidates that forked off different refs have no common ancestry to lift
/// out — so there is none, and each column says its own. Absence is a value.
#[test]
fn candidates_off_different_refs_share_no_ancestry() {
    let clean = child("mole", vec![commit("ffff9999", 25)]);
    let out = cohorts(&rail(vec![forked("hare", 24), clean]));
    assert_eq!(out.len(), 1, "born at one notch either way");
    assert_eq!(out[0].common, None);
    let labels: Vec<&str> = out[0].members.iter().map(|m| m.fork.as_str()).collect();
    assert_eq!(labels, vec!["from here", "from a config commit"]);
}

/// Cohorts come back in notch order, one per birth notch — a partition of the
/// cards, never a second copy of them.
#[test]
fn cohorts_partition_the_cards_in_notch_order() {
    let early = child("early", vec![commit("aaaa1111", 10), commit("early", 12)]);
    let built = rail(vec![forked("hare", 30), early]);
    let out = cohorts(&built);
    let notches: Vec<usize> = out.iter().map(|c| c.notch).collect();
    assert_eq!(notches, vec![0, 1]);
    let total: usize = out.iter().map(|c| c.members.len()).sum();
    assert_eq!(total, built.cards.len());
    // No dispatches, no cohorts — V1's burden check reaches this rung too.
    assert!(cohorts(&rail(vec![])).is_empty());
}
