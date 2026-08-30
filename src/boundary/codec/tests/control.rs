//! The §8.6 capability family's envelope: strict about the one field it has.
//!
//! The `tool_use` id is deliberately absent in both directions — it is derived
//! from the hold mark at fire time, so a headless caller cannot quote a stale
//! one, and the answer always lands on what is parked now.

use super::*;
use crate::control::judge::Ruling;

/// The family's values: every verdict, and the floor in both directions.
pub(super) fn surface() -> Vec<Gesture> {
    let mut out = Vec::new();
    for ruling in [Ruling::Pass, Ruling::Hold, Ruling::Refuse] {
        out.push(Gesture::Act(Action::AnswerHold {
            workspace: "ws".into(),
            agent: "c-1".into(),
            ruling,
        }));
    }
    for raised in [true, false] {
        out.push(Gesture::Act(Action::Floor {
            workspace: "ws".into(),
            agent: "c-1".into(),
            raised,
        }));
    }
    out
}

#[test]
fn the_held_id_is_never_on_the_wire() {
    for ruling in [Ruling::Pass, Ruling::Hold, Ruling::Refuse] {
        let envelope = encode(&Gesture::Act(Action::AnswerHold {
            workspace: "ws".into(),
            agent: "c-1".into(),
            ruling,
        }))
        .to_string();
        assert!(!envelope.contains("tool_use"), "{envelope}");
    }
}

/// A verdict nobody could act on is refused at the edge, naming what was said
/// and what is allowed — never defaulted, because a default verdict would be
/// yog deciding what the operator meant.
#[test]
fn an_unknown_verdict_is_refused() {
    let refused = decode(&serde_json::json!({
        "op": "answer", "workspace": "/ws", "agent": "c-1", "verdict": "maybe"
    }));
    assert_eq!(
        refused,
        Err("answer: unknown verdict \"maybe\"; say pass, hold or refuse".to_owned())
    );
    // And the two address fields are required like every other envelope's.
    assert!(decode(&serde_json::json!({ "op": "answer", "agent": "c-1" })).is_err());
    assert!(decode(&serde_json::json!({ "op": "answer", "workspace": "/ws" })).is_err());
}

/// The §4.9 fifth rung: two ops for one variant, because raising and lowering
/// are two instructions — and the direction is the op, never a field a caller
/// could omit into the wrong one.
#[test]
fn the_floor_spells_its_direction_as_the_op() {
    for raised in [true, false] {
        let gesture = Gesture::Act(Action::Floor {
            workspace: "ws".into(),
            agent: "c-1".into(),
            raised,
        });
        let envelope = encode(&gesture).to_string();
        let op = if raised { "revoke" } else { "restore" };
        assert!(envelope.contains(op), "{envelope}");
        assert!(!envelope.contains("raised"), "{envelope}");
    }
    // Both address fields are required, like every other envelope's.
    assert!(decode(&serde_json::json!({ "op": "revoke", "agent": "c-1" })).is_err());
    assert!(decode(&serde_json::json!({ "op": "restore", "workspace": "/ws" })).is_err());
}
