//! The typed setting's two directions (bl-dc3f): every control kind crosses,
//! the bounds ride the number, the fault is absent rather than null, and the
//! decode is strict about all of it.

use serde_json::json;

use super::{decode, setting};
use crate::config_edit::form::{Control, Row};

fn row(control: Control, fault: Option<&str>) -> Row {
    Row {
        entry: "worker".to_owned(),
        name: "provider".to_owned(),
        control,
        help: "the brazen provider row this role dispatches through".to_owned(),
        value: "codex".to_owned(),
        fault: fault.map(str::to_owned),
    }
}

/// Every kind round-trips, bounds included, and re-encodes byte for byte.
#[test]
fn every_control_kind_round_trips_exactly() {
    for control in [
        Control::Provider,
        Control::List,
        Control::Text,
        Control::Number { min: 1, max: 9 },
    ] {
        for fault in [None, Some("no such row")] {
            let before = row(control, fault);
            let frame = setting(&before);
            let after = decode(&frame).expect("a frame this codec wrote reads back");
            assert_eq!(after, before);
            assert_eq!(setting(&after), frame);
        }
    }
}

/// A usable value carries **no** `fault` key at all — the roster's discipline,
/// so a reader never tells "no fault" from "a fault with nothing to say".
#[test]
fn a_usable_value_omits_the_fault_key() {
    let clean = setting(&row(Control::Text, None));
    assert!(clean.get("fault").is_none(), "{clean}");
    assert_eq!(clean["control"]["kind"], "text");
    assert!(
        clean["control"].get("min").is_none(),
        "only a number carries bounds"
    );
    let bounded = setting(&row(Control::Number { min: 2, max: 4 }, None));
    assert_eq!(bounded["control"]["min"], 2);
    assert_eq!(bounded["control"]["max"], 4);
}

/// Strict on every half: the object, the kind word, and a number's bounds —
/// which are refused rather than defaulted, since a control that invented its
/// own range would judge input by a rule the engine never stated.
#[test]
fn an_unreadable_setting_refuses_by_name() {
    let mut frame = setting(&row(Control::Text, None));
    frame["control"] = json!({ "kind": "dial" });
    let err = decode(&frame).expect_err("an unknown kind is refused");
    assert!(err.contains("dial"), "{err}");

    frame["control"] = json!({ "kind": "number", "min": 1 });
    assert!(decode(&frame).is_err(), "a number missing its max refuses");

    frame["control"] = json!("text");
    assert!(decode(&frame).is_err(), "a control that is not an object");

    let mut bare = setting(&row(Control::Text, None));
    bare.as_object_mut().unwrap().remove("control");
    let err = decode(&bare).expect_err("an absent control refuses");
    assert!(err.contains("control"), "{err}");

    assert!(decode(&json!("setting")).is_err(), "and a non-object row");
    assert!(
        decode(&json!({ "entry": "w" })).is_err(),
        "and one missing every other field"
    );
}
