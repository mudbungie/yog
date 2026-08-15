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

/// The roster spells as the verb alone: its workspace is the seat's, exactly as
/// `/providers`' is.
#[test]
fn the_roster_spells_as_the_verb_alone() {
    rt(Gesture::Ask(Query::Clients {
        workspace: "ws".to_owned(),
    }));
}
