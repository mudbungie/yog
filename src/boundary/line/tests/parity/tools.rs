//! REMOTE §5's pair on the line (bl-4e08): a presentation whose whole tail is a
//! JSON document, and the roster that reads it back.

use super::rt;
use crate::boundary::{Action, Gesture, Query};

/// One advertised tool, with a schema deep enough that a spelling which rebuilt
/// it rather than carrying it verbatim would show.
fn advertised() -> crate::registry::tools::Tool {
    crate::registry::tools::Tool {
        name: "Bash".to_owned(),
        description: "run a command".to_owned(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {"command": {"type": "string", "minLength": 1}},
            "required": ["command"],
        }),
    }
}

/// The set is the whole tail, so the round trip is what says a **document**
/// survives a line — and the empty set has to be typable, or a tool host could
/// never withdraw one.
#[test]
fn the_presentation_round_trips_with_its_schema_and_when_it_is_empty() {
    for tools in [Vec::new(), vec![advertised()]] {
        rt(Gesture::Act(Action::Advertise { tools }));
    }
}

/// The routing leg's four (bl-024b). Both acts end in a document taken
/// verbatim, so the round trip is what says a JSON payload survives a line;
/// both reads state only what no seat can supply, which for one of them is
/// nothing at all.
#[test]
fn the_routing_legs_gestures_round_trip() {
    rt(Gesture::Act(Action::Route(
        crate::registry::mailbox::Verb::Invoke(crate::registry::mailbox::Call {
            client: "laptop".to_owned(),
            tool: "Bash".to_owned(),
            input: serde_json::json!({"command": "ls -l", "timeout": 30}),
        }),
    )));
    rt(Gesture::Act(Action::Route(
        crate::registry::mailbox::Verb::Complete(crate::registry::mailbox::Completion {
            invocation: "inv-1".to_owned(),
            capture: crate::registry::mailbox::Capture {
                stdout: "hello\n".to_owned(),
                stderr: "warned\n".to_owned(),
                exit_code: 3,
            },
        }),
    )));
    rt(Gesture::Ask(Query::Invocations));
    rt(Gesture::Ask(Query::Capture {
        invocation: "inv-1".to_owned(),
    }));
}

/// The roster spells as the verb alone: its workspace is the seat's, exactly as
/// `/providers`' is.
#[test]
fn the_roster_spells_as_the_verb_alone() {
    rt(Gesture::Ask(Query::Clients {
        workspace: "ws".to_owned(),
    }));
}
