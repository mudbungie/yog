//! The row encoders' tables ([`rows`](super::super::rows)): every field a
//! workspace, conversation, join or ops row carries, and the two token
//! mappings that must stay total.

use super::super::rows::{flight_token, state_token};
use super::super::{Reply, WsRow, encode};
use crate::git_tree::AgentState;
use crate::nav::convs::ConvBall;
use crate::nav::convs::{ConvRow, Flight};
use crate::opslog::OpRow;
use crate::opslog::Origin;
use crate::projects::join::JoinRow;
use crate::projects::join::JoinState;
use std::path::PathBuf;

#[test]
fn workspace_rows_carry_the_classification_and_rollups() {
    use crate::binding::{Workspace, WorkspaceKind};
    let rows = vec![
        WsRow {
            workspace: Workspace {
                path: PathBuf::from("/n/alba"),
                kind: WorkspaceKind::Named {
                    name: "alba".into(),
                },
            },
            attention: 2,
            agents: 5,
            running: true,
        },
        WsRow {
            workspace: Workspace {
                path: PathBuf::from("/f"),
                kind: WorkspaceKind::Foreign,
            },
            attention: 0,
            agents: 0,
            running: false,
        },
        WsRow {
            workspace: Workspace {
                path: PathBuf::from("/r"),
                kind: WorkspaceKind::Replay,
            },
            attention: 0,
            agents: 1,
            running: false,
        },
    ];
    let v = encode(&Reply::Workspaces(rows));
    assert_eq!(v["kind"], "workspaces");
    let rows = v["rows"].as_array().unwrap();
    assert_eq!(rows[0]["kind"], "named");
    assert_eq!(rows[0]["name"], "alba");
    assert_eq!(rows[0]["attention"], 2);
    assert_eq!(rows[0]["running"], true);
    assert_eq!(rows[1]["kind"], "foreign");
    assert!(rows[1].get("name").is_none(), "no name to claim");
    assert_eq!(rows[2]["kind"], "replay");
}

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
        tone: crate::transcript::Tone::Plain,
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
        ball: Some(ConvBall {
            id: "stray".into(),
            state: None,
            title: None,
            badge: None,
        }),
        name: None,
        name_display_only: false,
        verdict: None,
        tone: crate::transcript::Tone::Plain,
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
    // bl-8068: `name` is what a peer feeds `message`, and lernie resolves by
    // exact id else unique *stored* name. A legacy-rung title has no stored
    // fact behind it, so handing it over would hand over a target lernie
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
        ball: None,
        name: Some("marbling-lake".into()),
        name_display_only: true,
        verdict: None,
        tone: crate::transcript::Tone::Plain,
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

#[test]
fn join_rows_encode_the_binding_facts() {
    let full = JoinRow {
        project: PathBuf::from("/p"),
        ball_id: "bl-1".into(),
        state: JoinState::Delivered,
        workspace: Some(PathBuf::from("/ws")),
        claimant: Some("alba".into()),
        title: Some("t".into()),
    };
    let bare = JoinRow {
        project: PathBuf::from("/p"),
        ball_id: "bl-2".into(),
        state: JoinState::ReadyStartable,
        workspace: None,
        claimant: None,
        title: None,
    };
    let v = encode(&Reply::Balls(vec![full, bare]));
    let rows = v["rows"].as_array().unwrap();
    assert_eq!(rows[0]["state"], "delivered");
    assert_eq!(rows[0]["workspace"], "/ws");
    assert_eq!(rows[0]["claimant"], "alba");
    assert_eq!(rows[1]["state"], "ready");
    assert!(rows[1].get("workspace").is_none());
    assert!(rows[1].get("claimant").is_none());
    assert!(rows[1].get("title").is_none());
}

#[test]
fn ops_rows_encode_the_durable_line_fields() {
    let row = OpRow {
        ts: "1700".into(),
        argv: "bl close x".into(),
        cwd: "/p".into(),
        exit: 1,
        stdout: String::new(),
        stderr: "gate".into(),
        origin: Origin::Balls,
    };
    let v = encode(&Reply::Ops(vec![row]));
    let rows = v["rows"].as_array().unwrap();
    assert_eq!(rows[0]["ts"], "1700");
    assert_eq!(rows[0]["exit"], 1);
    assert_eq!(rows[0]["origin"], "balls");
}
