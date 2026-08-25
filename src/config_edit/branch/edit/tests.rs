//! Tests for the config-branch edit half (§9.3): the pure argv/env
//! composition and the `lernie config` drive — a recorder-script fake proving
//! `EDITOR`/`YOG_EDIT_SRC`/argv land and the outcome reaches `ops.jsonl`.
//!
//! The staging dir and its §5.2 sweep are `super::staging`'s own corpus.

use super::*;
use crate::cli_outbound::Cli;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use tempfile::tempdir;

#[test]
fn editor_env_value_appends_flag_and_quotes_the_binary() {
    assert_eq!(
        editor_env_value(Path::new("/opt/yog")),
        "'/opt/yog' --editor-apply"
    );
    // A spaced path stays one argv element through lernie's word-splitting.
    assert_eq!(
        editor_env_value(Path::new("/home/x y/yog")),
        "'/home/x y/yog' --editor-apply"
    );
    // A literal quote in the path is escaped (`'\''`), not left dangling.
    assert_eq!(
        editor_env_value(Path::new("/o'd/yog")),
        "'/o'\\''d/yog' --editor-apply"
    );
}

fn plan(origin: &EditOrigin) -> EditPlan {
    EditPlan::compose(
        Path::new("/opt/yog"),
        Path::new("/ws"),
        "default",
        origin,
        Path::new("/state/stage/7-0"),
    )
}

#[test]
fn compose_advance_is_config_ws_name() {
    let p = plan(&EditOrigin::Advance);
    assert_eq!(p.argv(), ["config", "/ws", "default"]);
    assert_eq!(
        p.env(),
        [
            ("EDITOR", "'/opt/yog' --editor-apply"),
            ("YOG_EDIT_SRC", "/state/stage/7-0"),
        ]
    );
}

#[test]
fn compose_fork_adds_from_source() {
    let p = plan(&EditOrigin::Fork {
        source: "base".to_string(),
    });
    assert_eq!(p.argv(), ["config", "/ws", "default", "--from", "base"]);
}

#[test]
fn compose_orphan_adds_orphan_flag() {
    let p = plan(&EditOrigin::Orphan);
    assert_eq!(p.argv(), ["config", "/ws", "default", "--orphan"]);
}

/// A recorder `lernie` that logs `EDITOR`/`YOG_EDIT_SRC`/argv to `log`,
/// prints canned stdout/stderr, and exits `code`. Caller holds `SPAWN_LOCK`.
fn recorder(dir: &Path, log: &Path, code: i32) -> PathBuf {
    let path = dir.join("lernie");
    let body = format!(
        "#!/bin/sh\n{{ printf 'EDITOR=%s\\n' \"$EDITOR\"; \
         printf 'SRC=%s\\n' \"$YOG_EDIT_SRC\"; \
         for a in \"$@\"; do printf 'ARG=%s\\n' \"$a\"; done; }} >> {}\n\
         printf 'out-text'\nprintf 'err-text' 1>&2\nexit {}\n",
        log.display(),
        code
    );
    fs::write(&path, body).unwrap();
    let mut perms = fs::metadata(&path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&path, perms).unwrap();
    path
}

#[test]
fn drive_lands_env_and_argv_and_streams_the_outcome_to_ops_jsonl() {
    let dir = tempdir().unwrap();
    let log = dir.path().join("invocation.log");
    let bin = recorder(dir.path(), &log, 0);
    let state = dir.path().join("state");
    let p = EditPlan::compose(
        Path::new("/opt/yog"),
        Path::new("/ws"),
        "default",
        &EditOrigin::Orphan,
        Path::new("/state/stage/7-0"),
    );
    let entry = drive(
        &Cli::new(&bin),
        Path::new("/ws"),
        &p,
        "T0",
        &state,
        Origin::World,
    );

    // The child saw the shim EDITOR, the staging dir, and the config argv.
    assert_eq!(
        fs::read_to_string(&log).unwrap(),
        "EDITOR='/opt/yog' --editor-apply\nSRC=/state/stage/7-0\n\
         ARG=config\nARG=/ws\nARG=default\nARG=--orphan\n"
    );
    // The returned + logged outcome carries the binary-prefixed argv, cwd,
    // exit, and streams.
    assert_eq!(entry.exit, 0);
    assert_eq!(entry.stdout, "out-text");
    assert_eq!(entry.stderr, "err-text");
    assert_eq!(entry.cwd, "/ws");
    assert_eq!(entry.argv[0], bin.display().to_string());
    assert_eq!(&entry.argv[1..], ["config", "/ws", "default", "--orphan"]);
    let logged = opslog::tail(&state, 10);
    assert_eq!(logged, vec![entry]);
}

#[test]
fn drive_records_spawn_failure_as_a_negative_exit() {
    let dir = tempdir().unwrap();
    let state = dir.path().join("state");
    let p = plan(&EditOrigin::Advance);
    let entry = drive(
        &Cli::new("/definitely/not/a/real/lernie-xyz"),
        Path::new("/ws"),
        &p,
        "T0",
        &state,
        Origin::World,
    );
    assert_eq!(entry.exit, -1);
    assert!(!entry.stderr.is_empty());
    assert_eq!(opslog::tail(&state, 10).len(), 1);
}

#[test]
fn drive_records_a_signal_death_as_a_negative_exit() {
    // lernie killed mid-run exits by signal, not code: the outcome records
    // -1, not a spurious success.
    let dir = tempdir().unwrap();
    let bin = dir.path().join("lernie");
    fs::write(&bin, "#!/bin/sh\nkill -TERM $$\n").unwrap();
    fs::set_permissions(&bin, fs::Permissions::from_mode(0o755)).unwrap();
    let state = dir.path().join("state");
    let p = plan(&EditOrigin::Advance);
    let entry = drive(
        &Cli::new(&bin),
        Path::new("/ws"),
        &p,
        "T0",
        &state,
        Origin::World,
    );
    assert_eq!(entry.exit, -1);
}
