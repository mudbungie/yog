//! `ui_state` tests: forgiving load, seen watermarks, echo/adopt, the
//! write-through atomic save and its elision, the scalar accessors,
//! unknown-key preservation, and startup-focus derivation.

use super::*;
use std::path::Path;
use tempfile::tempdir;

/// A handle on `dir/ui.json`.
pub(super) fn mk(dir: &Path) -> UiState {
    UiState::open(dir.join("ui.json"))
}

/// Load a handle from `bytes`. `open` reads the file synchronously at
/// construction, so the tempdir is free to drop once it returns (these cases
/// never write back).
pub(super) fn load(bytes: &[u8]) -> UiState {
    let d = tempdir().unwrap();
    let p = d.path().join("ui.json");
    std::fs::write(&p, bytes).unwrap();
    UiState::open(p)
}

/// One seen mark as [`UiState::record_seen`] takes them.
pub(super) fn mark(kind: SeenKind, oid: &str) -> Vec<(SeenKind, String)> {
    vec![(kind, oid.to_string())]
}

#[test]
fn missing_file_is_default_and_no_echo() {
    let d = tempdir().unwrap();
    let ui = mk(d.path());
    assert!(ui.pinned().is_empty());
    assert!(ui.identity_last_used().is_none());
    assert!(!ui.is_seen(SeenKind::Notify, "/w", "a", "x"));
    assert!(!ui.is_echo(b"anything")); // no last_hash yet
}

#[test]
fn corrupt_or_nonobject_load_is_default() {
    assert!(load(b"{not json").pinned().is_empty());
    assert!(load(b"[1,2,3]").pinned().is_empty());
}

/// The point of bl-b54e: a mutator lands on disk before it returns. No flush,
/// no tick, no exit hook — nothing between the gesture and the file.
#[test]
fn every_mutation_is_on_disk_when_it_returns() {
    let d = tempdir().unwrap();
    let p = d.path().join("ui.json");
    let mut ui = UiState::open(p.clone());
    // After each gesture the file already holds exactly this document —
    // `is_echo` over the bytes read back is the byte-identity assertion.
    ui.set_pinned(vec!["/w".into()]);
    assert!(ui.is_echo(&std::fs::read(&p).unwrap()), "pin");
    ui.set_collapsed("proj:/x", true);
    assert!(ui.is_echo(&std::fs::read(&p).unwrap()), "collapse");
    ui.set_identity("me");
    assert!(ui.is_echo(&std::fs::read(&p).unwrap()), "identity");
    ui.record_seen("/w", "a", &mark(SeenKind::Notify, "n1"));
    assert!(ui.is_echo(&std::fs::read(&p).unwrap()), "seen");

    let back = std::fs::read_to_string(&p).unwrap();
    assert!(back.contains("/w") && back.contains("proj:/x") && back.contains("\"me\""));
}

/// The coalescing the debounce used to buy: a gesture that changes no byte
/// writes nothing (so a held key does not write per repeat).
#[test]
fn a_no_op_gesture_writes_nothing() {
    let d = tempdir().unwrap();
    let p = d.path().join("ui.json");
    let mut ui = UiState::open(p.clone());
    ui.set_pinned(vec!["/w".into()]);
    std::fs::write(&p, b"sentinel").unwrap(); // any real write clobbers this
    ui.set_pinned(vec!["/w".into()]); // identical bytes ⇒ elided
    ui.record_seen("/w", "a", &[]); // no marks ⇒ elided
    assert_eq!(std::fs::read(&p).unwrap(), b"sentinel");
}

#[test]
fn write_sets_echo_hash() {
    let d = tempdir().unwrap();
    let mut ui = mk(d.path());
    ui.set_identity("me@example.com");
    let bytes = std::fs::read(d.path().join("ui.json")).unwrap();
    assert!(ui.is_echo(&bytes)); // our own write
    assert!(!ui.is_echo(b"different"));
}

/// A failed write is swallowed and leaves the hash alone, so the next mutation
/// retries the whole document (LWW whole-file, never a half-applied delta).
#[test]
fn a_failed_write_is_swallowed_and_retried() {
    let d = tempdir().unwrap();
    let blocked = d.path().join("not-a-dir");
    std::fs::write(&blocked, b"x").unwrap(); // a file where a dir must be
    let mut ui = UiState::open(blocked.join("ui.json"));
    ui.set_identity("me");
    assert_eq!(ui.identity_last_used().as_deref(), Some("me")); // RAM holds
    assert!(!ui.is_echo(b"{}")); // no hash was adopted
    assert_eq!(std::fs::read(&blocked).unwrap(), b"x"); // nothing clobbered
}

#[test]
fn adopt_replaces_and_refreshes_hash() {
    let d = tempdir().unwrap();
    let mut ui = mk(d.path());
    ui.set_identity("old");
    let ext = br#"{"v":1,"identity_last_used":"new","seen":{"/w":{"a":{"notify":"n9"}}}}"#;
    ui.adopt(ext);
    assert_eq!(ui.identity_last_used().as_deref(), Some("new"));
    assert!(ui.is_seen(SeenKind::Notify, "/w", "a", "n9"));
    assert!(ui.is_echo(ext)); // adopted content is now our known state
    ui.adopt(b"garbage"); // corrupt external → default doc
    assert!(ui.identity_last_used().is_none());
}

