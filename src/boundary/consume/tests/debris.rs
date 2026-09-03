//! The crash-debris sweep's tables (§8.5, bl-d1f1): a dead claimant's gesture
//! is answered **in doubt** on its reply slot — never re-run — a live claim
//! and an answered one are left alone, and the depositor's poll terminates
//! with the sentence.

use super::*;
use crate::boundary::consume::sweep;

/// The crash, replayed for real: deposit, claim through the real claim (which
/// takes the lock), then drop the guard without ever replying — which is
/// byte-for-byte what a claimant dying between claim and reply leaves behind.
/// The sweep answers the reply slot with the in-doubt refusal and logs the
/// `gesture-debris` row; the gesture is NOT re-run.
#[test]
fn a_dead_claimants_gesture_is_answered_in_doubt_not_re_run() {
    let root = tempdir().unwrap();
    deposit::deposit(root.path(), "g-died", &json!({"op": "ack"})).unwrap();
    drop(deposit::claim(root.path(), "g-died").unwrap());
    assert_eq!(sweep(root.path(), "T1"), 1);
    let reply = deposit::read_reply(root.path(), "g-died").unwrap();
    assert_eq!(reply["ok"], false);
    let error = reply["error"].as_str().unwrap();
    assert!(error.contains("in doubt"), "{error}");
    assert!(
        error.contains("read the world"),
        "the refusal carries the recovery contract: {error}"
    );
    let ops = crate::opslog::tail(root.path(), 8);
    assert_eq!(
        ops.len(),
        1,
        "the debris row is durable, and it is the ONLY \
                row — an /ack that had re-run would have left its own: {ops:?}"
    );
    assert_eq!(ops[0].argv, vec!["yog-step", "gesture-debris"]);
    assert_eq!(
        sweep(root.path(), "T2"),
        0,
        "answered debris stays answered"
    );
}

/// A claim someone is holding is work in flight, not debris: the sweep must
/// skip it, because "the client asked nothing again" is exactly the state a
/// slow gesture is in.
#[test]
fn a_live_claim_is_never_swept() {
    let root = tempdir().unwrap();
    deposit::deposit(root.path(), "g-slow", &json!({"op": "ack"})).unwrap();
    let held = deposit::claim(root.path(), "g-slow").unwrap();
    assert_eq!(sweep(root.path(), "T1"), 0);
    assert!(
        deposit::read_reply(root.path(), "g-slow").is_none(),
        "no reply was forged for work in flight"
    );
    drop(held);
}

/// The ordinary answered audit — a claimed file with a parsed reply beside it
/// — is not debris either, and a world with no claimed dir sweeps to nothing.
#[test]
fn answered_audit_and_an_empty_world_sweep_to_nothing() {
    let root = tempdir().unwrap();
    assert_eq!(
        sweep(root.path(), "T1"),
        0,
        "no claimed dir is the empty set"
    );
    let d = deps(root.path());
    deposit::deposit(root.path(), "q-1", &json!({"op": "balls"})).unwrap();
    assert_eq!(consume(&d, &mut ui(), "T1", 100), 1);
    assert_eq!(
        sweep(root.path(), "T2"),
        0,
        "an answered claim is audit, not debris"
    );
}

/// The reply-write failure inside the sweep leaves the same `gesture-reply`
/// row the pass's own failure does (INV-2: no error class dropped) — and the
/// debris stays unanswered rather than counted.
#[test]
fn an_unwritable_in_doubt_reply_leaves_its_own_step_failure_row() {
    let root = tempdir().unwrap();
    deposit::deposit(root.path(), "g-died", &json!({"op": "ack"})).unwrap();
    drop(deposit::claim(root.path(), "g-died").unwrap());
    // A regular file where `replies/` must be a directory: the write fails.
    // (No mint ran here, so the dir was never created — the file just lands.)
    let replies = deposit::gestures_dir(root.path()).join("replies");
    std::fs::write(&replies, b"x").unwrap();
    assert_eq!(sweep(root.path(), "T1"), 0);
    let ops = crate::opslog::tail(root.path(), 8);
    assert!(
        ops.iter()
            .any(|e| e.argv.get(1).is_some_and(|s| s == "gesture-reply")),
        "{ops:?}"
    );
}
