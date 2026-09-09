//! The generator's own tests: the regeneration writes what the gate demands
//! (this file), and the standing record stamps what the boundary gained and
//! refuses what it lost or re-typed under the published major (`ledger`).

use std::fs;
use std::path::Path;

use serde_json::{Value, json};
use tempfile::tempdir;

use super::ledger::{Ledger, signature};
use super::{committed, destination, protocol, run, shapes, store};

/// The standing record's rule — what a gain, a loss and a re-type each cost.
mod ledger;

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
/// second time is still clean, so the generator is idempotent. The record's
/// own bytes are the sharp half: a regeneration with unchanged signatures
/// restamps nothing and moves no floor.
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

/// The whole surface is present, both halves, and every fixture is stamped
/// with the major the corpus is for.
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
    assert!(one.contains("/children:bool"), "{one:?}");
    assert!(one.contains("/tags/[]:string"), "{one:?}");
    let renamed = signature(&[json!({ "op": "stop", "kids": true, "tags": ["a"] })]);
    assert_ne!(one, renamed, "a renamed field is a moved shape");
    // Null is a type of its own: absent and present-as-null are different facts.
    assert_eq!(
        signature(&[json!({ "at": Value::Null })])
            .into_iter()
            .collect::<Vec<_>>(),
        vec!["/at:null", ":object"]
    );
}

/// An unreadable record is an empty one, and a destination that cannot be
/// written names the failure rather than swallowing it.
#[test]
fn an_unreadable_record_is_empty_and_an_unwritable_destination_refuses() {
    let empty = Ledger::read("not json");
    assert_eq!((empty.protocol, empty.floor, empty.edition()), (0, 0, 0));
    let dir = tempdir().expect("scratch");
    let blocked = dir.path().join("file");
    fs::write(&blocked, "").expect("seed");
    assert!(store::bless(&blocked.join("under")).is_err());
}

/// **The compiled constants and the committed record cannot disagree**
/// (bl-1be7). `build.rs` reads `corpus/shapes.json` and writes `EDITION` — the
/// newest stamp over every shape's signature — and `FLOOR`; the ledger reads
/// the same file, and [`Ledger::edition`] is the same arithmetic in Rust. Two
/// programs computing one fact off one file is the whole hazard, so the test
/// is the fact itself rather than either program's arms: a regeneration that
/// stamps a new edition and a build that did not re-run land here.
#[test]
fn the_hello_states_the_committed_edition_and_floor() {
    let record =
        Ledger::read(&fs::read_to_string(committed().join("shapes.json")).expect("record"));
    assert_eq!(
        crate::wire::hello::EDITION,
        record.edition(),
        "the compiled EDITION is not the record's newest stamp"
    );
    assert_eq!(
        crate::wire::hello::FLOOR,
        record.floor,
        "the compiled FLOOR is not the record's floor"
    );
    // And the record is the one this build's major is for, so the three
    // numbers are read off one file rather than two.
    assert_eq!(record.protocol, protocol());
}
