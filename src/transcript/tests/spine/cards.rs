//! **S10-T4 spine-paint**, the cards half: what hangs *under* an operable
//! commit's rule — the child born there, and V2.3's cohort when more than one
//! was. Split from [`super`] at §12's cap, on the seam between the rule (the
//! commit and its gesture) and what the commit gave birth to.

use super::{child, commit, follow_click, painted, spine};
use crate::transcript::tests::rows::SPEAKER;

/// The card hangs under the rule of the commit its child was born at: the
/// child's name, where it forked from, its state chip, its own spend, and the
/// tail of what it is saying right now.
#[test]
fn a_card_paints_under_the_rule_its_child_was_born_at() {
    let forked = child(
        "hare",
        vec![
            commit("0123456789abcdef", 10),
            commit("bbbb2222", 20),
            commit("cccc3333", 30),
        ],
    );
    let text = painted(
        &spine([Some("0123456789abcdef"), Some("bbbb2222")], vec![forked]),
        &mut None,
    );
    for fact in ["hare", "from here", "512 tokens", "hare is thinking", "◐"] {
        assert!(text.contains(fact), "{fact} is missing:\n{text}");
    }
}

/// **S12-T4 fan-paint**: a cohort renders as one group at its birth rule — the
/// shared ancestry said once above the columns, and every candidate carrying
/// the four facts an operator judges by.
#[test]
fn a_cohort_paints_its_shared_ancestry_once_and_every_candidate_beside_it() {
    let member = |name: &str, own: &str| {
        child(
            name,
            vec![
                commit("0123456789abcdef", 10),
                commit("bbbb2222", 20),
                commit(own, 30),
            ],
        )
    };
    let text = painted(
        &spine(
            [Some("0123456789abcdef"), Some("bbbb2222")],
            vec![member("hare", "cccc3333"), member("wren", "dddd4444")],
        ),
        &mut None,
    );
    assert!(text.contains("×2"), "the cohort states its width:\n{text}");
    assert_eq!(
        text.matches("from here").count(),
        1,
        "shared ancestry is stated once, not per column:\n{text}"
    );
    for fact in ["hare", "wren", "hare is thinking", "wren is thinking"] {
        assert!(text.contains(fact), "{fact} is missing:\n{text}");
    }
}

/// A cohort of one wears no header and is V1's card unchanged; a cohort with
/// no common ancestry says so and every column states its own.
#[test]
fn a_cohort_of_one_wears_no_header_and_a_mixed_one_says_mixed() {
    let clean = child("mole", vec![commit("ffff9999", 25)]);
    let lone = painted(
        &spine(
            [Some("0123456789abcdef"), Some("bbbb2222")],
            vec![clean.clone()],
        ),
        &mut None,
    );
    assert!(!lone.contains("×1"), "no header for one:\n{lone}");
    assert!(lone.contains("from a config commit"), "{lone}");

    let forked = child(
        "hare",
        vec![
            commit("0123456789abcdef", 10),
            commit("bbbb2222", 20),
            commit("cccc3333", 24),
        ],
    );
    let mixed = painted(
        &spine(
            [Some("0123456789abcdef"), Some("bbbb2222")],
            vec![forked, clean],
        ),
        &mut None,
    );
    assert!(mixed.contains("×2"), "{mixed}");
    assert!(mixed.contains("mixed"), "no ancestry to hoist:\n{mixed}");
    assert!(mixed.contains("from here"), "{mixed}");
    assert!(mixed.contains("from a config commit"), "{mixed}");
}

/// Following a card is the ordinary selection gesture: the click hands back
/// the child's agent id for the caller to retarget to.
#[test]
fn clicking_a_card_asks_to_open_that_child() {
    let rail = spine(
        [Some("0123456789abcdef"), Some("bbbb2222")],
        vec![child("mole", vec![commit("ffff9999", 25)])],
    );
    assert_eq!(
        follow_click(&rail, &mut None, "mole").as_deref(),
        Some("root-mole")
    );
    assert_eq!(SPEAKER, "shudder-storeroom");
}
