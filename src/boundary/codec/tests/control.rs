//! The §8.6 capability family's envelope: strict about both fields it has.
//!
//! The `tool_use` id is deliberately absent in both directions — it is derived
//! from the hold mark at fire time, so a headless caller cannot quote a stale
//! one, and the answer always lands on what is parked now.

use super::*;
use crate::control::judge::{Answer, Ruling, Scope};

/// The family's values: every verdict at every scope, and the floor in both
/// directions.
pub(super) fn surface() -> Vec<Gesture> {
    let mut out = Vec::new();
    for answer in answers() {
        out.push(Gesture::Act(Action::AnswerHold {
            workspace: "ws".into(),
            agent: "c-1".into(),
            answer,
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

/// Every answer the vocabulary spells: three verdicts over three scopes.
pub(super) fn answers() -> Vec<Answer> {
    let mut out = Vec::new();
    for ruling in [Ruling::Pass, Ruling::Hold, Ruling::Refuse] {
        for scope in [Scope::Call, Scope::Conversation, Scope::Workspace] {
            out.push(Answer { ruling, scope });
        }
    }
    out
}

#[test]
fn the_held_id_is_never_on_the_wire() {
    for answer in answers() {
        let envelope = encode(&Gesture::Act(Action::AnswerHold {
            workspace: "ws".into(),
            agent: "c-1".into(),
            answer,
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

/// And so is a scope (bl-94a5), for the harder version of the same reason: a
/// verdict read out of a typo settles one call, a *scope* read out of one
/// settles every call of a class. Absent refuses too — the field is required
/// rather than defaulted, so the two ends can never disagree about how wide an
/// instruction was.
#[test]
fn an_unknown_or_missing_scope_is_refused() {
    let said = |scope: serde_json::Value| {
        decode(&serde_json::json!({
            "op": "answer", "workspace": "/ws", "agent": "c-1",
            "verdict": "pass", "scope": scope
        }))
    };
    assert_eq!(
        said(serde_json::json!("everywhere")),
        Err("answer: unknown scope \"everywhere\"; say call, conversation or workspace".to_owned())
    );
    assert!(said(serde_json::json!(1)).is_err(), "not a string");
    assert!(
        decode(&serde_json::json!({
            "op": "answer", "workspace": "/ws", "agent": "c-1", "verdict": "pass"
        }))
        .is_err(),
        "absent is not a default"
    );
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
