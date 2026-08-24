//! The `--line` seat (§8.5): a slash line typed at a terminal reaches the same
//! inbox as an envelope, the context flags are what aim it, and a line the seat
//! cannot complete refuses at the depositor rather than depositing a guess.

use super::*;
use crate::boundary::deposit;
use serde_json::json;
use tempfile::tempdir;

/// The line spelling reaches the same inbox as the envelope (§8.5): read at the
/// seat the flags describe, encoded by the codec, deposited unchanged.
#[test]
fn a_line_deposits_the_envelope_it_spells() {
    let root = tempdir().unwrap();
    let exit = run(
        root.path(),
        &[
            "--project".into(),
            "/proj".into(),
            "--as".into(),
            "alba".into(),
            "/close bl-1".into(),
        ],
        "g-line",
        0,
        &mut no_wait(),
    );
    assert_eq!(exit, TIMEOUT_EXIT, "nothing is running to answer it");
    let pending = deposit::pending(root.path());
    let [(id, path)] = pending.as_slice() else {
        panic!("expected one deposit, got {pending:?}");
    };
    assert_eq!(id, "g-line-0");
    let written: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    assert_eq!(
        written,
        json!({"op": "close", "project": "/proj", "id": "bl-1", "name": "alba"}),
        "the line encodes to the envelope it spells"
    );
}

/// A line the seat cannot complete refuses at the depositor, exactly as a
/// malformed envelope does — the inbox stays clean either way.
#[test]
fn a_line_that_names_no_target_never_deposits() {
    let root = tempdir().unwrap();
    for args in [
        vec!["/scan".to_owned()],
        vec!["--ws".to_owned()],
        vec!["--nope".to_owned(), "x".to_owned(), "/scan".to_owned()],
        vec!["/enhance".to_owned()],
    ] {
        assert_eq!(
            run(root.path(), &args, "g", 1, &mut no_wait()),
            USAGE_EXIT,
            "{args:?}"
        );
    }
    assert!(deposit::pending(root.path()).is_empty());
}

/// The flags are the seat: a `/message` typed at a terminal says where it goes.
#[test]
fn the_context_flags_aim_a_line() {
    let root = tempdir().unwrap();
    run(
        root.path(),
        &[
            "--ws".into(),
            "/ws".into(),
            "--agent".into(),
            "c-1".into(),
            "/message ship it".into(),
        ],
        "g-msg",
        0,
        &mut no_wait(),
    );
    let pending = deposit::pending(root.path());
    let [(_, path)] = pending.as_slice() else {
        panic!("expected one deposit");
    };
    let written: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    assert_eq!(
        written,
        json!({"op": "message", "workspace": "/ws", "agent": "c-1", "content": "ship it"})
    );
}
