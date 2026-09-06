//! The grant read, and the confinement it restores (bl-52b7).

use super::*;
use crate::test_support::workspace::{seed_agent_branch, seed_workspace_config};
use tempfile::TempDir;

/// `providers.yaml` as litany's own template authors it: the worker holds the
/// whole pool, the **compactor declares no `tools:` line at all**, and the
/// reviewer's grant is its confinement.
const TEMPLATE: &str = "roles:\n  worker:\n    provider: anthropic\n    model: m\n    \
     tools: [apply_patch, bash, read_file]\n  compactor:\n    provider: anthropic\n    \
     model: h\n  reviewer:\n    provider: anthropic\n    model: h\n    \
     tools: [apply_patch, read_file]\n";

/// The same file with the roster granted to the worker — the one operator edit
/// that lets a conversation drive its machines.
const WITH_CLIENTS: &str = "roles:\n  worker:\n    provider: anthropic\n    model: m\n    \
     tools: [apply_patch, bash, read_file, clients]\n  compactor:\n    provider: anthropic\n    \
     model: h\n";

fn workspace(providers: &str) -> TempDir {
    let dir = TempDir::new().expect("tmp");
    seed_workspace_config(dir.path(), &[("providers.yaml", providers)]);
    dir
}

/// **The fixture is the shape yog's own enumerator reads.** Both reads here
/// address `agents/<id>`, and a bare `<id>` would answer *worker* for every
/// role — every compactor silently reading the worker grant — while every
/// beat below still passed, because the fixture and the reader would be wrong
/// together. So the fixture is pinned against a *different* module's reader of
/// the same namespace (`refs/heads/agents/`), which is what a live workspace
/// shows.
#[test]
fn the_fixture_branch_is_the_namespace_the_engine_enumerates() {
    let ws = workspace(TEMPLATE);
    seed_agent_branch(ws.path(), "amber", None);
    let ids: Vec<String> = crate::git_tree::living_agents(ws.path())
        .into_iter()
        .map(|(id, _)| id)
        .collect();
    assert_eq!(ids, ["amber"]);
}

#[test]
fn a_root_agent_is_a_worker_and_reads_the_worker_grant() {
    let ws = workspace(TEMPLATE);
    seed_agent_branch(ws.path(), "amber", None);
    assert_eq!(of(ws.path(), "amber"), ["apply_patch", "bash", "read_file"]);
}

#[test]
fn a_child_reads_the_grant_of_the_role_its_dispatch_commit_names() {
    let ws = workspace(TEMPLATE);
    seed_agent_branch(ws.path(), "amber-1", Some("reviewer"));
    assert_eq!(of(ws.path(), "amber-1"), ["apply_patch", "read_file"]);
}

/// The ball's own case: the compactor's row declares no `tools:` line, so its
/// grant is empty — and an empty grant is empty.
#[test]
fn a_compactor_is_granted_nothing() {
    let ws = workspace(TEMPLATE);
    seed_agent_branch(ws.path(), "amber-2", Some("compactor"));
    assert!(of(ws.path(), "amber-2").is_empty());
}

/// Every unreadable fact answers "granted nothing", never "granted anything".
#[test]
fn everything_unreadable_is_an_empty_grant() {
    let nowhere = TempDir::new().expect("tmp");
    assert!(of(nowhere.path(), "amber").is_empty());

    // A workspace whose config declares no such role.
    let ws = workspace(TEMPLATE);
    seed_agent_branch(ws.path(), "amber-3", Some("auditor"));
    assert!(of(ws.path(), "amber-3").is_empty());

    // A `tools:` value that is not the flow sequence litany writes.
    let block = workspace("roles:\n  worker:\n    provider: p\n    model: m\n    tools:\n");
    seed_agent_branch(block.path(), "amber", None);
    assert!(of(block.path(), "amber").is_empty());

    // A providers.yaml the governing commit does not carry.
    let bare = TempDir::new().expect("tmp");
    seed_workspace_config(bare.path(), &[("workflow.yaml", "steps: []\n")]);
    seed_agent_branch(bare.path(), "amber", None);
    assert!(of(bare.path(), "amber").is_empty());
}

#[test]
fn the_operator_grants_the_roster_by_naming_it() {
    let ws = workspace(WITH_CLIENTS);
    seed_agent_branch(ws.path(), "amber", None);
    assert!(of(ws.path(), "amber").iter().any(|t| t == "clients"));
    seed_agent_branch(ws.path(), "amber-1", Some("compactor"));
    assert!(of(ws.path(), "amber-1").is_empty());
}

/// The subject parse, at its edges: only litany's own shape names a role.
#[test]
fn only_a_dispatch_subject_names_a_role() {
    assert_eq!(
        named("dispatch: reviewer [a-1]").as_deref(),
        Some("reviewer")
    );
    // The role may itself contain a bracket-free space; the tail decides.
    assert_eq!(named("dispatch: a b [a-1]").as_deref(), Some("a b"));
    assert_eq!(named("step 001: dispatch [a]"), None);
    assert_eq!(named("dispatch: [a]"), None);
    assert_eq!(named("dispatch: reviewer"), None);
    assert_eq!(named(""), None);
}
