//! Config-frozen-at as a *query* (VISION V1.2; REMOTE §9.7, bl-13f9), over a
//! real config lineage. The §5.1 #17 fold itself is pinned in
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
    assert_eq!(gov.branch_name_if_tip_of_one.as_deref(), Some("default"));
    assert!(gov.files.contains(&"version".to_owned()));
}

/// Pinned, the commit is the one the seat named and nothing about the agent is
/// read — VISION V1.2's fold, spelled as a selection. Asked at a commit the
/// lineage has since advanced past, the answer freezes there and names no
/// branch, which is the pin's whole point.
#[test]
fn a_named_commit_is_the_one_the_walk_starts_from() {
    let (fx, snap) = world();
    let frozen = governing(&snap, &fx.path, AGENT, None)
        .expect("the fork point")
        .oid;
    fx.commit_other("workflow.yaml", "events: {}\n");
    let advanced = governing(&snap, &fx.path, AGENT, None).expect("still derivable");
    assert_eq!(advanced.oid, frozen, "the agent's own tip has not moved");
    // The same question asked at the lineage's *new* head answers that commit
    // instead, still-a-tip and carrying the file the advance added.
    let head = crate::config_edit::branch::config_branches(&fx.path)
        .unwrap()
        .into_iter()
        .find(|b| b.name == "default")
        .unwrap()
        .tip_oid;
    let pinned = governing(&snap, &fx.path, AGENT, Some(&head)).expect("derivable at the head");
    assert_eq!(pinned.oid, head);
    assert!(pinned.files.contains(&"workflow.yaml".to_owned()));
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
