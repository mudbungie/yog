//! **Which config governs** (§5.1 #17) — the merge-base fold to the fork
//! commit, a faithful port of litany `workspace.rs::{governing_config,
//! nearest}`, and the follow derivation over it (`branch::follow`), a port of
//! litany `workspace/current_config.rs`. Every arm of both is covered here,
//! because the two are one question and only their composition is observable:
//! fresh fork point, the **followed tip** after the lineage advances (the arm
//! bl-e654 inverted — it used to assert a freeze), both `nearest` directions,
//! the equal-candidate short-circuit, several refs on one commit deduplicating
//! to a followed lineage, the skip of an unrelated orphan config, the **held**
//! state two diverged lineages produce, and the two loud declines (no ancestor
//! / incomparable).

use super::super::{Governance, config_branches, governing_config};
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
    assert_eq!(gov.followed_lineage().as_deref(), Some("default"));
    assert_eq!(gov.diverged_lineages(), 0);
    assert!(gov.files.contains(&"version".to_string()));
}

/// **The inversion itself** (bl-e654). This assertion used to read the other
/// way — the agent stayed on its fork commit and the advanced lineage governed
/// only the next conversation. Under follow-the-tip the running conversation
/// resolves the advanced head at its next step, so the edit's own file is in
/// the answer's tree and the label names the lineage rather than a freeze.
#[test]
fn governing_config_follows_the_lineage_tip_after_the_branch_advances() {
    let fx = Fixture::new();
    fx.agent_off("r1", "config/default");
    // The user advances config/default past the agent's fork point.
    fx.commit_other("workflow.yaml", "events: {}\n");
    let gov = governing_config(&fx.path, &tip(&fx, "r1")).unwrap();
    assert_eq!(gov.oid, branch_tip(&fx, "default"));
    assert_eq!(gov.governance, Governance::Follows("default".to_owned()));
    // The edit reached it: the advanced commit's tree is what governs now.
    assert!(gov.files.contains(&"version".to_string()));
    assert!(gov.files.contains(&"workflow.yaml".to_string()));
    assert_eq!(
        gov.label(),
        format!("policy follows config/default, now at {}", gov.short_oid)
    );
}

/// **Two lineages reaching one conversation is held, not guessed.** `strict`
/// forks the agent's own fork commit and `default` advances past it, so both
/// heads contain it and neither may be picked: control stays on the fork
/// commit, the count rides the answer, and `retarget` is the act that settles
/// it. This is the state litany announces on the driver's stderr at every
/// step; yog derives it here rather than reading that sentence back (bl-b95e).
#[test]
fn two_diverged_lineages_hold_the_conversation_on_its_fork_commit() {
    let fx = Fixture::new();
    fx.agent_off("r1", "config/default");
    let fork_point = branch_tip(&fx, "default");
    fx.config_off("strict", "config/default");
    fx.commit_other("workflow.yaml", "events: {}\n");
    let gov = governing_config(&fx.path, &tip(&fx, "r1")).unwrap();
    assert_eq!(gov.oid, fork_point);
    assert_eq!(
        gov.governance,
        Governance::Held {
            diverged_lineages: 2
        }
    );
    assert_eq!(gov.followed_lineage(), None);
    assert_eq!(gov.diverged_lineages(), 2);
    // Held means held: the advance neither lineage settled is not in the tree.
    assert!(!gov.files.contains(&"workflow.yaml".to_string()));
    assert_eq!(
        gov.label(),
        format!(
            "policy held at {} — 2 diverged config lineages",
            gov.short_oid
        )
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
    assert_eq!(gov.followed_lineage().as_deref(), Some("strict"));
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
    // Two refs, ONE distinct tip: deduplication is what makes this a followed
    // lineage rather than a divergence, and the name is the first ref in
    // `for-each-ref` order at that tip.
    assert_eq!(gov.governance, Governance::Follows("default".to_owned()));
}

#[test]
fn governing_config_skips_an_unrelated_orphan_config() {
    let fx = Fixture::new();
    fx.agent_off("r1", "config/default");
    // Shares no history with the agent: merge-base miss, contributes nothing.
    fx.orphan_config("island");
    let gov = governing_config(&fx.path, &tip(&fx, "r1")).unwrap();
    // Skipped by both halves: no merge-base for the fold, and no containment
    // for the follow, so it is never one of the tips that could hold this.
    assert_eq!(gov.followed_lineage().as_deref(), Some("default"));
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
