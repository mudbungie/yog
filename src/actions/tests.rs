//! The §8.2 conversation gates the §8.5 agent answer carries: Stop, its
//! children cascade, and the nudge that is Stop's complement. Every one of them
//! reads the focused workspace's agent set — what a conversation is currently
//! doing — which is the only shape of gate this crate still holds (bl-33e9: a
//! gate derivable from a row the boundary already answers is the seat's).

use super::*;
use crate::git_tree::{Agent, AgentState};

fn branch(name: &str, state: AgentState) -> Agent {
    Agent {
        branch_name: format!("agents/{name}"),
        agent_id: name.to_string(),
        tip_oid: "0".repeat(40),
        tip_short_oid: "0000000".to_string(),
        tip_timestamp_unix: 0,
        last_action_unix: 0,
        messages: 0,
        steps: vec![],
        preview: None,
        stream: crate::git_tree::Stream::default(),
        tool_calls: vec![],
        state,
        state_uncertain: false,
        truncated: false,
        failure: None,
        flagged: None,
        pending: vec![],
        conflicted_oid: None,
        budget_oid: None,
        abandoned_oid: None,
        notify_oid: None,
        held: None,
        goal_ball: None,
        name: None,
        goal_name: None,
        call_start_unix: None,
    }
}

#[test]
fn stop_disabled_when_selection_is_none() {
    let bs = vec![branch("foo", AgentState::InFlight)];
    assert!(!stop_enabled(None, &bs));
}

#[test]
fn stop_disabled_when_selection_not_in_agents() {
    let bs = vec![branch("foo", AgentState::InFlight)];
    assert!(!stop_enabled(Some("bar"), &bs));
}

#[test]
fn stop_disabled_when_selected_agent_stopped() {
    let bs = vec![branch("foo", AgentState::Stopped)];
    assert!(!stop_enabled(Some("foo"), &bs));
}

#[test]
fn stop_disabled_when_selected_agent_quiescent() {
    // A finished-for-now agent has no executor to signal (§2.9).
    let bs = vec![branch("foo", AgentState::Quiescent)];
    assert!(!stop_enabled(Some("foo"), &bs));
}

#[test]
fn stop_enabled_when_selection_is_in_flight() {
    let bs = vec![branch("foo", AgentState::InFlight)];
    assert!(stop_enabled(Some("foo"), &bs));
}

#[test]
fn stop_enabled_when_selection_is_live() {
    // A driver between model calls (running a tool) is stoppable — the
    // very case §2.9's lock-fd discovery exists for.
    let bs = vec![branch("foo", AgentState::Live)];
    assert!(stop_enabled(Some("foo"), &bs));
}

#[test]
fn stop_disabled_when_agents_empty() {
    let bs: Vec<Agent> = vec![];
    assert!(!stop_enabled(Some("foo"), &bs));
}

#[test]
fn stop_picks_correct_agent_among_several() {
    let bs = vec![
        branch("a", AgentState::Stopped),
        branch("b", AgentState::InFlight),
        branch("c", AgentState::Quiescent),
        branch("d", AgentState::Live),
    ];
    assert!(stop_enabled(Some("b"), &bs));
    assert!(stop_enabled(Some("d"), &bs));
    assert!(!stop_enabled(Some("a"), &bs));
    assert!(!stop_enabled(Some("c"), &bs));
}

/// The nudge and Stop partition the four states between them (bl-9bef): every
/// agent is offered exactly one of the two, so neither is ever a control that
/// fires and does nothing (QUALITY H4). The one exception is
/// [`a_truncated_turn_is_not_nudgeable`], where the partition would put the
/// nudge on a shape litany answers with nothing.
#[test]
fn nudge_is_offered_exactly_where_stop_is_not() {
    let bs = vec![
        branch("a", AgentState::Stopped),
        branch("b", AgentState::InFlight),
        branch("c", AgentState::Quiescent),
        branch("d", AgentState::Live),
    ];
    for id in ["a", "b", "c", "d"] {
        assert_ne!(
            nudge_enabled(Some(id), &bs),
            stop_enabled(Some(id), &bs),
            "{id} is offered both or neither"
        );
    }
    assert!(
        nudge_enabled(Some("a"), &bs),
        "a stopped turn re-dispatches"
    );
    assert!(nudge_enabled(Some("c"), &bs), "a quiescent one continues");
    assert!(!nudge_enabled(None, &bs), "no selection");
    assert!(!nudge_enabled(Some("zz"), &bs), "absent id");
    assert!(!nudge_enabled(Some("a"), &[]), "no agents at all");
}

/// bl-fb87: a turn the output limit cut off leaves an assistant-side tail with
/// no `tool_use`, which linked litany's `advance` reads as `Warrant::NothingDue`
/// — it releases the lease and exits without creating a step. So the partition
/// above gives way here rather than offering a control that fires and does
/// nothing; Message, the recovery, is ungated and unaffected.
#[test]
fn a_truncated_turn_is_not_nudgeable() {
    let mut cut_off = branch("a", AgentState::Stopped);
    cut_off.truncated = true;
    let mut settled_at_rest = branch("b", AgentState::Quiescent);
    settled_at_rest.truncated = true;
    let bs = vec![cut_off, settled_at_rest, branch("c", AgentState::Stopped)];
    assert!(!nudge_enabled(Some("a"), &bs), "stopped and cut off");
    assert!(!nudge_enabled(Some("b"), &bs), "quiescent and cut off");
    assert!(
        nudge_enabled(Some("c"), &bs),
        "an ordinary resting conversation is unaffected"
    );
    // Neither is Stop offered — the conversation holds no driver — so this is
    // the one shape with no §8.2 conversation verb but Message, whose gate is
    // the seat's (bl-7cc8).
    assert!(!stop_enabled(Some("a"), &bs));
}

#[test]
fn stop_children_offered_only_with_a_descendant() {
    let bs = vec![
        branch("root-x", AgentState::Live),
        branch("root-x-c1", AgentState::Stopped),
        branch("root-y", AgentState::Live),
    ];
    assert!(stop_children_offered("root-x", &bs), "has a child");
    assert!(!stop_children_offered("root-y", &bs), "leaf agent");
    // A hyphen-boundary miss: root-xx is not a descendant of root-x.
    let bs2 = vec![
        branch("root-x", AgentState::Live),
        branch("root-xx", AgentState::Live),
    ];
    assert!(!stop_children_offered("root-x", &bs2));
}
