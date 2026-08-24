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

/// The rewrite reaches every shape, nested payloads included — and the read
/// answers through the very table the write borrows, so the two cannot disagree
/// about which arms have a workspace at all.
#[test]
fn a_gesture_that_names_a_workspace_carries_the_new_name_after_the_rewrite() {
    for (gesture, already) in shapes("leaf").into_iter().zip(shapes("host")) {
        assert_eq!(gesture.workspace().as_deref(), Some("leaf"), "{gesture:?}");
        let crossing = gesture.with_workspace("host");
        assert_eq!(
            crossing.workspace().as_deref(),
            Some("host"),
            "{crossing:?}"
        );
        assert_eq!(crossing, already, "only the name changed");
    }
}

/// A gesture naming no workspace comes back untouched: the general path with
/// nothing to rewrite, not a case of its own. The two families that name a
/// machine rather than a world are the ones a tool host speaks, which is why
/// that mode resolves channels instead of names.
#[test]
fn a_gesture_naming_no_workspace_is_unchanged() {
    for gesture in [
        Gesture::Act(Action::Ack),
        Gesture::Act(Action::Advertise { tools: Vec::new() }),
        Gesture::Ask(Query::Workspaces),
        Gesture::Ask(Query::Invocations),
    ] {
        assert_eq!(gesture.workspace(), None, "{gesture:?}");
        assert_eq!(gesture.clone().with_workspace("host"), gesture);
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
