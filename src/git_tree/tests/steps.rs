//! `GitTree::from_repo`'s step-content projection: the §4.4 stream events an
//! agent's `steps/<id>/<NNN>/response.json` carries, folded into
//! `Agent::stream` and the tool-call roster. The repo *skeleton* those steps
//! hang from is [`super::repo`]'s concern.

use super::fixture::Fixture;
use crate::git_tree::{GitTree, ToolCallState};

#[test]
fn from_repo_surfaces_partial_response_text() {
    // Live-streaming text view-model. The harness writes
    // `<workspace>/steps/<agent-id>/<NNN>/response.json` as JSONL of §4.4
    // stream events while the model produces output; the frontend reads it
    // on every tick and folds `text_delta` events into `Agent::stream`.
    let fx = Fixture::new();
    fx.build_agent("20260427T120100Z-strm", "summarize Rust ownership");
    fx.write_response_events(
        "20260427T120100Z-strm",
        1,
        &[
            r#"{"type":"message_start","message":{"id":"m1"}}"#,
            r#"{"type":"content_block_start","index":0,"content_block":{"type":"text","text":""}}"#,
            r#"{"type":"content_delta","index":0,"delta":{"text_delta":"Rust"}}"#,
            r#"{"type":"content_delta","index":0,"delta":{"text_delta":" tracks"}}"#,
            r#"{"type":"content_delta","index":0,"delta":{"text_delta":" ownership"}}"#,
        ],
    );
    let tree = GitTree::from_repo(&fx.path).unwrap();
    assert_eq!(tree.agents.len(), 1);
    assert_eq!(
        tree.agents[0].stream.text.as_deref(),
        Some("Rust tracks ownership")
    );
}

#[test]
fn from_repo_picks_latest_step_response_text() {
    // Multi-step loop: step 001 has a complete response, step 002 is
    // mid-stream. Streaming text reflects the latest step only.
    let fx = Fixture::new();
    fx.build_agent("20260427T120200Z-loop", "step into the loop");
    fx.write_response_events(
        "20260427T120200Z-loop",
        1,
        &[r#"{"type":"content_delta","index":0,"delta":{"text_delta":"first step body"}}"#],
    );
    fx.write_response_events(
        "20260427T120200Z-loop",
        2,
        &[r#"{"type":"content_delta","index":0,"delta":{"text_delta":"second step partial"}}"#],
    );
    let tree = GitTree::from_repo(&fx.path).unwrap();
    assert_eq!(
        tree.agents[0].stream.text.as_deref(),
        Some("second step partial")
    );
}

#[test]
fn from_repo_with_response_but_no_text_deltas_yet() {
    // Response file exists but only `message_start` has landed — no
    // text_delta events yet. The view-model should still be `None`.
    let fx = Fixture::new();
    fx.build_agent("20260427T120300Z-prep", "still preparing");
    fx.write_response_events(
        "20260427T120300Z-prep",
        1,
        &[r#"{"type":"message_start","message":{"id":"m1"}}"#],
    );
    let tree = GitTree::from_repo(&fx.path).unwrap();
    assert!(tree.agents[0].stream.text.is_none());
}

#[test]
fn from_repo_surfaces_in_flight_and_complete_tool_calls() {
    // Pulsing tool indicators. Latest step's tools/<id>/ dir with
    // input.json + no output.json is in-flight; both files present is
    // complete. Detection is filesystem-only (ARCH §3.3, §3.5).
    let fx = Fixture::new();
    fx.build_agent("20260427T140000Z-tool", "run two tools");
    fx.write_tool_call("20260427T140000Z-tool", 1, "toolu_done", Some(b"{}"));
    fx.write_tool_call("20260427T140000Z-tool", 1, "toolu_live", None);
    let tree = GitTree::from_repo(&fx.path).unwrap();
    let calls = &tree.agents[0].tool_calls;
    assert_eq!(calls.len(), 2);
    // Sorted by tool_id: "toolu_done" < "toolu_live".
    assert_eq!(calls[0].tool_id, "toolu_done");
    assert_eq!(calls[0].state, ToolCallState::Complete);
    assert_eq!(calls[1].tool_id, "toolu_live");
    assert_eq!(calls[1].state, ToolCallState::InFlight);
}

#[test]
fn from_repo_without_tool_calls_yields_empty_vec() {
    // No tools/ dir on disk — agent surfaces but tool_calls is empty.
    let fx = Fixture::new();
    fx.build_agent("20260427T140100Z-bare", "no tools yet");
    let tree = GitTree::from_repo(&fx.path).unwrap();
    assert!(tree.agents[0].tool_calls.is_empty());
}
