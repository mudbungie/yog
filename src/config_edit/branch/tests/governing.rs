//! The governing-config merge-base fold (§5.1 #17), a faithful port of
//! litany `workspace.rs::{governing_config, nearest}`. Every arm is covered:
//! fresh fork point, the frozen ancestor after advancement, both `nearest`
//! directions, the equal-candidate short-circuit, the skip of an unrelated
//! orphan config, and the two loud declines (no ancestor / incomparable).

use super::super::{config_branches, governing_config};
use crate::git_tree::tests::fixture::Fixture;
use crate::git_tree::{GitTree, GitTreeError};

/// The tip oid of agent `id`, read back through the real view-model — the
/// same value the shell hands `governing_config` in production.
fn tip(fx: &Fixture, id: &str) -> String {
    GitTree::from_repo(&fx.path)
        .unwrap()
        .agents
        .into_iter()
        .find(|a| a.agent_id == id)
        .unwrap()
        .tip_oid
}

/// The tip oid of `config/<name>`, via the browse surface under test.
fn branch_tip(fx: &Fixture, name: &str) -> String {
    config_branches(&fx.path)
        .unwrap()
        .into_iter()
        .find(|b| b.name == name)
        .unwrap()
        .tip_oid
}

#[test]
fn governing_config_is_the_fork_point_for_a_fresh_agent() {
    let fx = Fixture::new();
    fx.agent_off("r1", "config/default");
    let gov = governing_config(&fx.path, &tip(&fx, "r1")).unwrap();
    assert_eq!(gov.oid, branch_tip(&fx, "default"));
    assert_eq!(gov.branch_name_if_tip_of_one.as_deref(), Some("default"));
    assert!(gov.files.contains(&"version".to_string()));
}

#[test]
fn governing_config_freezes_at_the_fork_point_after_the_branch_advances() {
    let fx = Fixture::new();
    fx.agent_off("r1", "config/default");
    // The user advances config/default past the agent's fork point.
    fx.commit_other("workflow.yaml", "events: {}\n");
    let gov = governing_config(&fx.path, &tip(&fx, "r1")).unwrap();
    assert_ne!(gov.oid, branch_tip(&fx, "default"));
    assert_eq!(gov.branch_name_if_tip_of_one, None);
    // Frozen at the pre-advance commit: workflow.yaml is not there yet.
    assert!(gov.files.contains(&"version".to_string()));
    assert!(!gov.files.contains(&"workflow.yaml".to_string()));
    assert_eq!(
        gov.frozen_label(),
        format!("policy frozen at {}", gov.short_oid)
    );
}

#[test]
fn governing_config_picks_the_nearer_ancestor_arriving_later() {
    let fx = Fixture::new();
    // config/strict = default head + 1; sorts after default, so the nearer
    // candidate arrives second (the `a`-is-ancestor-of-`b` arm).
    fx.config_off("strict", "config/default");
    fx.agent_off("r1", "config/strict");
    let gov = governing_config(&fx.path, &tip(&fx, "r1")).unwrap();
    assert_eq!(gov.oid, branch_tip(&fx, "strict"));
    assert_eq!(gov.branch_name_if_tip_of_one.as_deref(), Some("strict"));
}

#[test]
fn governing_config_keeps_the_nearer_ancestor_arriving_first() {
    let fx = Fixture::new();
    // config/aaa = default head + 1; sorts before default, so the nearer
    // candidate arrives first (the `b`-is-ancestor-of-`a` arm).
    fx.config_off("aaa", "config/default");
    fx.agent_off("r1", "config/aaa");
    let gov = governing_config(&fx.path, &tip(&fx, "r1")).unwrap();
    assert_eq!(gov.oid, branch_tip(&fx, "aaa"));
}

#[test]
fn governing_config_folds_two_equal_candidates() {
    let fx = Fixture::new();
    // config/twin points at config/default's tip: two refs, one candidate.
    fx.config_alias("twin", "config/default");
    fx.agent_off("r1", "config/default");
    let gov = governing_config(&fx.path, &tip(&fx, "r1")).unwrap();
    assert_eq!(gov.oid, branch_tip(&fx, "default"));
    // find() returns the first ref in name order at that tip.
    assert_eq!(gov.branch_name_if_tip_of_one.as_deref(), Some("default"));
}

#[test]
fn governing_config_skips_an_unrelated_orphan_config() {
    let fx = Fixture::new();
    fx.agent_off("r1", "config/default");
    // Shares no history with the agent: merge-base miss, contributes nothing.
    fx.orphan_config("island");
    let gov = governing_config(&fx.path, &tip(&fx, "r1")).unwrap();
    assert_eq!(gov.branch_name_if_tip_of_one.as_deref(), Some("default"));
}

#[test]
fn governing_config_declines_an_agent_with_no_config_ancestor() {
    let fx = Fixture::new();
    fx.orphan_agent("x1");
    let err = governing_config(&fx.path, &tip(&fx, "x1")).unwrap_err();
    assert!(matches!(err, GitTreeError::Governing(_)), "{err}");
    assert!(err.to_string().contains("no config/* ancestor"), "{err}");
}

#[test]
fn governing_config_declines_incomparable_candidates() {
    let fx = Fixture::new();
    fx.agent_off("r1", "config/default");
    fx.orphan_config("island");
    // Merge the unrelated config into the agent: both config heads become
    // incomparable ancestors of the tip.
    fx.cross_merge("r1", "island");
    let err = governing_config(&fx.path, &tip(&fx, "r1")).unwrap_err();
    assert!(matches!(err, GitTreeError::Governing(_)), "{err}");
    assert!(err.to_string().contains("incomparable"), "{err}");
}
