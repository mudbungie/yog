//! Round-trip tables for the gesture codec (§8.5 deliverable 3): **every**
//! variant re-enters as itself. The other half of that deliverable — every
//! malformed envelope refusing with a reason, never a guessed default — is
//! [`refusals`], split out at §12's cap.

use super::*;
use crate::actions::verbs::edit;
use crate::monitor::Verb;
use crate::opslog::Origin;
use crate::projects::join::JoinState;
use crate::start::{BallSpec, Payload, Prepared};
use std::path::PathBuf;

mod balls;
mod control;
mod fan;
mod fleet;
mod fork;
mod query;
mod refusals;
mod retarget;

pub(super) fn rt(gesture: Gesture) {
    let encoded = encode(&gesture);
    assert_eq!(decode(&encoded), Ok(gesture.clone()), "via {encoded}");
}

pub(super) fn p(s: &str) -> PathBuf {
    PathBuf::from(s)
}

/// The workspace-and-conversation half: the §8.2 lernie family, the §3.6
/// deletes, the §4.9 monitor, the §4.11 answer and the trail's own verbs.
#[test]
fn every_action_variant_round_trips() {
    rt(Gesture::Act(Action::Message {
        workspace: "ws".into(),
        agent: "c-1".into(),
        content: "hi there".into(),
    }));
    rt(Gesture::Act(Action::Interrupt {
        workspace: "ws".into(),
        agent: "c-1".into(),
        content: "no, this".into(),
    }));
    rt(Gesture::Act(Action::Stop {
        workspace: "ws".into(),
        agent: "c-1".into(),
        children: true,
    }));
    rt(Gesture::Act(Action::Scan {
        workspace: "ws".into(),
    }));
    rt(Gesture::Act(Action::Nudge {
        workspace: "ws".into(),
        agent: "c-1".into(),
    }));
    rt(Gesture::Act(Action::DeleteWorkspace {
        workspace: "ws".into(),
        typed: "alba".into(),
    }));
    rt(Gesture::Act(Action::DeleteAgent {
        workspace: "ws".into(),
        agent: "c-1".into(),
        typed: "the goal name".into(),
    }));
    rt(Gesture::Act(Action::Monitor(Verb::Arm {
        workspace: "ws".into(),
        model: "claude-haiku-4-5".into(),
    })));
    rt(Gesture::Act(Action::Monitor(Verb::Disarm {
        workspace: "ws".into(),
    })));
    rt(Gesture::Act(Action::Monitor(Verb::Flag {
        workspace: "ws".into(),
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
            workspace: "ws".into(),
            agent: "c-1".into(),
            ruling,
        }));
    }
    rt(Gesture::Act(Action::Ack));
    rt(Gesture::Act(Action::MarkSeen {
        workspace: "ws".into(),
        agent: "c-1".into(),
    }));
    rt(Gesture::Act(Action::ClearTrail));
    // REMOTE §5's presentation (bl-4e08). The empty set is a set — a host that
    // stops offering everything says so with this gesture, not by silence —
    // and the schema must survive the trip verbatim.
    for tools in [Vec::new(), vec![advertised()]] {
        rt(Gesture::Act(Action::Advertise { tools }));
    }
    // The routing leg's two acts (bl-024b): the model's arguments and the
    // program's own output, each carried verbatim.
    rt(Gesture::Act(Action::Route(
        crate::registry::mailbox::Verb::Invoke(crate::registry::mailbox::Call {
            client: "laptop".into(),
            tool: "Bash".into(),
            input: serde_json::json!({"command": "ls -l", "timeout": 30}),
        }),
    )));
    rt(Gesture::Act(Action::Route(
        crate::registry::mailbox::Verb::Complete(crate::registry::mailbox::Completion {
            invocation: "inv-1".into(),
            capture: crate::registry::mailbox::Capture {
                stdout: "hello\n".into(),
                stderr: "warned\n".into(),
                exit_code: 3,
            },
        }),
    )));
}

