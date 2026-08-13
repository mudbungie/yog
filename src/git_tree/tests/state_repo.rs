//! End-to-end §3.5 agent-state classification against real workspace
//! fixtures.
//!
//! These fixtures run **no live executor**, so the executor-lock probe
//! finds no holder and the `response.json` writer probe finds the file
//! closed: every agent settles into a *quiescent* (clean terminal) or
//! *stopped* (failed / killed / no-run) classification (§4.4 terminal
//! rules). The `live` and `in_flight` states require a running driver
//! holding the lock and are covered by the unit tests in
//! `super::super::state` (probe-injected) and `super::super::lock_probe`.

use super::fixture::Fixture;
use crate::git_tree::{AgentState, GitTree};

#[test]
fn agent_with_no_response_yet_classifies_as_stopped() {
    // Branch exists, dispatch commit landed, but no `response.json` and no
    // live executor: nothing is driving it → stopped (§3.5, §2.9).
    let fx = Fixture::new();
    fx.commit_other("README.md", "initial");
    fx.build_agent("20260427T160000Z-pre0", "no response yet");
    let tree = GitTree::from_repo(&fx.path).unwrap();
    assert_eq!(tree.agents.len(), 1);
    assert_eq!(tree.agents[0].state, AgentState::Stopped);
}

#[test]
fn agent_with_partial_response_and_no_executor_classifies_as_stopped() {
    // Streaming began (text_delta on disk) but no terminal `end` landed and
    // no executor holds the lock: killed mid-stream → stopped (§4.4).
    let fx = Fixture::new();
    fx.commit_other("README.md", "initial");
    fx.build_agent("20260427T160100Z-mid0", "mid stream");
    fx.write_response_events(
        "20260427T160100Z-mid0",
        1,
        &[
            r#"{"type":"message_start","v":1,"role":"assistant"}"#,
            r#"{"type":"content_delta","index":0,"delta":{"text_delta":"hello"}}"#,
        ],
    );
    let tree = GitTree::from_repo(&fx.path).unwrap();
    assert_eq!(tree.agents[0].state, AgentState::Stopped);
}

#[test]
fn agent_with_clean_terminal_classifies_as_quiescent() {
    // A `finish` + `end` segment with the fd closed and no lock: a clean,
    // complete model call awaiting a message → quiescent (§3.5, §4.4).
    let fx = Fixture::new();
    fx.commit_other("README.md", "initial");
    fx.build_agent("20260427T160200Z-end0", "ended");
    fx.write_response_events(
        "20260427T160200Z-end0",
        1,
        &[
            r#"{"type":"message_start","v":1,"role":"assistant"}"#,
            r#"{"type":"content_delta","index":0,"delta":{"text_delta":"done"}}"#,
            r#"{"type":"finish","reason":"stop"}"#,
            r#"{"type":"end"}"#,
        ],
    );
    let tree = GitTree::from_repo(&fx.path).unwrap();
    assert_eq!(tree.agents[0].state, AgentState::Quiescent);
}

#[test]
fn agent_with_error_terminal_classifies_as_stopped() {
    // A failed attempt (error + terminal end) with the fd closed is a
    // *failed* step, rendered stopped (§4.4, §2.10).
    let fx = Fixture::new();
    fx.commit_other("README.md", "initial");
    fx.build_agent("20260427T160300Z-err0", "errored");
    fx.write_response_events(
        "20260427T160300Z-err0",
        1,
        &[
            r#"{"type":"message_start","v":1,"role":"assistant"}"#,
            r#"{"type":"error","kind":"provider","message":"oops"}"#,
            r#"{"type":"end"}"#,
        ],
    );
    let tree = GitTree::from_repo(&fx.path).unwrap();
    assert_eq!(tree.agents[0].state, AgentState::Stopped);
}
