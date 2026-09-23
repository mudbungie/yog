//! The §9.4 workflow-mark read (bl-b680) over real-git workspaces: every arm
//! of the nearest-mark walk and of the lineage recovery, because the port must
//! agree with litany's `nearest_mark` about who holds the mark.

use super::{WorkflowMark, read};
use crate::config_edit::branch::config_branches;
use crate::git_tree::tests::fixture::Fixture;
use crate::git_tree::tests::git::run_git;

const ROOT: &str = "20260101T000000Z-r1";
const CHILD: &str = "20260101T000000Z-r1-20260101T000001Z-c1";

/// Point `refs/litany/workflow/<id>` at `commit`, as litany's `--config`
/// write does — the ref names a commit, never a lineage.
fn mark(fx: &Fixture, id: &str, commit: &str) {
    run_git(
        &fx.path.join("repo.git"),
        &["update-ref", &format!("refs/litany/workflow/{id}"), commit],
    );
}

fn tip_of(fx: &Fixture, name: &str) -> (String, String) {
    let b = config_branches(&fx.path)
        .unwrap()
        .into_iter()
        .find(|b| b.name == name)
        .unwrap();
    (b.tip_oid, b.tip_short_oid)
}

#[test]
fn an_unmarked_descent_reads_none_the_ordinary_state() {
    let fx = Fixture::new();
    fx.agent_off(ROOT, "config/default");
    assert_eq!(read(&fx.path, ROOT).unwrap(), None);
}

#[test]
fn the_agents_own_mark_answers_with_the_lineage_standing_on_the_commit() {
    let fx = Fixture::new();
    fx.agent_off(ROOT, "config/default");
    fx.config_off("strict", "config/default");
    let (oid, short_oid) = tip_of(&fx, "strict");
    mark(&fx, ROOT, &oid);
    assert_eq!(
        read(&fx.path, ROOT).unwrap(),
        Some(WorkflowMark {
            holder: ROOT.to_owned(),
            oid,
            short_oid,
            lineage: Some("strict".to_owned()),
        })
    );
}

/// Marking a root switches its whole tree: a child with no mark of its own
/// inherits the ancestor's, and the answer names the **holder** — the fact an
/// operator cannot derive from the agent they asked about.
#[test]
fn an_ancestors_mark_is_inherited_and_names_its_holder() {
    let fx = Fixture::new();
    fx.agent_off(ROOT, "config/default");
    fx.agent_off(CHILD, "config/default");
    mark(&fx, ROOT, &tip_of(&fx, "default").0);
    let found = read(&fx.path, CHILD).unwrap().unwrap();
    assert_eq!(found.holder, ROOT);
    assert_eq!(found.lineage.as_deref(), Some("default"));
}

/// Nearest wins: the child's own mark overrides the root's.
#[test]
fn the_nearest_mark_on_the_descent_wins() {
    let fx = Fixture::new();
    fx.agent_off(ROOT, "config/default");
    fx.agent_off(CHILD, "config/default");
    fx.config_off("strict", "config/default");
    let (strict, _) = tip_of(&fx, "strict");
    mark(&fx, ROOT, &tip_of(&fx, "default").0);
    mark(&fx, CHILD, &strict);
    let found = read(&fx.path, CHILD).unwrap().unwrap();
    assert_eq!(found.holder, CHILD);
    assert_eq!(found.oid, strict);
}

/// A mark pins a commit; when its lineage advances past it, no `config/*`
/// ref stands on the marked commit any more and the lineage is the absence
/// it is — rendered, never declined.
#[test]
fn a_mark_the_lineage_advanced_past_names_no_lineage() {
    let fx = Fixture::new();
    fx.agent_off(ROOT, "config/default");
    let (pinned, _) = tip_of(&fx, "default");
    mark(&fx, ROOT, &pinned);
    fx.commit_other("workflow.yaml", "events: {}\n");
    let found = read(&fx.path, ROOT).unwrap().unwrap();
    assert_eq!(found.oid, pinned);
    assert_eq!(found.lineage, None);
}

/// litany's rule, ported: an unreadable mark is no mark. The workspace that
/// cannot answer is refused by the governing derivation beside this one.
#[test]
fn a_workspace_with_no_repo_reads_as_unmarked() {
    let dir = tempfile::tempdir().unwrap();
    assert_eq!(read(dir.path(), ROOT).unwrap(), None);
}
