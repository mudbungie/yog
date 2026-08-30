//! The two **structural starts** the §11 in-flight strip times its elapsed
//! against (§5.1 #28, bl-9dfb): `Agent::call_start_unix` off the latest step's
//! `request.json` and `ToolCall::start_unix` off the call's `input.json`. Proven
//! end-to-end through [`GitTree::from_repo`], because the whole point is that
//! both are gathered at snapshot time — a unit test of the readers alone would
//! not show that the render path never stats.
//!
//! Why these files and not the tip commit, verified against the pinned litany
//! (`=0.0.3`, `src/prompt/dispatch`): both drivers — `run_exchange`'s loop and
//! the `litany advance` hop — land `request.json`, then take the timestamp they
//! will later write as `meta.json`'s `started_at`, then invoke the adapter. The
//! stamp and litany's own notion of the step's start are therefore the same
//! instant, and `meta.json` itself is useless here because it lands only *after*
//! the call returns. The executor does the same for a tool: `input.json` is
//! written atomically, then `started_at` is taken, then the tool is spawned.

use super::fixture::Fixture;
use super::mtime;
use crate::git_tree::{Agent, GitTree};

fn agent_of(tree: &GitTree, id: &str) -> Agent {
    tree.agents
        .iter()
        .find(|a| a.agent_id == id)
        .expect("the enumerated agent")
        .clone()
}

#[test]
fn the_model_call_start_is_the_latest_steps_request_stamp() {
    // Two steps on disk: the start is the *latest* one's, because that is the
    // call in flight. An earlier step's request stamps a call that has ended.
    let fx = Fixture::new();
    let id = "20260802T120000Z-call";
    fx.build_agent(id, "first ask");
    let latest = fx.write_request(id, 2, "second ask");
    let agent = agent_of(&GitTree::from_repo(&fx.path).unwrap(), id);
    assert_eq!(agent.call_start_unix, Some(mtime(&latest)));
}

#[test]
fn a_latest_step_with_no_request_yet_has_no_start() {
    // The executor creates the step dir and opens `response.json` around the
    // request write, so yog can catch a step mid-landing. It reads the latest
    // step and nothing else — falling back to step 001's stamp would date the
    // live call by a call that finished — so the answer is absence, and the
    // strip omits the segment.
    let fx = Fixture::new();
    let id = "20260802T120100Z-mid";
    fx.build_agent(id, "first ask");
    fx.write_response_events(id, 2, &[r#"{"type":"message_start"}"#]);
    let agent = agent_of(&GitTree::from_repo(&fx.path).unwrap(), id);
    assert_eq!(agent.call_start_unix, None);
}

#[test]
fn an_agent_with_no_step_tree_has_no_start() {
    // A freshly forked child has a branch and a worktree but has not stepped:
    // the general path with empty inputs, not a bootstrap case.
    let fx = Fixture::new();
    let id = "20260802T120200Z-fresh";
    fx.build_agent(id, "dispatch a child");
    let child = format!("{id}-k-1");
    fx.build_child(id, &child);
    let tree = GitTree::from_repo(&fx.path).unwrap();
    assert_eq!(agent_of(&tree, &child).call_start_unix, None);
    // …and the parent, which did step, still carries one.
    assert!(agent_of(&tree, id).call_start_unix.is_some());
}

#[test]
fn a_tool_calls_start_is_its_own_input_records_stamp() {
    // The stamp rides beside the name and the state, off the one record that
    // already decides them: per call, not per step, so two calls in one step
    // are timed apart.
    let fx = Fixture::new();
    let id = "20260802T120300Z-tools";
    fx.build_agent(id, "run two tools");
    let done = fx.write_tool_call(id, 1, "toolu_done", Some(b"{}"));
    let live = fx.write_tool_call(id, 1, "toolu_live", None);
    let agent = agent_of(&GitTree::from_repo(&fx.path).unwrap(), id);
    let calls = &agent.tool_calls;
    assert_eq!(calls.len(), 2);
    assert_eq!(calls[0].start_unix, Some(mtime(&done)));
    assert_eq!(calls[1].start_unix, Some(mtime(&live)));
}