#[test]
fn transcript_density_knobs_roundtrip_with_the_operator_defaults() {
    let d = tempdir().unwrap();
    let mut ui = mk(d.path());
    // The §11 ruling as defaults: replies open, everything else folded.
    assert!(ui.transcript_expand_responses());
    assert!(!ui.transcript_expand_others());
    ui.set_transcript_expand_responses(false);
    ui.set_transcript_expand_others(true);
    assert!(!ui.transcript_expand_responses());
    assert!(ui.transcript_expand_others());
    // Non-bool values fall back to the defaults (the forgiving read).
    let junk = load(br#"{"transcript_expand_responses":1,"transcript_expand_others":"y"}"#);
    assert!(junk.transcript_expand_responses());
    assert!(!junk.transcript_expand_others());
}

/// The §6 escalation knob (bl-e160): armed by default — the strip is invisible
/// exactly when the operator needs it, so a notifier off until you find its
/// switch is a feature nobody has. Severable: the key alone turns it off, and
/// deleting the key restores the default.
#[test]
fn the_desktop_escalation_knob_is_armed_by_default_and_switched_off_by_one_key() {
    let d = tempdir().unwrap();
    assert!(mk(d.path()).notify_unfocused());
    assert!(!load(br#"{"notify_unfocused":false}"#).notify_unfocused());
    // A hand-edited non-bool is the forgiving read's default, not a refusal.
    assert!(load(br#"{"notify_unfocused":"loud"}"#).notify_unfocused());
}

#[test]
fn unknown_keys_survive_writeback() {
    let d = tempdir().unwrap();
    let p = d.path().join("ui.json");
    std::fs::write(&p, br#"{"v":1,"future_field":{"x":1},"pinned":["/a"]}"#).unwrap();
    let mut ui = UiState::open(p.clone());
    ui.set_identity("me");
    let back = std::fs::read_to_string(&p).unwrap();
    assert!(back.contains("future_field"));
    assert!(back.contains("\"me\""));
}

#[test]
fn write_creates_missing_state_dir_and_cleans_temp() {
    let d = tempdir().unwrap();
    let nested = d.path().join("state").join("yog");
    let mut ui = UiState::open(nested.join("ui.json"));
    ui.set_pinned(vec!["/z".into()]);
    assert!(nested.join("ui.json").exists());
    let leftovers = std::fs::read_dir(&nested)
        .unwrap()
        .filter_map(std::result::Result::ok)
        .filter(|e| {
            e.file_name()
                .to_string_lossy()
                .starts_with(".ui.json.yog-tmp")
        })
        .count();
    assert_eq!(leftovers, 0); // temp renamed away, not left behind
}

#[test]
fn system_clock_is_monotonic_and_stamps_unix_seconds() {
    let c = SystemClock;
    let a = c.now();
    assert!(c.now() >= a);
    // The §4.2 wall-clock field: unix seconds as a string, opaque to opslog.
    // Any real clock is well past the epoch, and the parse is the contract.
    let stamp: u64 = c.stamp().parse().expect("unix seconds");
    assert!(stamp > 1_700_000_000, "a plausible wall clock: {stamp}");
}

#[test]
fn content_hash_is_stable_and_sensitive() {
    assert_eq!(content_hash(b"abc"), content_hash(b"abc"));
    assert_ne!(content_hash(b"abd"), content_hash(b"abc"));
}

#[test]
fn startup_focus_prefers_attention_then_first() {
    let r = ["/a", "/b", "/c"];
    assert_eq!(derive_startup_focus(&r, &["/b"]).as_deref(), Some("/b")); // attention
    assert_eq!(derive_startup_focus(&r, &[]).as_deref(), Some("/a")); // else first
    assert_eq!(derive_startup_focus(&r, &["/zzz"]).as_deref(), Some("/a")); // attention off-roster
    assert_eq!(derive_startup_focus(&[], &["/b"]), None); // empty roster
}

#[test]
fn format_iso8601_pads_every_field() {
    assert_eq!(
        format_iso8601(2026, 8, 2, 0, 24, 26),
        "2026-08-02 00:24:26Z"
    );
    assert_eq!(format_iso8601(1970, 1, 1, 0, 0, 0), "1970-01-01 00:00:00Z");
}

/// bl-61db: the activity row's raw epoch, rendered the same way the chat
/// header renders its id's stamp (bl-16da) — `date -u -d @1785630266` is the
/// independent oracle for this value.
#[test]
fn iso8601_extended_reads_the_activity_row_example() {
    assert_eq!(iso8601_extended(1_785_630_266), "2026-08-02 00:24:26Z");
}

#[test]
fn iso8601_extended_covers_the_epoch_and_a_leap_day() {
    assert_eq!(iso8601_extended(0), "1970-01-01 00:00:00Z");
    // 2000 is a leap year (divisible by 400) — `date -u -d @951868799`.
    assert_eq!(iso8601_extended(951_868_799), "2000-02-29 23:59:59Z");
    // year-end rollover — `date -u -d @1735689599`.
    assert_eq!(iso8601_extended(1_735_689_599), "2024-12-31 23:59:59Z");
}
