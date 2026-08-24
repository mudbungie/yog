//! Round-trip tables for the gesture codec (§8.5 deliverable 3): **every**
//! variant re-enters as itself. The other half of that deliverable — every
//! malformed envelope refusing with a reason, never a guessed default — is
//! [`refusals`], split out at §12's cap.

use super::*;
use crate::actions::verbs::edit;
use crate::monitor::Verb;
use std::path::PathBuf;

mod balls;
mod control;
mod fan;
mod fleet;
mod fork;
mod query;
mod refusals;
mod retarget;
/// The §8.1 start family's own enum tables.
mod start;

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
