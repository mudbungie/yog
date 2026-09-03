//! The generator's own tests: the regeneration writes what the gate demands,
//! and the standing record refuses a shape that moved under a standing
//! protocol version.

use std::fs;
use std::path::Path;

use serde_json::{Value, json};
use tempfile::tempdir;

use super::ledger::{Ledger, advance, signature};
use super::{Shape, canonical, committed, destination, protocol, run, shapes, store};

/// **The drift gate.** A boundary change that alters an emitted byte fails
/// here, and the sentence carries both halves of the remedy. It is also the
/// regeneration: `make corpus` runs this one test with the destination named,
/// so the bytes the gate demands are the bytes it would write.
#[test]
fn gate() {
    let verdict = run(destination(), &committed());
    assert_eq!(verdict, Ok(()), "{verdict:?}");
}

fn blessed(dir: &Path) {
    assert_eq!(run(Some(dir.to_owned()), dir), Ok(()));
}

/// The round trip the corpus itself is: write, then verify — and verifying a
/// second time is still clean, so the generator is idempotent. The record's own
/// bytes are the sharp half: a regeneration at an unchanged protocol with
/// unchanged signatures restamps nothing (bl-00de).
#[test]
fn a_regenerated_corpus_passes_its_own_gate_twice() {
    let dir = tempdir().expect("scratch");
    blessed(dir.path());
    assert_eq!(run(None, dir.path()), Ok(()));
    let record = fs::read_to_string(dir.path().join("shapes.json")).expect("record");
    blessed(dir.path());
    assert_eq!(run(None, dir.path()), Ok(()));
    assert_eq!(
        fs::read_to_string(dir.path().join("shapes.json")).expect("record"),
        record,
        "a no-op regeneration is byte-identical"
    );
}

/// The whole surface is present, both halves, and every fixture is stamped.
#[test]
fn every_shape_is_a_stamped_file_of_frames() {
    let dir = tempdir().expect("scratch");
    blessed(dir.path());
    let all = shapes();
    assert!(all.len() > 80, "{} shapes", all.len());
    for shape in &all {
        let text = fs::read_to_string(dir.path().join(shape.path())).expect("fixture");
        let value = serde_json::from_str::<Value>(&text).expect("canonical json");
        assert_eq!(value["protocol"], json!(protocol()), "{}", shape.key());
        assert_eq!(value["shape"], json!(shape.name), "{}", shape.key());
        assert!(!shape.frames.is_empty(), "{} has no frames", shape.key());
    }
    // Both directions are represented, and the refused envelope with them.
    assert!(all.iter().any(|s| s.key() == "request/message"));
    assert!(all.iter().any(|s| s.key() == "reply/refusal"));
}

/// A stale corpus names its files and both halves of the remedy.
#[test]
fn a_stale_fixture_and_an_orphan_are_both_named() {
    let dir = tempdir().expect("scratch");
    blessed(dir.path());
    fs::write(dir.path().join("request/ack.json"), "{}\n").expect("tamper");
    fs::write(dir.path().join("reply/ghost.json"), "{}\n").expect("orphan");
    let refusal = run(None, dir.path()).expect_err("stale");
    assert!(refusal.contains("request/ack.json"), "{refusal}");
    assert!(refusal.contains("reply/ghost.json"), "{refusal}");
    assert!(refusal.contains("make corpus"), "{refusal}");
    assert!(refusal.contains("PROTOCOL"), "{refusal}");
    // And the regeneration repairs both — the orphan is deleted, not left.
    blessed(dir.path());
    assert!(!dir.path().join("reply/ghost.json").exists());
    assert_eq!(run(None, dir.path()), Ok(()));
}

/// One `ack` shape whose signature has gained `reason`.
fn moved_ack() -> Shape {
    Shape {
        direction: "request",
        name: "ack".to_owned(),
        frames: vec![json!({ "op": "ack", "reason": "why" })],
    }
}

/// A record generated at `protocol` holding one `ack` stamped `since`.
fn recorded(protocol: u32, since: u32) -> Ledger {
    Ledger::read(&canonical(&json!({
        "protocol": protocol,
        "shapes": { "request/ack": { "since": since, "signature": ["/op:string", ":object"] } },
    })))
}

