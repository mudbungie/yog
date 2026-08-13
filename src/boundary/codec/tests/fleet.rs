//! The §4.3 armed loop's envelope: total where the line is terse.
//!
//! A line takes the project and the workspace from the seat it was typed at;
//! the envelope has no seat, so it names both — and the cap, which is the one
//! thing no seat could supply and yog must never guess.

use super::*;
use crate::fleet::Verb as Fleet;

#[test]
fn arming_and_disbanding_round_trip() {
    rt(Gesture::Act(Action::Fleet(Fleet::Arm {
        workspace: p("/ws"),
        project: p("/proj"),
        cap: 4,
    })));
    rt(Gesture::Act(Action::Fleet(Fleet::Disarm {
        workspace: p("/ws"),
    })));
}

/// Every field an arm needs is required, and a cap that is not a number is a
/// refusal naming it — arming on a guessed cap would spend the operator's money
/// on yog's opinion.
#[test]
fn an_arm_missing_a_field_or_carrying_a_bad_cap_refuses() {
    for envelope in [
        serde_json::json!({ "op": "fleet", "project": "/proj", "cap": 1 }),
        serde_json::json!({ "op": "fleet", "workspace": "/ws", "cap": 1 }),
        serde_json::json!({ "op": "fleet", "workspace": "/ws", "project": "/proj" }),
        serde_json::json!({ "op": "fleet", "workspace": "/ws", "project": "/proj", "cap": "lots" }),
    ] {
        assert!(decode(&envelope).is_err(), "{envelope}");
    }
    // Disbanding still needs the workspace it disbands.
    assert!(decode(&serde_json::json!({ "op": "disband" })).is_err());
}
