//! Round-trip tables for the gesture codec (§8.5 deliverable 3): **every**
//! variant re-enters as itself, and every malformed envelope refuses with a
//! reason — never a guessed default.

use super::*;
use crate::monitor::Verb;
use crate::opslog::Origin;
use crate::projects::join::JoinState;
use crate::start::{BallSpec, Payload, Prepared};
use std::path::PathBuf;

mod control;
mod fleet;
mod fork;
mod query;

pub(super) fn rt(gesture: Gesture) {
    let encoded = encode(&gesture);
    assert_eq!(decode(&encoded), Ok(gesture.clone()), "via {encoded}");
}

pub(super) fn p(s: &str) -> PathBuf {
    PathBuf::from(s)
}

#[test]
fn every_action_variant_round_trips() {
    rt(Gesture::Act(Action::Message {
        workspace: p("/ws"),
        agent: "c-1".into(),
        content: "hi there".into(),
    }));
    rt(Gesture::Act(Action::Stop {
        workspace: p("/ws"),
        agent: "c-1".into(),
        children: true,
    }));
    rt(Gesture::Act(Action::Scan {
        workspace: p("/ws"),
    }));
    rt(Gesture::Act(Action::Close {
        project: p("/proj"),
        id: "bl-1".into(),
        name: "alba".into(),
    }));
    rt(Gesture::Act(Action::Assign {
        project: p("/proj"),
        id: "bl-1".into(),
        name: "alba".into(),
    }));
    rt(Gesture::Act(Action::Release {
        project: p("/proj"),
        id: "bl-1".into(),
        name: "alba".into(),
    }));
    rt(Gesture::Act(Action::Move {
        project: p("/proj"),
        id: "bl-1".into(),
        from: "alba".into(),
        to: "koi".into(),
    }));
    rt(Gesture::Act(Action::Create {
        project: p("/proj"),
        title: "a title".into(),
        name: "alba".into(),
        body: Some("the body".into()),
    }));
    rt(Gesture::Act(Action::Create {
        project: p("/proj"),
        title: "a title".into(),
        name: "alba".into(),
        body: None,
    }));
    rt(Gesture::Act(Action::Update {
        project: p("/proj"),
        id: "bl-1".into(),
        name: "alba".into(),
        title: Some("t".into()),
        body: Some(String::new()),
        note: Some("n".into()),
    }));
    rt(Gesture::Act(Action::Update {
        project: p("/proj"),
        id: "bl-1".into(),
        name: "alba".into(),
        title: None,
        body: None,
        note: None,
    }));
    rt(Gesture::Act(Action::DeleteWorkspace {
        workspace: p("/ws"),
        typed: "alba".into(),
    }));
    rt(Gesture::Act(Action::DeleteAgent {
        workspace: p("/ws"),
        agent: "c-1".into(),
        typed: "the goal name".into(),
    }));
    rt(Gesture::Act(Action::Monitor(Verb::Arm {
        workspace: p("/ws"),
        model: "claude-haiku-4-5".into(),
    })));
    rt(Gesture::Act(Action::Monitor(Verb::Disarm {
        workspace: p("/ws"),
    })));
    rt(Gesture::Act(Action::Monitor(Verb::Flag {
        workspace: p("/ws"),
        agent: "c-1".into(),
        reason: "it is rewriting an unrelated crate".into(),
    })));
    // The §8.6 capability answer, one envelope per verdict — the vocabulary is
    // the control's own, so all three spell and read back.
    for ruling in [
        crate::control::judge::Ruling::Pass,
        crate::control::judge::Ruling::Hold,
        crate::control::judge::Ruling::Refuse,
    ] {
        rt(Gesture::Act(Action::AnswerHold {
            workspace: p("/ws"),
            agent: "c-1".into(),
            ruling,
        }));
    }
    rt(Gesture::Act(Action::Ack));
    rt(Gesture::Act(Action::MarkSeen {
        workspace: p("/ws"),
        agent: "c-1".into(),
    }));
    rt(Gesture::Act(Action::ClearTrail));
}

#[test]
fn every_payload_rung_round_trips_inside_prepare() {
    for payload in [
        Payload::Bare,
        Payload::Path { dir: p("/work") },
        Payload::Ball {
            project: p("/proj"),
            ball: BallSpec::New {
                title: "t".into(),
                body: "b".into(),
            },
        },
    ] {
        rt(Gesture::Act(Action::Prepare {
            workspace: p("/ws"),
            payload,
        }));
    }
}

