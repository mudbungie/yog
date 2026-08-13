//! Table tests for the §11 live-activity derivation ([`super::super::flight`]):
//! each class from the facts that make it, the operator's priority when several
//! hold at once, and the two ways a conversation reads as at rest.

use super::*;
use crate::git_tree::ToolCallState;
use crate::nav::convs::Flight;
use crate::nav::convs::flight::{conversation_flight, flight};

/// An agent with one named tool call in `state` on its latest step.
fn with_tool(id: &str, agent_state: AgentState, tool: ToolCallState) -> Agent {
    named_tool(id, agent_state, tool, Some("Read"), Some(1))
}

/// `flight` over a subtree given root-first, as `members` hands it over.
fn of(agents: &[Agent]) -> Option<Flight> {
    let refs: Vec<&Agent> = agents.iter().collect();
    flight(&refs)
}

#[test]
fn inference_is_any_member_with_an_open_model_call() {
    // The root itself streaming.
    assert_eq!(
        of(&[agent("r-0", AgentState::InFlight, 1)]),
        Some(Flight::Inference)
    );
    // …or a child, under a settled root: the class is the conversation's, not
    // the root's (the §11 badge aggregates the same way).
    let via_child = [
        agent("r-0", AgentState::Quiescent, 1),
        agent("r-0-c-1", AgentState::InFlight, 2),
    ];
    assert_eq!(of(&via_child), Some(Flight::Inference));
}

#[test]
fn tools_is_a_landed_input_with_no_output_under_a_live_driver() {
    let live = [with_tool("r-0", AgentState::Live, ToolCallState::InFlight)];
    assert_eq!(of(&live), Some(Flight::Tools));
    // A completed call is not in flight — both files are on disk.
    let done = [with_tool("r-0", AgentState::Live, ToolCallState::Complete)];
    assert_eq!(of(&done), None);
}

#[test]
fn a_tool_record_left_by_a_dead_driver_is_not_in_flight() {
    // `output.json` never lands for a tool whose driver was killed mid-call, so
    // the record is in-flight forever. Requiring a running member is what stops
    // that conversation pulsing until someone deletes the workspace.
    let orphan = [with_tool(
        "r-0",
        AgentState::Stopped,
        ToolCallState::InFlight,
    )];
    assert_eq!(of(&orphan), None);
    let quiet = [with_tool(
        "r-0",
        AgentState::Quiescent,
        ToolCallState::InFlight,
    )];
    assert_eq!(of(&quiet), None);
}

#[test]
fn subagents_is_a_running_descendant_and_never_the_root_alone() {
    let child = [
        agent("r-0", AgentState::Quiescent, 1),
        agent("r-0-c-1", AgentState::Live, 2),
    ];
    assert_eq!(of(&child), Some(Flight::Subagents));
    // The root holding its own driver is not a subagent — it is the
    // conversation, and with nothing else in flight it shows nothing.
    let root_only = [agent("r-0", AgentState::Live, 1)];
    assert_eq!(of(&root_only), None);
}

#[test]
fn priority_is_inference_then_tools_then_subagents() {
    // All three hold at once: a streaming root, a tool running under a live
    // child, and that child is a dispatched descendant. One class shows.
    let mut child = with_tool("r-0-c-1", AgentState::Live, ToolCallState::InFlight);
    child.tip_timestamp_unix = 2;
    let all = [agent("r-0", AgentState::InFlight, 1), child.clone()];
    assert_eq!(of(&all), Some(Flight::Inference), "inference outranks both");
    // Drop the model call: tools outranks the subagent it runs under.
    let two = [agent("r-0", AgentState::Quiescent, 1), child];
    assert_eq!(of(&two), Some(Flight::Tools));
}

#[test]
fn a_settled_conversation_and_an_empty_one_are_both_at_rest() {
    let settled = [
        agent("r-0", AgentState::Quiescent, 1),
        agent("r-0-c-1", AgentState::Stopped, 2),
    ];
    assert_eq!(of(&settled), None, "nothing in flight, nothing to paint");
    assert_eq!(flight(&[]), None);
}

#[test]
fn the_row_carries_the_class_and_the_pane_reads_the_same_one() {
    // The two §11 seats are one derivation: the list row's field and the
    // altitude-1 pane's query answer alike over the same snapshot.
    let agents = [
        agent("r-0", AgentState::Quiescent, 1),
        agent("r-0-c-1", AgentState::InFlight, 2),
    ];
    let row = &build(&agents, "/ws", &unseen, 10, &plain, &[])[0];
    assert_eq!(row.flight, Some(Flight::Inference));
    assert_eq!(conversation_flight(&agents, "r-0"), Some(Flight::Inference));
    // An id that roots no conversation here has nothing in flight.
    assert_eq!(conversation_flight(&agents, "ghost"), None);
    // A settled conversation's row carries no class — the row that paints
    // nothing extra is the same row that schedules no repaint (§7.2).
    let quiet = [agent("q-0", AgentState::Quiescent, 1)];
    assert_eq!(
        build(&quiet, "/ws", &unseen, 10, &plain, &[])[0].flight,
        None
    );
}
