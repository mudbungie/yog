//! `GitTree::from_repo` end-to-end tests: real workspaces (ARCH §2.2),
//! real commits, assertions on the resulting view-model's *skeleton* — the
//! trunk, the agents, their marks and descent. The step-content projection
//! (streaming text, tool calls) is [`super::steps`]'s concern, and the text a
//! row wears — its §3.3 name and its goal preview — is [`super::naming`]'s.

use super::fixture::Fixture;
use crate::git_tree::{GitTree, GitTreeError};
use tempfile::tempdir;

#[test]
fn from_repo_errors_when_repo_missing() {
    let dir = tempdir().unwrap();
    let Err(err) = GitTree::from_repo(&dir.path().join("nope")) else {
        panic!("expected error");
    };
    assert!(
        matches!(err, GitTreeError::Git { .. } | GitTreeError::Spawn(_)),
        "got {err:?}"
    );
}

#[test]
fn from_repo_errors_when_repo_git_absent() {
    // A directory that exists but has no `repo.git` (i.e. not a
    // workspace) should fail with a git error rather than silently
    // returning an empty tree.
    let dir = tempdir().unwrap();
    let err = GitTree::from_repo(dir.path()).unwrap_err();
    assert!(
        matches!(err, GitTreeError::Git { .. } | GitTreeError::Spawn(_)),
        "got {err:?}"
    );
}

#[test]
fn from_repo_surfaces_the_config_lineage_as_the_trunk() {
    let fx = Fixture::new();
    fx.commit_other("workflow.yaml", "events: {}\n");
    let tree = GitTree::from_repo(&fx.path).unwrap();
    // Init config commit + the amendment, oldest to newest (§2.2).
    assert_eq!(tree.commits.len(), 2);
    assert_eq!(tree.commits[0].subject, "config: init [config/default]");
    assert_eq!(tree.commits[1].subject, "add workflow.yaml");
    assert_eq!(tree.commits[0].short_oid.len(), 8);
    assert!(tree.agents.is_empty());
}

#[test]
fn from_repo_agent_surfaces_with_steps_and_preview() {
    let fx = Fixture::new();
    fx.build_agent("20260422T120500Z-b002", "ping v03");
    let tree = GitTree::from_repo(&fx.path).unwrap();
    assert_eq!(tree.commits.len(), 1, "config lineage untouched");
    assert_eq!(tree.agents.len(), 1);
    let agent = &tree.agents[0];
    // The ref is `agents/<id>` (§2.3); the id is the identity.
    assert_eq!(agent.branch_name, "agents/20260422T120500Z-b002");
    assert_eq!(agent.agent_id, "20260422T120500Z-b002");
    assert_eq!(agent.preview.as_deref(), Some("ping v03"));
    // Dispatch commit + compaction merge past the config lineage, each
    // carrying its subject (§7.1 "commits surfaced").
    assert_eq!(agent.steps.len(), 2);
    assert!(agent.steps[0].subject.contains("dispatch"));
    // The compaction is a `--no-ff` merge; `--first-parent` surfaces the
    // merge commit (its summary rides the second parent, §2.6).
    assert!(
        agent.steps[1].subject.contains("Merge"),
        "{:?}",
        agent.steps[1].subject
    );
    assert_eq!(agent.tip_short_oid.len(), 8);
    // No inbox and no mark refs by default.
    assert_eq!(agent.pending.len(), 0);
    assert!(agent.conflicted_oid.is_none());
    assert!(agent.budget_oid.is_none());
    assert!(agent.abandoned_oid.is_none());
    assert!(agent.notify_oid.is_none());
}

#[test]
fn from_repo_multiple_agents_appear() {
    let fx = Fixture::new();
    fx.build_agent("20260422T120000Z-old0", "first prompt");
    fx.build_agent("20260422T120500Z-new0", "second prompt");
    let tree = GitTree::from_repo(&fx.path).unwrap();
    assert_eq!(tree.commits.len(), 1);
    assert_eq!(tree.agents.len(), 2);
    let ids: Vec<&str> = tree.agents.iter().map(|a| a.agent_id.as_str()).collect();
    assert!(ids.contains(&"20260422T120000Z-old0"), "{ids:?}");
    assert!(ids.contains(&"20260422T120500Z-new0"), "{ids:?}");
}

#[test]
fn from_repo_surfaces_pending_message_count() {
    // §7.1 pending-message indicator: files in the agent's inbox count;
    // the atomic-rename temp dotfile is excluded.
    let fx = Fixture::new();
    fx.build_agent("20260422T121000Z-msg0", "with mail");
    fx.deposit_message("20260422T121000Z-msg0", "user-001.md", "hi");
    fx.deposit_message("20260422T121000Z-msg0", "p1-002.md", "steer");
    fx.deposit_message("20260422T121000Z-msg0", ".user-003.md.tmp", "partial");
    let tree = GitTree::from_repo(&fx.path).unwrap();
    assert_eq!(tree.agents[0].pending.len(), 2);
    // The listing is the §5.1 #11 parse itself, oldest-first by filename —
    // the same bytes every seat (badge, Inbox tab, inbox-composer) reads.
    assert_eq!(tree.agents[0].pending[0].name, "p1-002.md");
    assert_eq!(tree.agents[0].pending[1].name, "user-001.md");
    assert_eq!(tree.agents[0].pending[1].deposit.body, "hi");
}

