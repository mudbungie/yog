//! The `bl` family's parity table (§8.5) — every ball gesture spells as a line
//! and reads back as itself. Its own file on the seam `line/balls.rs` draws:
//! the family whose payload grows every time balls learns a field.

use super::rt;
use crate::actions::verbs::edit;
use crate::boundary::{Action, Gesture};

#[test]
fn every_ball_action_round_trips() {
    let (project, id, name) = ("proj".to_owned(), "bl-1".to_owned(), "alba".to_owned());
    rt(Gesture::Act(Action::Close {
        project: project.clone(),
        id: id.clone(),
        name: name.clone(),
    }));
    rt(Gesture::Act(Action::Assign {
        project: project.clone(),
        id: id.clone(),
        name: name.clone(),
    }));
    rt(Gesture::Act(Action::Release {
        project: project.clone(),
        id: id.clone(),
        name: name.clone(),
    }));
    for body in [None, Some("the body".to_owned())] {
        rt(Gesture::Act(Action::Create {
            project: project.clone(),
            name: name.clone(),
            fields: edit::Create {
                title: "a new ball".to_owned(),
                body,
                ..edit::Create::default()
            },
        }));
    }
    for fields in [
        (Some("t".to_owned()), None, None),
        (None, Some("b".to_owned()), None),
        (None, None, Some("a note".to_owned())),
        (
            Some("t".to_owned()),
            Some("b".to_owned()),
            Some("n".to_owned()),
        ),
    ] {
        rt(Gesture::Act(Action::Update {
            project: project.clone(),
            id: id.clone(),
            name: name.clone(),
            fields: edit::Update {
                title: fields.0,
                body: fields.1,
                note: fields.2,
                ..edit::Update::default()
            },
        }));
    }
}

/// The eight scheduling flags (bl-dbde), each one a line the operator can
/// type, and every one of them read back as the fact it names — the whole list
/// on one gesture, so the **order** the reader preserves is proven too.
pub(super) fn every_field() -> Vec<edit::Field> {
    vec![
        edit::Field::Priority(Some(-2)),
        edit::Field::Priority(None),
        edit::Field::Tag {
            tag: "boundary".to_owned(),
            on: true,
        },
        edit::Field::Tag {
            tag: "stale".to_owned(),
            on: false,
        },
        edit::Field::Parent(Some("bl-1a2b".to_owned())),
        edit::Field::Parent(None),
        edit::Field::Needs {
            edge: "bl-9:close".to_owned(),
            on: true,
        },
        edit::Field::Needs {
            edge: "bl-8".to_owned(),
            on: false,
        },
    ]
}

#[test]
fn every_scheduling_fact_round_trips_on_both_authoring_verbs() {
    rt(Gesture::Act(Action::Create {
        project: "proj".to_owned(),
        name: "alba".to_owned(),
        fields: edit::Create {
            title: "a new ball".to_owned(),
            body: Some("the body".to_owned()),
            fields: every_field(),
        },
    }));
    rt(Gesture::Act(Action::Update {
        project: "proj".to_owned(),
        id: "bl-1".to_owned(),
        name: "alba".to_owned(),
        fields: edit::Update {
            note: Some("a note".to_owned()),
            fields: every_field(),
            ..edit::Update::default()
        },
    }));
}

/// A schedule-only update is a change: the "nothing to change" refusal reads
/// the whole payload, not just its three text fields.
#[test]
fn a_tag_alone_is_enough_to_be_an_update() {
    rt(Gesture::Act(Action::Update {
        project: "proj".to_owned(),
        id: "bl-1".to_owned(),
        name: "alba".to_owned(),
        fields: edit::Update {
            fields: vec![edit::Field::Priority(Some(9))],
            ..edit::Update::default()
        },
    }));
}