/// One advertised tool with a schema deep enough that a codec which rebuilt it
/// rather than carrying it would show.
fn advertised() -> crate::registry::tools::Tool {
    crate::registry::tools::Tool {
        name: "Bash".into(),
        description: "run a command".into(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {"command": {"type": "string", "minLength": 1}},
            "required": ["command"],
        }),
    }
}

/// The `bl` half, split from its sibling on the two substrates' own line — the
/// same seam `line::tests::parity` cuts, so the two serializations' tables read
/// as one pair rather than as one long list and one short one.
#[test]
fn every_ball_action_variant_round_trips() {
    rt(Gesture::Act(Action::Close {
        project: "proj".into(),
        id: "bl-1".into(),
        name: "alba".into(),
    }));
    rt(Gesture::Act(Action::Assign {
        project: "proj".into(),
        id: "bl-1".into(),
        name: "alba".into(),
    }));
    rt(Gesture::Act(Action::Release {
        project: "proj".into(),
        id: "bl-1".into(),
        name: "alba".into(),
    }));
    for body in [Some("the body".to_owned()), None] {
        rt(Gesture::Act(Action::Create {
            project: "proj".into(),
            name: "alba".into(),
            fields: edit::Create {
                title: "a title".into(),
                body,
                ..edit::Create::default()
            },
        }));
    }
    for fields in [
        edit::Update {
            title: Some("t".into()),
            body: Some(String::new()),
            note: Some("n".into()),
            ..edit::Update::default()
        },
        edit::Update::default(),
    ] {
        rt(Gesture::Act(Action::Update {
            project: "proj".into(),
            id: "bl-1".into(),
            name: "alba".into(),
            fields,
        }));
    }
}

#[test]
fn every_payload_rung_round_trips_inside_prepare() {
    for payload in [
        Payload::Bare,
        Payload::Path { dir: p("/work") },
        Payload::Ball {
            project: "proj".into(),
            ball: BallSpec::New {
                title: "t".into(),
                body: "b".into(),
            },
        },
    ] {
        rt(Gesture::Act(Action::Prepare {
            workspace: "ws".into(),
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
            workspace: "ws".into(),
            payload: Payload::Ball {
                project: "proj".into(),
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

/// Every origin **and both states of the §3.3 typed binding** (bl-6654): a
/// bound rung and the bare rung's `None` are two values of one field, so both
/// have to survive the wire — `null` decoding as "bind nothing" is the whole
/// reason the bare rung can be deposited back as the gesture it was.
#[test]
fn every_origin_round_trips_inside_a_prompt() {
    for origin in [Origin::Balls, Origin::Conversation, Origin::World] {
        for binding in [None, Some(p("/target"))] {
            // Both halves of the §3.3 seed (bl-1747): a seat that predicted a
            // name carries it, and one that predicted none carries `None` —
            // absent and present are different facts, so both cross.
            for seed in [None, Some(0xc0df)] {
                rt(Gesture::Act(Action::Prompt {
                    prepared: Prepared {
                        workspace: "ws".into(),
                        binding: binding.clone(),
                        goal: "the goal".into(),
                        origin,
                    },
                    goal: "edited goal".into(),
                    seed,
                }));
            }
        }
    }
}

#[test]
fn a_stop_without_the_children_field_defaults_to_false() {
    let decoded = decode(&serde_json::json!({
        "op": "stop", "workspace": "ws", "agent": "c-1"
    }));
    assert_eq!(
        decoded,
        Ok(Gesture::Act(Action::Stop {
            workspace: "ws".into(),
            agent: "c-1".into(),
            children: false,
        }))
    );
}

/// **The start family's envelope builder answers `null` to anything else**
/// (bl-1747). Its one caller is the action table's `Prepare | Prompt` arm, so
/// nothing else can reach it in production — but a fallback nobody can reach is
/// still one somebody could widen that arm onto, and `null` is the honest
/// answer: an envelope with no `op`, which decode refuses by name. Pinned here
/// rather than assumed, so the pair and its fallback are read together.
#[test]
fn the_start_envelope_answers_null_to_anything_but_its_own_two() {
    let value = encode_start(&Action::Ack);
    assert!(value.is_null(), "not a start gesture, so not an envelope");
    assert!(decode(&value).is_err(), "and decode refuses it");
}
