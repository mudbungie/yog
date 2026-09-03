//! The trail row's two directions (bl-4d81): the derived readings really cross,
//! `standing` is read back strictly, and the round trip is exact.

use serde_json::json;

use super::{decode, op_row};
use crate::opslog::{OpRow, OpView, Origin, Standing};

fn view(exit: i32, standing: Standing) -> OpView {
    OpView {
        row: OpRow {
            ts: "1700".to_owned(),
            argv: "litany prompt a".to_owned(),
            cwd: "/p".to_owned(),
            exit,
            stdout: String::new(),
            stderr: "boom".to_owned(),
            origin: Origin::Conversation,
        },
        standing,
    }
}

/// The three §7.3 readings ride the row, in the words their single homes give
/// them — a seat paints `exit_label`, never the sentinel behind it.
#[test]
fn the_derived_readings_cross_beside_the_durable_line() {
    let v = op_row(&view(crate::opslog::SYNTHETIC_EXIT, Standing::Live));
    assert_eq!(v["exit"], crate::opslog::SYNTHETIC_EXIT);
    assert_eq!(v["failed"], true);
    assert_eq!(v["exit_label"], "failed to spawn — never started");
    assert_eq!(v["standing"], "live");
    assert_eq!(v["origin"], "conversation");
}

/// Every standing round-trips, and the frame is returned exactly — the two
/// recomputed fields included, which is what lets them have one home.
#[test]
fn every_standing_round_trips_exactly() {
    for standing in [
        Standing::Clean,
        Standing::Detached,
        Standing::Live,
        Standing::Retired,
        Standing::Acked,
    ] {
        let before = view(1, standing);
        let frame = op_row(&before);
        let after = decode(&frame).expect("a frame this codec wrote reads back");
        assert_eq!(after, before);
        assert_eq!(op_row(&after), frame, "and re-encodes byte for byte");
    }
}

/// Strict, like every other decoder here: an unknown word and a missing field
/// each refuse, naming the offender.
#[test]
fn an_unreadable_standing_refuses_by_name() {
    let mut frame = op_row(&view(1, Standing::Live));
    frame["standing"] = json!("shrugging");
    let err = decode(&frame).expect_err("an unknown token is refused");
    assert!(
        err.contains("standing") && err.contains("shrugging"),
        "{err}"
    );

    let missing = json!({ "ts": "1", "argv": "a", "cwd": "/", "exit": 0,
                          "stdout": "", "stderr": "", "origin": "world" });
    assert!(decode(&missing).is_err(), "and so is an absent standing");
    assert!(
        decode(&json!("row")).is_err(),
        "and a row that is not an object"
    );
}

/// `exit` is an `i32` on the row and a JSON number on the wire, so a number no
/// row could have carried is refused rather than truncated.
#[test]
fn an_out_of_range_exit_refuses() {
    let mut frame = op_row(&view(1, Standing::Clean));
    frame["exit"] = json!(i64::from(i32::MAX) + 1);
    let err = decode(&frame).expect_err("out of range");
    assert!(err.contains("out of range"), "{err}");
}
