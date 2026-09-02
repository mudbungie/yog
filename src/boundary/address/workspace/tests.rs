//! **The one mapping, written** (REMOTE §8.2): a gesture leaving a client box
//! down a renamed entry's channel must carry the name that workspace answers to
//! on its host, and every other gesture must cross exactly as it was typed.

use super::*;
use crate::boundary::config::ConfigFile;
use crate::boundary::config::Read;
use crate::config_edit::branch::edit::EditOrigin;
use crate::opslog::Origin;
use crate::start::Prepared;

/// The two §9 destinations that name a wall, as an act and as its read
/// (bl-523f) — the family whose address is nested inside `target` rather than
/// at the gesture's top level.
fn config_shapes(named: &str) -> Vec<Gesture> {
    let brazen = ConfigFile::Brazen {
        workspace: named.to_owned(),
    };
    let branch = ConfigFile::Branch {
        workspace: named.to_owned(),
        lineage: "policy".to_owned(),
        origin: EditOrigin::Advance,
        path: "providers.yaml".to_owned(),
    };
    vec![
        Gesture::Act(Action::ApplyConfig {
            file: brazen.clone(),
            text: String::new(),
        }),
        Gesture::Act(Action::ApplyConfig {
            file: branch.clone(),
            text: String::new(),
        }),
        Gesture::Ask(Query::Config(Read::File { file: brazen })),
        Gesture::Ask(Query::Config(Read::File { file: branch })),
    ]
}

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

/// **The §9 family is addressed like everything else** (bl-523f). The wall a
/// config act lands in is the workspace it names, so the table answers it and
/// the §8.2 channel mapping can rewrite it — without this row a config act
/// aimed at a workspace held on another box under a local rename resolved to
/// no entry, fell through to the local engine, and edited the local wall's
/// file. Both halves of the family, because a read of a file and a write of it
/// must cross the same channel.
#[test]
fn a_config_gesture_is_addressed_by_the_wall_its_destination_names() {
    for (gesture, other) in config_shapes("leaf").into_iter().zip(config_shapes("host")) {
        assert_eq!(gesture.workspace().as_deref(), Some("leaf"), "{gesture:?}");
        assert_eq!(other.workspace().as_deref(), Some("host"), "{other:?}");
        assert_ne!(gesture, other, "the wall is the only difference");
    }
}

/// The rewrite half, on the nested field (REMOTE §8.2): a config gesture
/// leaving this box down a renamed entry's channel carries the name that
/// workspace answers to on its host — the same one mapping every other gesture
/// spends, reaching one level down into the destination.
#[test]
fn the_mapping_rewrites_a_config_gestures_nested_wall() {
    let mut act = Action::ApplyConfig {
        file: ConfigFile::Brazen {
            workspace: "leaf".to_owned(),
        },
        text: "x".to_owned(),
    };
    *act.workspace_slot().expect("the family names a wall") = "host".to_owned();
    assert_eq!(
        act,
        Action::ApplyConfig {
            file: ConfigFile::Brazen {
                workspace: "host".to_owned()
            },
            text: "x".to_owned(),
        }
    );
    let mut ask = Query::Config(Read::File {
        file: ConfigFile::Cadence,
    });
    assert!(
        ask.workspace_slot().is_none(),
        "a destination naming no world has nothing to rewrite"
    );
}

/// The three destinations that name no world answer `None` on both halves —
/// litany's two globals and yog's own cadence file are facts about the box, so
/// they cross no entry and route nowhere.
#[test]
fn a_config_destination_naming_no_world_answers_none() {
    for file in [
        ConfigFile::LitanyModels,
        ConfigFile::LitanyWorkflow {
            name: "nightly".to_owned(),
        },
        ConfigFile::Cadence,
    ] {
        let act = Gesture::Act(Action::ApplyConfig {
            file: file.clone(),
            text: String::new(),
        });
        let ask = Gesture::Ask(Query::Config(Read::File { file }));
        assert_eq!(act.workspace(), None, "{act:?}");
        assert_eq!(ask.workspace(), None, "{ask:?}");
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