/// **The rule, mechanically**: a wire-visible shape that changed while the
/// protocol version stood still is refused, and the sentence says both what to
/// bump and what to run. The comparison is against the record's OWN version,
/// never the shape's `since` — so an old stamp is no licence (bl-00de).
#[test]
fn a_changed_shape_at_an_unbumped_version_demands_the_bump() {
    let previous = recorded(protocol(), 1);
    let refusal = advance(&[moved_ack()], &previous, protocol())
        .err()
        .expect("refused");
    assert!(refusal.contains("request/ack"), "{refusal}");
    assert!(refusal.contains("src/wire/hello.rs"), "{refusal}");
    assert!(refusal.contains("make corpus"), "{refusal}");
}

/// And the bump stamps the NEW number, not the stale one the shape carried.
/// This is the drift the old per-shape test let through: a shape edited at a
/// version later found spent kept the pre-bump stamp forever.
#[test]
fn a_change_after_a_bump_stamps_the_new_number() {
    let next = advance(&[moved_ack()], &recorded(12, 11), 13).expect("lawful across a bump");
    assert_eq!(next.protocol, 13);
    assert_eq!(next.shapes["request/ack"].since, 13);
}

/// A shape that vanished is the same offence: a spelling in use stopped being
/// spelled, which is a meaning change however few bytes it moves.
#[test]
fn a_vanished_shape_demands_the_bump_too() {
    let previous = Ledger::read(&canonical(&json!({
        "protocol": protocol(),
        "shapes": { "request/gone": { "since": 1, "signature": [":object"] } },
    })));
    let refusal = advance(&[], &previous, protocol()).err().expect("refused");
    assert!(refusal.contains("request/gone"), "{refusal}");
}

/// And the bump is the way through: at a higher version the same change is
/// lawful, and the record re-stamps only what moved.
#[test]
fn a_bump_lets_the_change_through_and_leaves_the_still_shapes_alone() {
    let moved = Shape {
        direction: "request",
        name: "ack".to_owned(),
        frames: vec![json!({ "op": "ack", "reason": "why" })],
    };
    let still = Shape {
        direction: "request",
        name: "scan".to_owned(),
        frames: vec![json!({ "op": "scan" })],
    };
    let previous = Ledger::read(&canonical(&json!({
        "protocol": 1,
        "shapes": {
            "request/ack": { "since": 1, "signature": ["/op:string", ":object"] },
            "request/scan": { "since": 1, "signature": ["/op:string", ":object"] },
        },
    })));
    let next = advance(&[moved, still], &previous, 2).expect("lawful at a higher version");
    assert_eq!(next.protocol, 2);
    assert_eq!(next.shapes["request/ack"].since, 2, "the shape that moved");
    assert_eq!(
        next.shapes["request/scan"].since, 1,
        "the shape that did not"
    );
}

/// A **new** shape is not a bump — REMOTE is explicit that strict decode
/// already refuses an unknown verb in band — so it lands at the standing
/// version.
#[test]
fn a_new_shape_lands_at_the_standing_version() {
    let fresh = Shape {
        direction: "request",
        name: "novel".to_owned(),
        frames: vec![json!({ "op": "novel" })],
    };
    let next = advance(&[fresh], &Ledger::read(""), protocol()).expect("additive");
    assert_eq!(next.shapes["request/novel"].since, protocol());
}

/// The signature is field paths and their types: another sample of the same
/// shape moves nothing, a renamed field moves it.
#[test]
fn a_signature_reads_fields_and_not_bytes() {
    let one = signature(&[json!({ "op": "stop", "children": true, "tags": ["a"] })]);
    let two = signature(&[
        json!({ "op": "stop", "children": false, "tags": [] }),
        json!({ "op": "stop", "children": true, "tags": ["a", "b"] }),
    ]);
    assert_eq!(one, two, "samples are not shapes");
    assert!(one.contains(&"/children:bool".to_owned()), "{one:?}");
    assert!(one.contains(&"/tags/[]:string".to_owned()), "{one:?}");
    let renamed = signature(&[json!({ "op": "stop", "kids": true, "tags": ["a"] })]);
    assert_ne!(one, renamed, "a renamed field is a moved shape");
    // Null is a type of its own: absent and present-as-null are different facts.
    assert_eq!(
        signature(&[json!({ "at": Value::Null })]),
        vec!["/at:null", ":object"]
    );
}

/// An unreadable record is an empty one, and a destination that cannot be
/// written names the failure rather than swallowing it.
#[test]
fn an_unreadable_record_is_empty_and_an_unwritable_destination_refuses() {
    assert_eq!(Ledger::read("not json").protocol, 0);
    let dir = tempdir().expect("scratch");
    let blocked = dir.path().join("file");
    fs::write(&blocked, "").expect("seed");
    assert!(store::bless(&blocked.join("under")).is_err());
}
