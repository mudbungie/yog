//! The doctor's `rendezvous` object (bl-355c): absent reads as `None`, and
//! present, every counter is required and typed — a seat must not paint a
//! loop's standing out of a half-said one.

use serde_json::json;

use super::super::super::super::{Reply, decode};
use super::refuses;

fn doctor(rendezvous: serde_json::Value) -> serde_json::Value {
    json!({ "ok": true, "kind": "doctor", "rows": [], "rendezvous": rendezvous })
}

#[test]
fn an_absent_or_null_rendezvous_is_none() {
    for wire in [
        json!({ "ok": true, "kind": "doctor", "rows": [] }),
        doctor(serde_json::Value::Null),
    ] {
        let Ok(Ok(Reply::Doctor(report))) = decode(&wire) else {
            panic!("{wire} did not decode");
        };
        assert_eq!(report.rendezvous, None);
    }
}

#[test]
fn a_present_rendezvous_is_read_strictly() {
    let full = json!({ "active": true, "published": 1, "calls": 2, "punched": 3,
                       "served": 4, "last_poll_unix": 5 });
    refuses(&doctor(json!(7)), "rendezvous is not an object");
    for key in [
        "active",
        "published",
        "calls",
        "punched",
        "served",
        "last_poll_unix",
    ] {
        let mut half = full.clone();
        half.as_object_mut().expect("object").remove(key);
        refuses(&doctor(half), key);
        let mut wrong = full.clone();
        wrong[key] = json!("x");
        refuses(&doctor(wrong), key);
    }
    refuses(
        &doctor(json!({ "active": true, "published": -1, "calls": 2,
                            "punched": 3, "served": 4, "last_poll_unix": 5 })),
        "published",
    );
}
