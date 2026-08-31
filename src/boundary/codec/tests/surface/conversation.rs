//! The workspace-and-conversation half of the gesture surface: the §8.2 litany
//! family, the §3.6 deletes, the §4.9 monitor, the §4.11 answer and the trail's
//! own verbs — the list the round trip and the conformance corpus both walk.

use crate::boundary::{Action, Gesture};
use crate::monitor::Verb;

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
        subject_cwd: false,
    }
}

pub(super) fn surface() -> Vec<Gesture> {
    let mut out = vec![
        Gesture::Act(Action::Message {
            workspace: "ws".into(),
            agent: "c-1".into(),
            content: "hi there".into(),
        }),
        Gesture::Act(Action::Interrupt {
            workspace: "ws".into(),
            agent: "c-1".into(),
            content: "no, this".into(),
        }),
        Gesture::Act(Action::Stop {
            workspace: "ws".into(),
            agent: "c-1".into(),
            children: true,
        }),
        Gesture::Act(Action::Scan {
            workspace: "ws".into(),
        }),
        Gesture::Act(Action::Nudge {
            workspace: "ws".into(),
            agent: "c-1".into(),
        }),
        Gesture::Act(Action::DeleteWorkspace {
            workspace: "ws".into(),
            typed: "alba".into(),
        }),
        Gesture::Act(Action::DeleteAgent {
            workspace: "ws".into(),
            agent: "c-1".into(),
            typed: "the goal name".into(),
        }),
        Gesture::Act(Action::Monitor(Verb::Arm {
            workspace: "ws".into(),
            model: "claude-haiku-4-5".into(),
        })),
        Gesture::Act(Action::Monitor(Verb::Disarm {
            workspace: "ws".into(),
        })),
        Gesture::Act(Action::Monitor(Verb::Flag {
            workspace: "ws".into(),
            agent: "c-1".into(),
            reason: "it is rewriting an unrelated crate".into(),
        })),
    ];
    // The §8.6 capability answer, one envelope per verdict — the vocabulary is
    // the control's own, so all three spell and read back.
    for ruling in [
        crate::control::judge::Ruling::Pass,
        crate::control::judge::Ruling::Hold,
        crate::control::judge::Ruling::Refuse,
    ] {
        out.push(Gesture::Act(Action::AnswerHold {
            workspace: "ws".into(),
            agent: "c-1".into(),
            ruling,
        }));
    }
    out.push(Gesture::Act(Action::Ack));
    out.push(Gesture::Act(Action::MarkSeen {
        workspace: "ws".into(),
        agent: "c-1".into(),
    }));
    out.push(Gesture::Act(Action::ClearTrail));
    // REMOTE §5's presentation (bl-4e08). The empty set is a set — a host that
    // stops offering everything says so with this gesture, not by silence —
    // and the schema must survive the trip verbatim. The consenting element
    // (bl-77be) rides beside the plain one, because a table that only ever
    // spells the easy case proves only that the easy case crosses.
    let consenting = crate::registry::tools::Tool {
        subject_cwd: true,
        ..advertised()
    };
    for tools in [Vec::new(), vec![advertised()], vec![consenting]] {
        out.push(Gesture::Act(Action::Advertise { tools }));
    }
    // The routing leg's two acts (bl-024b): the model's arguments and the
    // program's own output, each carried verbatim. The worktree lane's call
    // (bl-77be) rides beside the bare one: the subject's location is an
    // optional field, and both arms must cross.
    for cwd in [None, Some("/w/home/agents/c-1".to_owned())] {
        out.push(Gesture::Act(Action::Route(
            crate::registry::mailbox::Verb::Invoke(crate::registry::mailbox::Call {
                client: "laptop".into(),
                tool: "Bash".into(),
                input: serde_json::json!({"command": "ls -l", "timeout": 30}),
                cwd,
            }),
        )));
    }
    out.push(Gesture::Act(Action::Route(
        crate::registry::mailbox::Verb::Complete(crate::registry::mailbox::Completion {
            invocation: "inv-1".into(),
            capture: crate::registry::mailbox::Capture {
                stdout: "hello\n".into(),
                stderr: "warned\n".into(),
                exit_code: 3,
            },
        }),
    )));
    out
}
