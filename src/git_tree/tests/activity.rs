//! The §11 recency fact (`Agent::last_action_unix`, bl-cad5): the newest of the
//! tip commit timestamp, the newest `messages/` entry mtime and the latest
//! step's `response.json` mtime. Proven end-to-end through
//! [`GitTree::from_repo`], because the whole point is that the fact is gathered
//! at snapshot time — a unit test of the fold alone would not show that.

use super::fixture::Fixture;
use super::mtime;
use crate::git_tree::{Agent, GitTree};
use std::fs;

fn agent_of(tree: &GitTree, id: &str) -> Agent {
    tree.agents
        .iter()
        .find(|a| a.agent_id == id)
        .expect("the enumerated agent")
        .clone()
}

/// Write one committed-transcript entry into `agents/<id>/messages/`.
fn write_message(fx: &Fixture, id: &str, name: &str) -> std::path::PathBuf {
    let dir = fx.path.join("agents").join(id).join("messages");
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join(name);
    fs::write(&path, "from a peer\n").unwrap();
    path
}

#[test]
fn a_delivered_message_is_an_action_even_though_it_commits_nothing() {
    // A message file lands by rename into `messages/` — no commit moves, so the
    // tip timestamp cannot see it. The recency fact must.
    let fx = Fixture::new();
    let id = "20260801T120000Z-msg";
    fx.build_agent(id, "wait for mail");
    let msg = write_message(&fx, id, "001-peer.md");
    let agent = agent_of(&GitTree::from_repo(&fx.path).unwrap(), id);
    assert_eq!(agent.last_action_unix, mtime(&msg));
    assert!(agent.last_action_unix >= agent.tip_timestamp_unix);
}

#[test]
fn the_live_streaming_tail_is_an_action() {
    // The harness rewrites `steps/<id>/<NNN>/response.json` as tokens arrive;
    // the step commits only when the turn ends. Mid-stream, the tail's mtime is
    // the only evidence the conversation is moving at all.
    let fx = Fixture::new();
    let id = "20260801T120100Z-strm";
    fx.build_agent(id, "start streaming");
    fx.write_response_events(
        id,
        2,
        &[r#"{"type":"content_delta","index":0,"delta":{"text_delta":"mid"}}"#],
    );
    let tail = fx.path.join("steps").join(id).join("002/response.json");
    let agent = agent_of(&GitTree::from_repo(&fx.path).unwrap(), id);
    assert_eq!(agent.last_action_unix, mtime(&tail));
    assert!(agent.last_action_unix >= agent.tip_timestamp_unix);
}

#[test]
fn the_newest_of_the_three_wins() {
    // Both off-commit signals present: the later one is the answer.
    let fx = Fixture::new();
    let id = "20260801T120200Z-both";
    fx.build_agent(id, "mail then tokens");
    write_message(&fx, id, "001-peer.md");
    fx.write_response_events(id, 1, &[r#"{"type":"end"}"#]);
    let tail = fx.path.join("steps").join(id).join("001/response.json");
    let agent = agent_of(&GitTree::from_repo(&fx.path).unwrap(), id);
    assert_eq!(agent.last_action_unix, mtime(&tail));
}

#[test]
fn with_nothing_but_commits_the_tip_is_the_last_action() {
    // Three empty cases at once: an agent whose only step wrote a request but
    // no response tail, an empty `messages/` directory, and (the child) an
    // agent with no `steps/` directory at all. Each contributes nothing, so the
    // tip stands — no special case, just a max over zeroes.
    let fx = Fixture::new();
    let id = "20260801T120300Z-quiet";
    fx.build_agent(id, "say nothing");
    fs::create_dir_all(fx.path.join("agents").join(id).join("messages")).unwrap();
    let child = format!("{id}-k-1");
    fx.build_child(id, &child);
    let tree = GitTree::from_repo(&fx.path).unwrap();
    let root = agent_of(&tree, id);
    assert_eq!(root.last_action_unix, root.tip_timestamp_unix);
    let kid = agent_of(&tree, &child);
    assert_eq!(kid.last_action_unix, kid.tip_timestamp_unix);
}

/// The landed-message **count** (§5.1 #12, bl-915e), off the same readdir the
/// recency fact above rides: what §7.2's pending echo reconciles against. Zero
/// for an agent with no transcript directory at all — the general path with no
/// inputs, which is what a just-written branch has.
#[test]
fn the_message_count_rides_the_same_directory_walk_as_the_recency_fact() {
    let fx = Fixture::new();
    let id = "20260801T120000Z-count";
    fx.build_agent(id, "count me");
    let bare = agent_of(&GitTree::from_repo(&fx.path).unwrap(), id);
    assert_eq!(bare.messages, 0, "no messages/ directory is no messages");

    write_message(&fx, id, "001-user.md");
    write_message(&fx, id, "002-peer.md");
    let two = agent_of(&GitTree::from_repo(&fx.path).unwrap(), id);
    assert_eq!(two.messages, 2);
    assert!(
        two.last_action_unix >= bare.last_action_unix,
        "and the recency fact still comes off the same walk"
    );
    assert!(!two.in_memory(), "a derived agent always has a tip");
}
