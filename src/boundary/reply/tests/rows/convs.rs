//! The conversation row's own table: every optional fact written only when it
//! is present, the display-only name that is shown but never handed over as a
//! message target, and the two token mappings that must stay total.

use super::super::super::rows::{flight_token, state_token};
use super::super::super::{Reply, encode};
use crate::git_tree::AgentState;
use crate::nav::convs::ConvBall;
use crate::nav::convs::{ConvRow, Flight};
use crate::projects::join::JoinState;

#[test]
fn conversation_rows_encode_their_optional_facts_only_when_present() {
    let full = ConvRow {
        root_id: "c-1".into(),
        state: AgentState::InFlight,
        uncertain: true,
        preview: "first line".into(),
        age_secs: 42,
        flight: Some(Flight::Inference),
        attention: 1,
        members: 3,
        depth: 0,
        direct: 2,
        stoppable: false,
        stop_children: false,
        ball: Some(ConvBall {
            id: "bl-7".into(),
            state: Some(JoinState::Bound),
            title: Some("t".into()),
            badge: Some("closed".into()),
        }),
        name: Some("brave-fox".into()),
        name_display_only: false,
        verdict: Some(crate::monitor::Check {
            workspace: "/ws".into(),
            agent: "c-1".into(),
            verdict: crate::monitor::Verdict::Drifting,
            sha: "abc123".into(),
            reason: "wandering".into(),
            model: "haiku".into(),
            input_tokens: Some(10),
            output_tokens: Some(2),
        }),
        tone: crate::nav::convs::Tone::Plain,
    };
    let bare = ConvRow {
        root_id: "c-2".into(),
        state: AgentState::Quiescent,
        uncertain: false,
        preview: String::new(),
        age_secs: 0,
        flight: None,
        attention: 0,
        members: 1,
        depth: 0,
        direct: 0,
        stoppable: false,
        stop_children: false,
        ball: Some(ConvBall {
            id: "stray".into(),
            state: None,
            title: None,
            badge: None,
        }),
        name: None,
        name_display_only: false,
        verdict: None,
        tone: crate::nav::convs::Tone::Plain,
    };
    let plain = ConvRow {
        ball: None,
        ..bare.clone()
    };
    let v = encode(&Reply::Conversations(vec![full, bare, plain]));
    let rows = v["rows"].as_array().unwrap();
    assert_eq!(rows[0]["state"], "in-flight");
    assert_eq!(rows[0]["flight"], "inference");
    assert_eq!(rows[0]["name"], "brave-fox");
    assert_eq!(rows[0]["display"], "brave-fox");
    assert_eq!(rows[0]["members"], 3);
    assert_eq!(
        rows[0]["direct"], 2,
        "the strict child count rides the answer"
    );
    assert_eq!(rows[1]["direct"], 0);
    assert_eq!(rows[0]["ball"]["state"], "bound");
    assert_eq!(rows[0]["ball"]["badge"], "closed");
    assert_eq!(rows[1]["state"], "quiescent");
    assert!(rows[1].get("flight").is_none());
    assert!(rows[1].get("name").is_none());
    assert_eq!(rows[1]["ball"]["id"], "stray");
    assert!(rows[1]["ball"].get("state").is_none());
    assert!(rows[2].get("ball").is_none());
}

#[test]
fn a_display_only_name_is_withheld_from_the_boundary_as_a_message_target() {
    // bl-8068: `name` is what a peer feeds `message`, and litany resolves by
    // exact id else unique *stored* name. A legacy-rung title has no stored
    // fact behind it, so handing it over would hand over a target litany
    // refuses (`no agent "marbling-lake" in this workspace`) — the very
    // failure this ball diagnosed. `display` still says what the row is
    // called, and `root_id` is the address that works.
    let legacy = ConvRow {
        root_id: "20260802T215937Z-2de238fa".into(),
        state: AgentState::Quiescent,
        uncertain: false,
        preview: "first line".into(),
        age_secs: 0,
        flight: None,
        attention: 0,
        members: 1,
        depth: 0,
        direct: 0,
        stoppable: false,
        stop_children: false,
        ball: None,
        name: Some("marbling-lake".into()),
        name_display_only: true,
        verdict: None,
        tone: crate::nav::convs::Tone::Plain,
    };
    let addressable = ConvRow {
        name_display_only: false,
        ..legacy.clone()
    };
    let v = encode(&Reply::Conversations(vec![legacy, addressable]));
    let rows = v["rows"].as_array().unwrap();
    assert!(
        rows[0].get("name").is_none(),
        "a display-only name is not a message target"
    );
    assert_eq!(rows[0]["display"], "marbling-lake", "it is still shown");
    assert_eq!(rows[0]["root_id"], "20260802T215937Z-2de238fa");
    assert_eq!(
        rows[1]["name"], "marbling-lake",
        "a stored fact is a target"
    );
}

#[test]
fn the_agent_state_and_flight_tokens_are_total() {
    for (state, token) in [
        (AgentState::Live, "live"),
        (AgentState::InFlight, "in-flight"),
        (AgentState::Quiescent, "quiescent"),
        (AgentState::Stopped, "stopped"),
    ] {
        assert_eq!(state_token(state), token);
    }
    for (flight, token) in [
        (Flight::Inference, "inference"),
        (Flight::Tools, "tools"),
        (Flight::Subagents, "subagents"),
    ] {
        assert_eq!(flight_token(flight), token);
    }
}
