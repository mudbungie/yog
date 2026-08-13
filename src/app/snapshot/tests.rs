//! The snapshot's two derived facts: what a conversation's descent counts, and
//! what grew between two derivations (§7.2, bl-ee0a).

use super::*;
use crate::app::tests::agent;
use crate::git_tree::AgentState;

/// A tree of `ids` in lernie's §2.3 grammar — an id is a chain of `<ts>-<short>`
/// segments, so `alpha-1` is a root and `alpha-1-kid-2` its child. All
/// quiescent: growth is about the branch set, not liveness.
fn tree(ids: &[&str]) -> GitTree {
    GitTree {
        commits: vec![],
        agents: ids
            .iter()
            .map(|id| agent(id, AgentState::Quiescent))
            .collect(),
    }
}

fn ws() -> PathBuf {
    PathBuf::from("/ws")
}

#[test]
fn branch_counts_count_a_root_plus_its_descent() {
    let counts = branch_counts(&tree(&[
        "alpha-1",
        "alpha-1-kid-1",
        "alpha-1-kid-1-deep-1",
        "beta-1",
    ]));
    assert_eq!(
        counts.get("alpha-1"),
        Some(&3),
        "the root and its two children"
    );
    assert_eq!(counts.get("beta-1"), Some(&1), "a lone root counts itself");
    assert_eq!(counts.len(), 2, "one entry per conversation: {counts:?}");
    assert!(branch_counts(&GitTree::default()).is_empty());
}

#[test]
fn growth_names_the_conversation_that_gained_branches() {
    let before = tree(&["alpha-1", "beta-1"]);
    let after = tree(&["alpha-1", "alpha-1-kid-1", "alpha-1-kid-2", "beta-1"]);
    let grew = growth_between(&ws(), Some(&before), &after);
    assert_eq!(grew.len(), 1, "only alpha grew: {grew:?}");
    assert_eq!(grew[0].added, 2);
    assert_eq!(grew[0].workspace, ws());
    assert_eq!(grew[0].conversation, "alpha-1", "the §3.3 display name");
}

#[test]
fn a_first_derivation_and_a_shrinking_one_are_not_growth() {
    let after = tree(&["alpha-1", "alpha-1-kid-1"]);
    assert!(
        growth_between(&ws(), None, &after).is_empty(),
        "nothing to compare against is not growth"
    );
    let before = tree(&["alpha-1", "alpha-1-kid-1", "alpha-1-kid-2"]);
    assert!(
        growth_between(&ws(), Some(&before), &after).is_empty(),
        "a conversation that shrank is not growth"
    );
    // A conversation that did not exist before appeared; the roster says so and
    // the growth line must not claim it grew.
    let fresh = tree(&["alpha-1", "alpha-1-kid-1", "gamma-1"]);
    let grew = growth_between(&ws(), Some(&after), &fresh);
    assert!(grew.is_empty(), "a new root is not growth: {grew:?}");
}

#[test]
fn the_growth_line_leads_with_the_biggest_grower_and_counts_the_rest() {
    assert_eq!(growth_label(&[]), None, "a quiet world says nothing");
    let before = tree(&["alpha-1", "beta-1"]);
    let after = tree(&[
        "alpha-1",
        "alpha-1-kid-1",
        "alpha-1-kid-2",
        "alpha-1-kid-3",
        "beta-1",
        "beta-1-kid-1",
    ]);
    let grew = growth_between(&ws(), Some(&before), &after);
    assert_eq!(
        growth_label(&grew).as_deref(),
        Some("alpha-1 +3 branches (and 1 more)"),
    );
    assert_eq!(
        growth_label(&grew[..1]).as_deref(),
        Some("alpha-1 +3 branches"),
        "a single grower needs no tail",
    );
}

#[test]
fn the_empty_snapshot_is_the_general_shape_with_no_inputs() {
    let now = std::time::Instant::now();
    let empty = Snapshot::empty(now);
    assert!(empty.workspaces.is_empty());
    assert!(empty.trees.is_empty());
    assert!(empty.balls_by_project.is_empty());
    assert!(empty.join_rows.is_empty());
    assert!(empty.ops.is_empty());
    assert!(empty.growth.is_empty());
    assert_eq!(empty.ui_bytes, None);
    assert_eq!(empty.derived_at, now);
    assert_eq!(empty.clone(), empty, "cheap to hand out, compares by value");
}