#[test]
fn from_repo_surfaces_all_four_mark_oids() {
    // §2.6 / §6 / ARCH §8 ref-derived marks, keyed by raw agent id; each
    // surfaces its ref oid (the §6 watermark evidence), not a bare bool.
    let fx = Fixture::new();
    fx.build_agent("20260422T121500Z-mrk0", "marked");
    fx.mark_ref("refs/lernie/conflicted/20260422T121500Z-mrk0");
    fx.mark_ref("refs/lernie/budget-exhausted/20260422T121500Z-mrk0");
    fx.mark_ref("refs/lernie/abandoned/20260422T121500Z-mrk0");
    fx.mark_ref("refs/lernie/notify/20260422T121500Z-mrk0");
    let tree = GitTree::from_repo(&fx.path).unwrap();
    let agent = &tree.agents[0];
    // mark_ref points every mark at config/default, so all four oids are
    // the same non-empty 40-char sha — presence is what the marks encode.
    for oid in [
        &agent.conflicted_oid,
        &agent.budget_oid,
        &agent.abandoned_oid,
        &agent.notify_oid,
    ] {
        assert_eq!(oid.as_deref().map(str::len), Some(40));
    }
}

/// The fifth mark carries a **value** (ARCH §3.3, DESIGN §8.6): the parked
/// invocation, read off the blob the ref names — and a blob that is not the
/// shape lernie writes is no park at all, never a forged one.
#[test]
fn from_repo_reads_the_parked_invocation_off_the_hold_mark() {
    let fx = Fixture::new();
    fx.build_agent("20260422T123000Z-hld0", "parked");
    fx.build_agent("20260422T123000Z-hld1", "mangled");
    fx.hold_ref(
        "20260422T123000Z-hld0",
        r#"{"tool_use_id":"toolu_5","tool":"bash","reason":"open-world (no rule classifies `curl`)"}"#,
    );
    fx.hold_ref("20260422T123000Z-hld1", "not the shape lernie writes");
    let tree = GitTree::from_repo(&fx.path).unwrap();
    let parked = tree
        .agents
        .iter()
        .find(|a| a.agent_id == "20260422T123000Z-hld0")
        .unwrap();
    let held = parked.held.as_ref().expect("the park");
    assert_eq!(held.tool_use_id, "toolu_5");
    assert_eq!(held.tool, "bash");
    assert!(held.reason.contains("open-world"));
    assert!(parked.marks().contains(&crate::git_tree::AgentMark::Held));
    let mangled = tree
        .agents
        .iter()
        .find(|a| a.agent_id == "20260422T123000Z-hld1")
        .unwrap();
    assert_eq!(mangled.held, None);
    assert!(!mangled.marks().contains(&crate::git_tree::AgentMark::Held));
}

#[test]
fn from_repo_marks_only_the_named_agent() {
    // A mark on one agent does not bleed onto a sibling.
    let fx = Fixture::new();
    fx.build_agent("20260422T122000Z-aaa0", "marked");
    fx.build_agent("20260422T122000Z-bbb0", "clean");
    fx.mark_ref("refs/lernie/budget-exhausted/20260422T122000Z-aaa0");
    let tree = GitTree::from_repo(&fx.path).unwrap();
    let marked = tree
        .agents
        .iter()
        .find(|a| a.agent_id == "20260422T122000Z-aaa0")
        .unwrap();
    let clean = tree
        .agents
        .iter()
        .find(|a| a.agent_id == "20260422T122000Z-bbb0")
        .unwrap();
    assert!(marked.budget_oid.is_some());
    assert!(clean.budget_oid.is_none());
}

#[test]
fn from_repo_enumerates_a_child_agent_for_the_descent_tree() {
    // A child fork appears as its own agent row; the descent tree is
    // derived from the ids at render time (§2.3, §7.1).
    let fx = Fixture::new();
    fx.build_agent("20260422T123000Z-par0", "parent");
    fx.build_child("20260422T123000Z-par0", "20260422T123000Z-par0-c1");
    let tree = GitTree::from_repo(&fx.path).unwrap();
    let ids: Vec<&str> = tree.agents.iter().map(|a| a.agent_id.as_str()).collect();
    assert!(ids.contains(&"20260422T123000Z-par0"), "{ids:?}");
    assert!(ids.contains(&"20260422T123000Z-par0-c1"), "{ids:?}");
}
