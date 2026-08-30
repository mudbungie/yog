//! What a row is **called** and what it **previews**, read through
//! `GitTree::from_repo`: the §3.3 name fact off the agent's own branch (its
//! one home since bl-50f3), an empty name blob reading as unnamed, a descent
//! child answered by that same query, and the goal preview — the goal on disk
//! rather than the assembled context's pinned frame, and dropped when there is
//! no goal at all. Split from [`super::repo`] at §12's budget on the seam
//! between the tree's **skeleton** — which rows exist, what they are marked
//! with — and the text each row wears.

use super::fixture::Fixture;
use crate::git_tree::GitTree;

#[test]
fn from_repo_reads_the_litany_name_fact_off_the_branch() {
    // §3.3 as ruled by bl-50f3: the name's one home is the `name` blob on the
    // agent's own branch (lernie 0.0.4), read `git show agents/<id>:name`.
    let fx = Fixture::new();
    fx.build_agent("20260422T124000Z-nam0", "named work");
    fx.name_agent("20260422T124000Z-nam0", "pale-otter");
    fx.build_agent("20260422T124000Z-old0", "pre-0.0.4 root");
    let tree = GitTree::from_repo(&fx.path).unwrap();
    let named = tree
        .agents
        .iter()
        .find(|a| a.agent_id == "20260422T124000Z-nam0")
        .unwrap();
    let legacy = tree
        .agents
        .iter()
        .find(|a| a.agent_id == "20260422T124000Z-old0")
        .unwrap();
    assert_eq!(named.name.as_deref(), Some("pale-otter"));
    assert!(
        legacy.name.is_none(),
        "a branch with no blob (pre-0.0.4) reads None, never an error"
    );
}

#[test]
fn from_repo_reads_an_empty_name_blob_as_unnamed() {
    // litany writes the file on EVERY dispatch commit — empty means unnamed
    // (one shape, no absence-vs-empty split); yog folds both to None.
    let fx = Fixture::new();
    fx.build_agent("20260422T124500Z-emt0", "unnamed");
    fx.name_agent("20260422T124500Z-emt0", "");
    let tree = GitTree::from_repo(&fx.path).unwrap();
    assert!(tree.agents[0].name.is_none());
}

#[test]
fn from_repo_reads_a_descent_childs_name_by_the_same_query() {
    // §3.3's honest-scope limit is lifted (bl-08f2): a litany-dispatched child
    // may carry a name, and it surfaces through the identical read — no yog
    // special case.
    let fx = Fixture::new();
    fx.build_agent("20260422T125000Z-par0", "parent");
    fx.build_child("20260422T125000Z-par0", "20260422T125000Z-par0-c1");
    fx.name_agent("20260422T125000Z-par0-c1", "quiet-heron");
    let tree = GitTree::from_repo(&fx.path).unwrap();
    let child = tree
        .agents
        .iter()
        .find(|a| a.agent_id == "20260422T125000Z-par0-c1")
        .unwrap();
    assert_eq!(child.name.as_deref(), Some("quiet-heron"));
}

/// Two facts, two sources since bl-368d: the preview is the goal on disk, the
/// streaming text is the step record. Take the goal away and the preview goes
/// silent even though the step record is still sitting there.
#[test]
fn from_repo_agent_without_a_goal_on_disk_drops_preview() {
    let fx = Fixture::new();
    fx.build_agent("20260422T130000Z-xxxx", "seed");
    std::fs::remove_file(fx.path.join("agents/20260422T130000Z-xxxx/goal.md")).unwrap();
    let tree = GitTree::from_repo(&fx.path).unwrap();
    assert_eq!(tree.agents.len(), 1);
    assert!(tree.agents[0].preview.is_none());
    // And the step record's own fact is untouched by that: it holds a request
    // and no response, so the stream is silent for its own reason.
    assert!(tree.agents[0].stream.text.is_none());
}

/// The defect bl-368d closed. A root's **assembled context** opens with the
/// §3.7 pinned-instruction frame and wraps a deposit in its `---` envelope, so
/// a preview read off `request.json` showed `<file path="instructions/…">`
/// where the operator's words belong. The payload's home is `goal.md` and that
/// is what the row reads — the assembled request is written here, carrying the
/// frame, and contributes nothing.
#[test]
fn from_repo_previews_the_goal_not_the_assembled_contexts_pinned_frame() {
    let fx = Fixture::new();
    let id = "20260422T131500Z-frm0";
    fx.build_agent(id, "seed");
    fx.write_goal(
        id,
        "You are slate-newt.\n\nunbar the postern\n\nthe body runs on",
    );
    fx.write_assembled_request(id);
    let tree = GitTree::from_repo(&fx.path).unwrap();
    assert_eq!(
        tree.agents[0].preview.as_deref(),
        Some("unbar the postern"),
        "the operator's headline, not the frame's opening tag"
    );
}
