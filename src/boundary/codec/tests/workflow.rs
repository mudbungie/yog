//! The §9.4 workflow mark's envelopes (bl-b680): both directions on the wire,
//! and the strictness the set's lineage is read under.

use crate::boundary::codec::{decode, encode};
use crate::boundary::{Action, Gesture};
use serde_json::json;

/// One entry per direction, the pin's precedent: a fixture that only ever
/// spelled the set would leave the clear unproven on the wire.
pub(super) fn surface() -> Vec<Gesture> {
    vec![gesture(Some("strict")), gesture(None)]
}

fn gesture(config: Option<&str>) -> Gesture {
    Gesture::Act(Action::Workflow {
        workspace: "ws".into(),
        agent: "c-1".into(),
        config: config.map(str::to_owned),
    })
}

#[test]
fn the_set_names_its_lineage_and_the_clear_is_its_own_op() {
    assert_eq!(
        encode(&gesture(Some("strict"))),
        json!({ "op": "workflow", "workspace": "ws", "agent": "c-1", "config": "strict" })
    );
    assert_eq!(
        encode(&gesture(None)),
        json!({ "op": "clear-workflow", "workspace": "ws", "agent": "c-1" })
    );
}

/// A set with no lineage is refused by name rather than read as a clear: the
/// direction is the op token, never the presence of a field.
#[test]
fn a_set_without_a_lineage_refuses_and_a_clear_ignores_one() {
    let err = decode(&json!({ "op": "workflow", "workspace": "ws", "agent": "c-1" }))
        .expect_err("the lineage is required");
    assert!(err.contains("config"), "{err}");
    assert_eq!(
        decode(
            &json!({ "op": "clear-workflow", "workspace": "ws", "agent": "c-1",
                        "config": "strict" })
        ),
        Ok(gesture(None))
    );
}
