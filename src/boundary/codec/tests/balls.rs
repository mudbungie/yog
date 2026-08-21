//! The `bl` family's **scheduling facts** on the wire (bl-dbde): the four the
//! §11 board orders on and the §4.3 fleet selects by, which a remote seat could
//! not state at all before. Its own file beside the family tables above, on the
//! seam `codec/balls.rs` is already cut on.

use super::{decode, rt};
use crate::actions::verbs::edit::{Create, Field, Update};
use crate::boundary::{Action, Gesture};
use serde_json::json;

/// Every field application, in the order a fold must apply them.
fn every_field() -> Vec<Field> {
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

#[test]
fn every_scheduling_fact_round_trips_on_both_authoring_verbs() {
    rt(Gesture::Act(Action::Create {
        project: "proj".into(),
        name: "alba".into(),
        fields: Create {
            title: "a title".into(),
            body: None,
            fields: every_field(),
        },
    }));
    rt(Gesture::Act(Action::Update {
        project: "proj".into(),
        id: "bl-1".into(),
        name: "alba".into(),
        fields: Update {
            fields: every_field(),
            ..Update::default()
        },
    }));
}

/// An empty list is **absent** on the wire, and absence reads back as empty —
/// the same absent-is-a-value rule the optional string fields take.
#[test]
fn an_empty_schedule_is_omitted_and_absence_reads_as_empty() {
    let bare = json!({"op": "update", "project": "p", "id": "bl-1", "name": "alba"});
    assert_eq!(
        decode(&bare),
        Ok(Gesture::Act(Action::Update {
            project: "p".into(),
            id: "bl-1".into(),
            name: "alba".into(),
            fields: Update::default(),
        })),
        "a seat that knows nothing of fields still speaks the envelope"
    );
    let encoded = super::super::encode(&Gesture::Act(Action::Update {
        project: "p".into(),
        id: "bl-1".into(),
        name: "alba".into(),
        fields: Update::default(),
    }));
    assert_eq!(encoded, bare, "and nothing empty is written back out");
}

/// The strict edge: a field row that is not an object, names no field, names an
/// unknown one, or omits the direction the two add-or-drop facts need.
#[test]
fn a_malformed_field_row_refuses_naming_its_offence() {
    let cases: Vec<(serde_json::Value, &str)> = vec![
        (json!("boundary"), "a ball field is an object"),
        (json!({}), "field \"field\""),
        (json!({"field": "epic"}), "unknown ball field \"epic\""),
        (json!({"field": "tag", "value": "t"}), "field \"on\""),
        (json!({"field": "needs", "value": "bl-1"}), "field \"on\""),
        (
            json!({"field": "priority", "value": "high"}),
            "field \"value\"",
        ),
        (json!({"field": "parent", "value": 7}), "field \"value\""),
    ];
    for (row, needle) in cases {
        let envelope = json!({"op": "update", "project": "p", "id": "bl-1",
                              "name": "alba", "fields": [row]});
        let err = decode(&envelope).expect_err(&envelope.to_string());
        assert!(err.contains(needle), "{envelope} -> {err:?}");
    }
    let not_a_list = json!({"op": "create", "project": "p", "title": "t",
                            "name": "alba", "fields": "priority"});
    let err = decode(&not_a_list).expect_err("a schedule is a list");
    assert!(err.contains("non-array field \"fields\""), "{err:?}");
}
