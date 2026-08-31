//! **The one mapping, written** (REMOTE §8.2): a gesture leaving a client box
//! down a renamed entry's channel must carry the name that workspace answers to
//! on its host, and every other gesture must cross exactly as it was typed.

use super::*;
use crate::opslog::Origin;
use crate::start::Prepared;

/// One gesture of each shape the table answers through: a plain field, a
/// [`Prepared`] two variants share, and the two nested verb families.
fn shapes(named: &str) -> Vec<Gesture> {
    let prepared = Prepared {
        workspace: named.to_owned(),
        binding: None,
        goal: String::new(),
        origin: Origin::Conversation,
        lineage: None,
    };
    vec![
        Gesture::Act(Action::Scan {
            workspace: named.to_owned(),
        }),
        Gesture::Act(Action::Monitor(crate::monitor::Verb::Disarm {
            workspace: named.to_owned(),
        })),
        Gesture::Act(Action::Fleet(crate::fleet::Verb::Disarm {
            workspace: named.to_owned(),
        })),
        Gesture::Act(Action::Prompt {
            prepared,
            goal: String::new(),
            seed: None,
        }),
        Gesture::Ask(Query::Conversations {
            workspace: named.to_owned(),
        }),
    ]
}

/// The read reaches every shape, nested payloads included, through the one
/// table — so no arm can carry a workspace the reader cannot see.
#[test]
fn a_gesture_that_names_a_workspace_answers_with_it() {
    for (gesture, other) in shapes("leaf").into_iter().zip(shapes("host")) {
        assert_eq!(gesture.workspace().as_deref(), Some("leaf"), "{gesture:?}");
        assert_eq!(other.workspace().as_deref(), Some("host"), "{other:?}");
        assert_ne!(gesture, other, "the name is the only difference");
    }
}

/// A gesture naming no workspace answers `None`: the general path with nothing
/// to name, not a case of its own. The two families that name a machine rather
/// than a world are the ones a tool host speaks.
#[test]
fn a_gesture_naming_no_workspace_answers_none() {
    for gesture in [
        Gesture::Act(Action::Ack),
        Gesture::Act(Action::Advertise { tools: Vec::new() }),
        Gesture::Ask(Query::Workspaces),
        Gesture::Ask(Query::Invocations),
    ] {
        assert_eq!(gesture.workspace(), None, "{gesture:?}");
    }
}

/// Reading does not consume: the clone the single table costs is the reader's
/// own, so the gesture it was asked about still names what it named.
#[test]
fn reading_the_name_leaves_the_gesture_holding_it() {
    let gesture = Gesture::Act(Action::Scan {
        workspace: "leaf".to_owned(),
    });
    assert_eq!(gesture.workspace().as_deref(), Some("leaf"));
    assert_eq!(gesture.workspace().as_deref(), Some("leaf"));
}
