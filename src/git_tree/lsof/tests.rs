//! Unit tests for the `lsof -F` codec and the tri-state probe (lsof).

use super::*;
use std::cell::Cell;
use std::path::PathBuf;

const INBOX: &str = "/ws/inbox/20260427T140000Z-aaaa";
const RESPONSE: &str = "/ws/steps/20260427T140000Z-aaaa/003/response.json";

/// A fake runner replaying one recorded output (`None` = a spawn failure /
/// absent lsof). The recorded bytes stand in for real `lsof -F pan` output.
struct FakeLsof {
    output: Option<Vec<u8>>,
    calls: Cell<usize>,
}

/// `LsofProbe` over a fake replaying `out` (owned so `format!` bytes fit).
fn probe(out: Option<Vec<u8>>) -> LsofProbe<FakeLsof> {
    LsofProbe::new(FakeLsof {
        output: out,
        calls: Cell::new(0),
    })
}

impl LsofRunner for FakeLsof {
    fn run(&self, _target: &Path) -> Option<Vec<u8>> {
        self.calls.set(self.calls.get() + 1);
        self.output.clone()
    }
}

fn target(s: &str) -> PathBuf {
    PathBuf::from(s)
}

#[test]
fn writer_open_for_write_is_holder_and_writer() {
    // A file set with `au` (read+write) over the target: held and a writer.
    // Includes an `f` delimiter and an ignored `t` (type) field.
    let out = format!("p4321\nf3\ntREG\nau\nn{RESPONSE}\n");
    let seen = parse(out.as_bytes(), &target(RESPONSE)).unwrap();
    assert!(seen.any_holder && seen.any_writer);
}

#[test]
fn reader_only_is_a_holder_but_not_a_writer() {
    // `ar` (read) over the target: a holder (the lock cares) but no writer.
    let out = format!("p4321\nf3\nar\nn{INBOX}\n");
    let seen = parse(out.as_bytes(), &target(INBOX)).unwrap();
    assert!(seen.any_holder && !seen.any_writer);
}

#[test]
fn write_access_w_counts_as_a_writer() {
    let out = format!("p9\naw\nn{RESPONSE}\n");
    let seen = parse(out.as_bytes(), &target(RESPONSE)).unwrap();
    assert!(seen.any_writer);
}

#[test]
fn a_non_matching_name_is_no_holder() {
    // A process holding some *other* file: not our target. A blank line and
    // the `f`-reset are exercised; the write access must not leak onto the
    // following unrelated name.
    let out = "p4321\nf3\naw\nn/some/other/file\n\nf4\nn/and/another\n";
    let seen = parse(out.as_bytes(), &target(RESPONSE)).unwrap();
    assert!(!seen.any_holder && !seen.any_writer);
}

#[test]
fn access_does_not_leak_across_files_without_an_f_delimiter() {
    // BSD/GNU shape with no `f`: a writer on one file, then the target with
    // no `a` of its own — the reset-on-`n` keeps it from inheriting `w`.
    let out = format!("p7\naw\nn/other\nn{RESPONSE}\n");
    let seen = parse(out.as_bytes(), &target(RESPONSE)).unwrap();
    assert!(seen.any_holder && !seen.any_writer);
}

#[test]
fn empty_output_is_a_definite_no_holder() {
    let seen = parse(b"", &target(RESPONSE)).unwrap();
    assert!(!seen.any_holder && !seen.any_writer);
}

#[test]
fn non_field_output_is_unparseable() {
    // lsof error text on stdout (no `p` set) is Unknown, not Free.
    assert!(parse(b"lsof: WARNING: bad argument\n", &target(RESPONSE)).is_none());
}

#[test]
fn invalid_utf8_is_unparseable() {
    assert!(parse(&[0xff, 0xfe, 0x00], &target(RESPONSE)).is_none());
}

#[test]
fn lock_state_reports_held_free_and_unknown() {
    let held = probe(Some(format!("p1\nf3\nar\nn{INBOX}\n").into_bytes()));
    assert_eq!(held.lock_state(&target(INBOX)), Probe::Held);
    let free = probe(Some(Vec::new()));
    assert_eq!(free.lock_state(&target(INBOX)), Probe::Free);
    let absent = probe(None);
    assert_eq!(absent.lock_state(&target(INBOX)), Probe::Unknown);
}

#[test]
fn writer_state_reports_held_free_and_unknown() {
    let held = probe(Some(format!("p1\nf3\naw\nn{RESPONSE}\n").into_bytes()));
    assert_eq!(held.writer_state(&target(RESPONSE)), Probe::Held);
    // A reader-only holder is Free for the writer question.
    let reader = probe(Some(format!("p1\nf3\nar\nn{RESPONSE}\n").into_bytes()));
    assert_eq!(reader.writer_state(&target(RESPONSE)), Probe::Free);
    // Unparseable stdout degrades to Unknown, never a false Free.
    let garbled = probe(Some(b"garbage\n".to_vec()));
    assert_eq!(garbled.writer_state(&target(RESPONSE)), Probe::Unknown);
    assert_eq!(garbled.runner.calls.get(), 1);
}
