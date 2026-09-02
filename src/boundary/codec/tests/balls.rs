//! The `bl` family's strict edges (bl-dbde): an empty schedule that is absent
//! on the wire and reads back as empty, and a field row that refuses by name.
//! The family's own round-trip values moved to [`super::surface::ball`] when
//! the conformance corpus took the same list (bl-32cb).

use super::decode;
use crate::actions::verbs::Verb as BallVerb;
use crate::actions::verbs::edit::Update;
use crate::boundary::{Action, Gesture};
use serde_json::json;

/// An empty list is **absent** on the wire, and absence reads back as empty —
/// the same absent-is-a-value rule the optional string fields take.
#[test]
fn an_empty_schedule_is_omitted_and_absence_reads_as_empty() {
    let bare = json!({"op": "update", "project": "p", "id": "bl-1", "name": "alba"});
    assert_eq!(
        decode(&bare),
        Ok(Gesture::Act(Action::Ball(BallVerb::Update {
            project: "p".into(),
            id: "bl-1".into(),
            name: "alba".into(),
            fields: Update::default(),
        }))),
        "a seat that knows nothing of fields still speaks the envelope"
    );
    let encoded = super::super::encode(&Gesture::Act(Action::Ball(BallVerb::Update {
        project: "p".into(),
        id: "bl-1".into(),
        name: "alba".into(),
        fields: Update::default(),
    })));
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
