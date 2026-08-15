//! The named fields' own tables: the four `seen` watermarks (record, gate by
//! oid, coerce a wrong-typed slot, all-at-once, and the phantom that
//! materializes nothing), the pin list and the collapse overrides. The file
//! mechanics they end on — forgiving load, echo/adopt, the atomic write — are
//! `super::super`'s.

use super::super::tests::{load, load_pane, mark, mk};
use super::*;
use tempfile::tempdir;

#[test]
fn seen_records_and_gates_by_oid() {
    let d = tempdir().unwrap();
    let mut ui = mk(d.path());
    for (k, oid) in [
        (SeenKind::Notify, "n1"),
        (SeenKind::Stopped, "s1"),
        (SeenKind::Budget, "b1"),
        (SeenKind::Conflicted, "c1"),
    ] {
        ui.record_seen("/w", "a-b", &mark(k, oid));
        assert!(ui.is_seen(k, "/w", "a-b", oid));
        assert!(!ui.is_seen(k, "/w", "a-b", "moved")); // oid mismatch = unseen
    }
    assert!(!ui.is_seen(SeenKind::Notify, "/other", "a-b", "n1"));
    assert!(!ui.is_seen(SeenKind::Notify, "/w", "zzz", "n1"));
}

#[test]
fn record_seen_coerces_wrong_typed_slot() {
    let mut ui = load(br#"{"seen":{"/w":"oops"}}"#);
    ui.record_seen("/w", "a", &mark(SeenKind::Budget, "b1"));
    assert!(ui.is_seen(SeenKind::Budget, "/w", "a", "b1"));
}

/// The whole acknowledgement gesture is one call, hence one write.
#[test]
fn record_seen_takes_every_mark_at_once() {
    let d = tempdir().unwrap();
    let mut ui = mk(d.path());
    ui.record_seen(
        "/w",
        "a",
        &[
            (SeenKind::Notify, "n1".to_string()),
            (SeenKind::Stopped, "s1".to_string()),
        ],
    );
    assert!(ui.is_seen(SeenKind::Notify, "/w", "a", "n1"));
    assert!(ui.is_seen(SeenKind::Stopped, "/w", "a", "s1"));
}

/// A phantom agent (§3.5) contributes no evidence: the descent never happens,
/// so no empty `seen[ws][agent]` slot is materialized.
#[test]
fn record_seen_with_no_marks_materializes_nothing() {
    let d = tempdir().unwrap();
    let p = d.path().join("ui.json");
    let mut ui = UiState::open(p.clone());
    ui.record_seen("/w", "phantom", &[]);
    let back = std::fs::read_to_string(&p).unwrap();
    assert!(!back.contains("phantom"), "no slot for a phantom: {back}");
    assert!(!back.contains("seen"), "no seen map at all: {back}");
}

#[test]
fn pinned_and_collapsed_roundtrip() {
    let d = tempdir().unwrap();
    let mut ui = mk(d.path());
    assert!(!ui.is_collapsed("proj:/none")); // no collapsed key yet
    ui.set_pinned(vec!["/a".into(), "/b".into()]);
    assert_eq!(ui.pinned(), vec!["/a".to_string(), "/b".to_string()]);
    ui.set_collapsed("proj:/x", true);
    assert!(ui.is_collapsed("proj:/x"));
    assert!(!ui.is_collapsed("proj:/y")); // present array, key absent
    ui.set_collapsed("proj:/x", false); // remove
    assert!(!ui.is_collapsed("proj:/x"));
}

#[test]
fn getters_filter_wrong_types() {
    assert_eq!(
        load(br#"{"pinned":["/a",7]}"#).pinned(),
        vec!["/a".to_string()]
    ); // number dropped
    // The collapse set is the PANE's (REMOTE §7), and forgiving on its own terms.
    let mut ui = load_pane(br#"{"collapsed":["k",3]}"#);
    assert!(ui.is_collapsed("k"));
    ui.set_collapsed("m", true); // rebuilds set, dropping the non-string 3
    assert!(ui.is_collapsed("m"));
    assert!(ui.is_collapsed("k"));
}
