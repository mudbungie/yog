//! Encoding tables for the reply spelling (§8.5): every variant, every
//! optional-field branch, and the [`cleared`] predicate.

mod board;
mod config;
mod inspector;
mod queue;
mod receipts;
mod rows;
mod search;
mod workdiff;

use super::rows::{flight_token, state_token};
use super::*;
use crate::nav::convs::{ConvBall, Flight};
use crate::opslog::Origin;
use crate::projects::join::JoinState;
use std::path::PathBuf;

fn outcome(exit: i32) -> Outcome {
    Outcome {
        exit,
        stdout: "out".into(),
        stderr: "err".into(),
    }
}

#[test]
fn an_outcome_reply_carries_the_captured_run_and_its_verdict() {
    let ok = encode(&Reply::Outcome(outcome(0)));
    assert_eq!(ok["ok"], true);
    assert_eq!(ok["kind"], "outcome");
    assert_eq!(ok["exit"], 0);
    assert_eq!(ok["stdout"], "out");
    assert_eq!(ok["stderr"], "err");
    let failed = encode(&Reply::Outcome(outcome(3)));
    assert_eq!(failed["ok"], false);
}

#[test]
fn the_prepared_reply_is_the_prompt_gestures_own_spelling() {
    let prepared = Prepared {
        name: "alba".into(),
        workspace: PathBuf::from("/ws"),
        binding: Some(PathBuf::from("/target")),
        goal: "g".into(),
        origin: Origin::Balls,
    };
    let v = encode(&Reply::Prepared(prepared.clone()));
    assert_eq!(v["ok"], true);
    assert_eq!(v["kind"], "prepared");
    // The round-trip promise: the reply body re-enters as the next gesture.
    let back = serde_json::json!({ "op": "prompt", "prepared": v["prepared"], "goal": "g2" });
    assert_eq!(
        super::super::codec::decode(&back),
        Ok(super::super::Gesture::Act(super::super::Action::Prompt {
            prepared,
            goal: "g2".into(),
        }))
    );
}

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
    use crate::git_tree::AgentState;
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
        verdict: None,
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
fn the_agent_state_and_flight_tokens_are_total() {
    use crate::git_tree::AgentState;
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

#[test]
fn cleared_is_the_draft_clearing_predicate() {
    assert!(cleared(&Ok(Reply::Outcome(outcome(0)))));
    assert!(!cleared(&Ok(Reply::Outcome(outcome(2)))));
    assert!(cleared(&Ok(Reply::Started {
        conversation: "x".into()
    })));
    assert!(!cleared(&Err("refused".into())));
}

#[test]
fn a_refusal_names_its_reason() {
    let v = refusal("unknown op");
    assert_eq!(v["ok"], false);
    assert_eq!(v["error"], "unknown op");
}

/// A help reply is rows like any other query's: the four facts a page is made
/// of, so a headless reader renders what the window renders.
#[test]
fn a_help_reply_carries_each_page_as_data() {
    let rows = crate::boundary::help::rows(Some("scan"));
    let encoded = encode(&Reply::Help(rows));
    assert_eq!(encoded["ok"], true);
    assert_eq!(encoded["kind"], "help");
    let row = &encoded["rows"][0];
    assert_eq!(row["verb"], "scan");
    assert_eq!(row["usage"], "/scan");
    assert!(row["summary"].as_str().is_some_and(|s| !s.is_empty()));
    assert!(row["detail"].as_str().is_some_and(|s| s.len() > 40));
}
