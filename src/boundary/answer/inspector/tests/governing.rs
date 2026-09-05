//! Which-config-governs as a *query* (VISION V1.2; REMOTE §9.7, bl-13f9), over
//! a real config lineage. The §5.1 #17 derivation itself is pinned in
//! `config_edit::branch::tests::governing`; what is pinned here is the one
//! thing this layer adds — **which commit the walk starts from** — and the
//! refusal that stands where the family's other reads answer absent.

use super::super::governing;
use crate::boundary::tests::{agent, snapshot};
use crate::git_tree::tests::fixture::Fixture;
use crate::git_tree::{AgentState, GitTree};

const AGENT: &str = "r1";

/// A workspace with one conversation forked off `config/default`, and a
/// snapshot carrying that conversation at its real tip.
fn world() -> (Fixture, crate::app::Snapshot) {
    let fx = Fixture::new();
    fx.agent_off(AGENT, "config/default");
    let tip = GitTree::from_repo(&fx.path)
        .unwrap()
        .agents
        .into_iter()
        .find(|a| a.agent_id == AGENT)
        .unwrap()
        .tip_oid;
    let mut row = agent(AGENT, AgentState::Quiescent, 100);
    row.tip_oid = tip;
    let snap = snapshot(&fx.path, "alba", vec![row], vec![]);
    (fx, snap)
}

/// Unpinned, the commit is the agent's own tip — resolved off the published
/// snapshot, so a seat asks without holding one. That is what makes this the
/// `Files` shape rather than a read that needs a tip handed to it.
#[test]
fn an_unnamed_commit_is_the_agents_own_tip() {
    let (fx, snap) = world();
    let gov = governing(&snap, &fx.path, AGENT, None).expect("the fork point is derivable");
    assert_eq!(gov.followed_lineage().as_deref(), Some("default"));
    assert!(gov.files.contains(&"version".to_owned()));
}

/// Pinned, the commit is the one the seat named and nothing about the agent is
/// read — VISION V1.2's fold, spelled as a selection.
///
/// **What the pin can and cannot buy, since bl-e654.** It still chooses the
/// rev the walk starts from, which is the only thing this layer adds. What it
/// no longer buys is an *as-of policy*: under follow-the-tip the answer is the
/// followed lineage's current head whatever rev the walk starts from on that
/// lineage, so an unpinned read and a read pinned to an older commit of the
/// same lineage agree — and both move when the lineage advances. Per-step
/// policy provenance is not derivable from ancestry any more; litany SHIPPED
/// it in 0.0.10 (their bl-e4a0: each step's `meta.json` records
/// `config_commit` and `workflow_commit` beside the branch tip it already
/// carried). yog reads that record field-by-field and asks for neither yet, so
/// per-step provenance is available to a reader that wants it and this layer
/// is unchanged — what it answers is still the followed lineage's head, and
/// asserting the old freeze here would pin a fact the engine stopped
/// producing.
#[test]
fn a_named_commit_is_the_one_the_walk_starts_from() {
    let (fx, snap) = world();
    let at_fork = governing(&snap, &fx.path, AGENT, None)
        .expect("the fork point")
        .oid;
    fx.commit_other("workflow.yaml", "events: {}\n");
    let head = crate::config_edit::branch::config_branches(&fx.path)
        .unwrap()
        .into_iter()
        .find(|b| b.name == "default")
        .unwrap()
        .tip_oid;
    // The conversation followed the advance: unpinned, the answer is the new
    // head and carries the file the edit added.
    let advanced = governing(&snap, &fx.path, AGENT, None).expect("still derivable");
    assert_ne!(advanced.oid, at_fork, "the lineage moved and it followed");
    assert_eq!(advanced.oid, head);
    assert!(advanced.files.contains(&"workflow.yaml".to_owned()));
    // Pinned to the commit the conversation forked off, the walk starts there
    // and resolves the same followed head — the selection chooses the rev, not
    // a moment in policy.
    let pinned = governing(&snap, &fx.path, AGENT, Some(&at_fork)).expect("derivable at the fork");
    assert_eq!(pinned.oid, head);
}

/// It **refuses** where its siblings answer absent: an agent the snapshot does
/// not carry has no tip, and "this conversation has no policy" is never a
/// reading. The engine's own sentence is what the seat paints.
#[test]
fn an_agent_the_snapshot_does_not_carry_refuses_rather_than_answering_absent() {
    let (fx, snap) = world();
    let said = governing(&snap, &fx.path, "unheard-of", None).expect_err("no tip, no answer");
    assert!(!said.is_empty(), "the refusal carries git's own words");
}
