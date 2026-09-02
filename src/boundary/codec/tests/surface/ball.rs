//! The `bl` half of the gesture surface, split from its sibling on the two
//! substrates' own line — the same seam `line::tests::parity` cuts, so the two
//! serializations' tables read as one pair rather than as one long list and one
//! short one. It carries the family's five verbs and, beside them, the §11
//! board's **scheduling facts** (bl-dbde): the four a remote seat could not
//! state at all before.

use crate::actions::verbs::Verb as BallVerb;
use crate::actions::verbs::edit::{Create, Field, Update};
use crate::boundary::{Action, Gesture};

/// Every field application, in the order a fold must apply them.
pub(crate) fn every_field() -> Vec<Field> {
    vec![
        Field::Priority(Some(-2)),
        Field::Priority(None),
        Field::Tag {
            tag: "boundary".to_owned(),
            on: true,
        },
        Field::Tag {
            tag: "stale".to_owned(),
            on: false,
        },
        Field::Parent(Some("bl-1a2b".to_owned())),
        Field::Parent(None),
        Field::Needs {
            edge: "bl-9:close".to_owned(),
            on: true,
        },
        Field::Needs {
            edge: "bl-8".to_owned(),
            on: false,
        },
    ]
}

pub(super) fn surface() -> Vec<Gesture> {
    let mut out = vec![
        Gesture::Act(Action::Ball(BallVerb::Close {
            project: "proj".into(),
            id: "bl-1".into(),
            name: "alba".into(),
        })),
        Gesture::Act(Action::Ball(BallVerb::Assign {
            project: "proj".into(),
            id: "bl-1".into(),
            name: "alba".into(),
        })),
        Gesture::Act(Action::Ball(BallVerb::Release {
            project: "proj".into(),
            id: "bl-1".into(),
            name: "alba".into(),
        })),
    ];
    for body in [Some("the body".to_owned()), None] {
        out.push(Gesture::Act(Action::Ball(BallVerb::Create {
            project: "proj".into(),
            name: "alba".into(),
            fields: Create {
                title: "a title".into(),
                body,
                ..Create::default()
            },
        })));
    }
    for fields in [
        Update {
            title: Some("t".into()),
            body: Some(String::new()),
            note: Some("n".into()),
            ..Update::default()
        },
        Update::default(),
    ] {
        out.push(Gesture::Act(Action::Ball(BallVerb::Update {
            project: "proj".into(),
            id: "bl-1".into(),
            name: "alba".into(),
            fields,
        })));
    }
    // The scheduling facts, on both authoring verbs.
    out.push(Gesture::Act(Action::Ball(BallVerb::Create {
        project: "proj".into(),
        name: "alba".into(),
        fields: Create {
            title: "a title".into(),
            body: None,
            fields: every_field(),
        },
    })));
    out.push(Gesture::Act(Action::Ball(BallVerb::Update {
        project: "proj".into(),
        id: "bl-1".into(),
        name: "alba".into(),
        fields: Update {
            fields: every_field(),
            ..Update::default()
        },
    })));
    out
}