#[test]
fn every_join_state_round_trips_inside_an_existing_ball() {
    for join in [
        JoinState::ReadyStartable,
        JoinState::Blocked,
        JoinState::Bound,
        JoinState::ClaimedElsewhere,
        JoinState::Delivered,
        JoinState::UnassignedWorkspace,
        JoinState::OrphanedProject,
    ] {
        rt(Gesture::Act(Action::Prepare {
            workspace: p("/ws"),
            payload: Payload::Ball {
                project: p("/proj"),
                ball: BallSpec::Existing {
                    id: "bl-9".into(),
                    title: "t".into(),
                    body: "b".into(),
                    join,
                },
            },
        }));
    }
}

#[test]
fn every_origin_round_trips_inside_a_prompt() {
    for origin in [Origin::Balls, Origin::Conversation, Origin::World] {
        rt(Gesture::Act(Action::Prompt {
            prepared: Prepared {
                name: "alba".into(),
                workspace: p("/ws"),
                cwd: p("/cwd"),
                goal: "the goal".into(),
                origin,
            },
            goal: "edited goal".into(),
        }));
    }
}

#[test]
fn a_stop_without_the_children_field_defaults_to_false() {
    let decoded = decode(&serde_json::json!({
        "op": "stop", "workspace": "/ws", "agent": "c-1"
    }));
    assert_eq!(
        decoded,
        Ok(Gesture::Act(Action::Stop {
            workspace: p("/ws"),
            agent: "c-1".into(),
            children: false,
        }))
    );
}

/// Every refusal names its offence — the depositor's only diagnostic.
#[test]
fn malformed_envelopes_refuse_with_a_reason() {
    use serde_json::json;
    let cases: Vec<(serde_json::Value, &str)> = vec![
        (json!("not an object"), "not a JSON object"),
        (json!({}), "missing or non-string field \"op\""),
        (json!({"op": "warp"}), "unknown op \"warp\""),
        (
            json!({"op": "message", "workspace": "/ws"}),
            "field \"agent\"",
        ),
        (
            json!({"op": "message", "workspace": 7, "agent": "a", "content": "c"}),
            "field \"workspace\"",
        ),
        (json!({"op": "ops"}), "non-integer field \"max\""),
        (json!({"op": "seen", "workspace": "/ws"}), "field \"agent\""),
        (
            json!({"op": "ops", "max": "many"}),
            "non-integer field \"max\"",
        ),
        (
            json!({"op": "prepare", "workspace": "/ws"}),
            "missing payload",
        ),
        (
            json!({"op": "prepare", "workspace": "/ws", "payload": 3}),
            "payload: not an object",
        ),
        (
            json!({"op": "prepare", "workspace": "/ws", "payload": {"rung": "warp"}}),
            "unknown rung",
        ),
        (
            json!({"op": "prepare", "workspace": "/ws",
                   "payload": {"rung": "ball", "project": "/p"}}),
            "missing ball",
        ),
        (
            json!({"op": "prepare", "workspace": "/ws",
                   "payload": {"rung": "ball", "project": "/p", "ball": 4}}),
            "ball: not an object",
        ),
        (
            json!({"op": "prepare", "workspace": "/ws",
                   "payload": {"rung": "ball", "project": "/p",
                               "ball": {"id": 9, "title": "t", "body": "b"}}}),
            "id not a string",
        ),
        (
            json!({"op": "prepare", "workspace": "/ws",
                   "payload": {"rung": "ball", "project": "/p",
                               "ball": {"id": "x", "title": "t", "body": "b"}}}),
            "field \"join\"",
        ),
        (
            json!({"op": "prepare", "workspace": "/ws",
                   "payload": {"rung": "ball", "project": "/p",
                               "ball": {"id": "x", "title": "t", "body": "b", "join": "warp"}}}),
            "unknown join state",
        ),
        (json!({"op": "prompt", "goal": "g"}), "missing prepared"),
        (
            json!({"op": "prompt", "prepared": [], "goal": "g"}),
            "prepared: not an object",
        ),
        (
            json!({"op": "prompt", "goal": "g",
                   "prepared": {"name": "n", "workspace": "/w", "cwd": "/c",
                                "goal": "g", "origin": "warp"}}),
            "unknown origin",
        ),
        (
            json!({"op": "update", "project": "/p", "id": "x", "name": "n", "title": 5}),
            "title: not a string",
        ),
    ];
    for (envelope, needle) in cases {
        let err = decode(&envelope).expect_err(&envelope.to_string());
        assert!(err.contains(needle), "{envelope} -> {err:?}");
    }
}
