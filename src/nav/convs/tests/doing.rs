//! The per-agent §5.1 #28b fact: the five states, and the mark's seat roster
//! built from them.

use super::*;
use crate::git_tree::{Delta, ToolCallState};
use crate::nav::convs::{Doing, doing, seats};

/// An agent mid-model-call whose stream has produced `delta` so far.
fn streaming(id: &str, delta: Option<Delta>) -> Agent {
    let mut a = agent(id, AgentState::InFlight, 1);
    a.stream.last_delta = delta;
    a
}

/// The three states of an open model call — the split the last content delta
/// buys. Nothing back yet is *waiting*, and it is the empty stream's own
/// reading, never a case.
#[test]
fn an_open_call_splits_three_ways_on_its_last_delta() {
    assert_eq!(doing(&streaming("a", None)), Doing::Waiting);
    assert_eq!(
        doing(&streaming("a", Some(Delta::Thinking))),
        Doing::Thinking
    );
    assert_eq!(doing(&streaming("a", Some(Delta::Text))), Doing::Inference);
    // All three are the one §5.1 #28 `Inference` class, which is what lets
    // `flight` fold over this instead of asking the state again.
    for delta in [None, Some(Delta::Thinking), Some(Delta::Text)] {
        assert!(doing(&streaming("a", delta)).is_model_call());
    }
}

/// A running tool under a live driver is `Tools`; the same record under a
/// **dead** driver is not. `output.json` never lands for a tool whose driver
/// died, so an unguarded reading would light that seat forever.
#[test]
fn a_tool_counts_only_under_a_driver_that_could_have_started_it() {
    let live = named_tool("a", AgentState::Live, ToolCallState::InFlight, None, None);
    assert_eq!(doing(&live), Doing::Tools);
    let dead = named_tool(
        "a",
        AgentState::Stopped,
        ToolCallState::InFlight,
        None,
        None,
    );
    assert_eq!(doing(&dead), Doing::Idle);
    let done = named_tool("a", AgentState::Live, ToolCallState::Complete, None, None);
    assert_eq!(doing(&done), Doing::Idle);
}

/// An open model call outranks a running tool — §5.1 #28's priority, read off
/// this one derivation rather than decided a second time.
#[test]
fn an_open_call_outranks_a_running_tool() {
    let mut both = named_tool(
        "a",
        AgentState::InFlight,
        ToolCallState::InFlight,
        None,
        None,
    );
    both.stream.last_delta = Some(Delta::Text);
    assert_eq!(doing(&both), Doing::Inference);
}

/// Everything settled is idle — and the mark says nothing more about it. A
/// quiescent agent awaiting a message and a killed one both read the same
/// green, because "did this branch end well" is a different question with its
/// own carriers.
#[test]
fn every_settled_state_is_one_idle() {
    for state in [AgentState::Quiescent, AgentState::Stopped] {
        assert_eq!(doing(&agent("a", state, 1)), Doing::Idle);
        assert!(!doing(&agent("a", state, 1)).is_model_call());
    }
    assert_eq!(doing(&agent("a", AgentState::Live, 1)), Doing::Idle);
}

/// The mark's seats are the eye first, then the subtree in §2.3 descent order
/// — each named as every other seat names an agent (§3.3).
#[test]
fn seats_are_the_root_then_its_descent() {
    let mut root = agent("20260427T120000Z-aaaa", AgentState::InFlight, 3);
    root.name = Some("energize".into());
    root.stream.last_delta = Some(Delta::Thinking);
    let kid = named_tool(
        "20260427T120000Z-aaaa-20260427T120100Z-bbbb",
        AgentState::Live,
        ToolCallState::InFlight,
        None,
        None,
    );
    let agents = vec![root, kid];
    let laid = seats(&agents, "20260427T120000Z-aaaa");
    assert_eq!(laid.len(), 2);
    assert_eq!(laid.first().map(|s| s.doing), Some(Doing::Thinking));
    assert_eq!(
        laid.first().map(|s| s.name.clone()),
        Some("energize".to_owned())
    );
    assert_eq!(laid.get(1).map(|s| s.doing), Some(Doing::Tools));
}

/// An id that roots nothing seats nobody — which is the mark at rest, not an
/// error and not a branch.
#[test]
fn an_id_that_roots_nothing_seats_nobody() {
    assert!(seats(&[], "no-such-root").is_empty());
}
