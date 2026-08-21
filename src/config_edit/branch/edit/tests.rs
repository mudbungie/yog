//! Tests for the config-branch edit half (§9.3): the pure argv/env
//! composition, staging writes and nonce, the `lernie config` drive (a
//! recorder-script fake proving `EDITOR`/`YOG_EDIT_SRC`/argv land and the
//! outcome reaches `ops.jsonl`), and the staging-sweep lifecycle.

use super::*;
use crate::cli_outbound::Cli;
use std::fs;
use std::os::unix::fs::PermissionsExt;
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

#[test]
fn next_nonce_is_unique_and_pid_scoped() {
    let a = next_nonce();
    let b = next_nonce();
    assert_ne!(a, b);
    let pid = std::process::id().to_string();
    assert!(a.starts_with(&format!("{pid}-")), "{a}");
    assert!(b.starts_with(&format!("{pid}-")), "{b}");
}

#[test]
fn stage_files_writes_nested_drafts_under_the_nonce() {
    let dir = tempdir().unwrap();
    let files = vec![
        DraftFile {
            rel_path: "workflow.yaml".into(),
            bytes: b"wf".to_vec(),
        },
        DraftFile {
            rel_path: "souls/coder.md".into(),
            bytes: b"soul".to_vec(),
        },
    ];
    let staged = stage_files(dir.path(), "7-0", &files).unwrap();
    assert_eq!(staged, dir.path().join("7-0"));
    assert_eq!(fs::read(staged.join("workflow.yaml")).unwrap(), b"wf");
    assert_eq!(fs::read(staged.join("souls/coder.md")).unwrap(), b"soul");
}

#[test]
fn stage_files_with_no_drafts_still_creates_the_dir() {
    let dir = tempdir().unwrap();
    let staged = stage_files(dir.path(), "7-1", &[]).unwrap();
    assert!(staged.is_dir());
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

#[test]
fn stale_staging_decides_on_the_24h_boundary() {
    let now = 1_000_000_000;
    let dirs = vec![
        (PathBuf::from("fresh"), now),
        (PathBuf::from("exactly-24h"), now - STALE_SECS),
        (PathBuf::from("stale"), now - STALE_SECS - 1),
    ];
    // Strictly-greater-than-24 h is stale; exactly 24 h is kept.
    assert_eq!(stale_staging(now, &dirs), vec![PathBuf::from("stale")]);
}

#[test]
fn sweep_staging_removes_only_stale_dirs_and_skips_non_dirs() {
    let dir = tempdir().unwrap();
    let stage = dir.path().join("stage");
    fs::create_dir_all(stage.join("nonce-a")).unwrap();
    // A stray non-dir entry and a dangling symlink must be skipped by the
    // enumeration (not-a-dir / un-stat-able), never swept.
    fs::write(stage.join("stray.txt"), b"x").unwrap();
    std::os::unix::fs::symlink(dir.path().join("gone"), stage.join("dangling")).unwrap();

    // now far in the future ⇒ the freshly-created dir is stale and removed.
    let removed = sweep_staging(&stage, i64::MAX / 2);
    assert_eq!(removed, vec![stage.join("nonce-a")]);
    assert!(!stage.join("nonce-a").exists());
    assert!(stage.join("stray.txt").exists());

    // A second dir, swept with now=0 ⇒ nothing is stale, nothing removed.
    fs::create_dir_all(stage.join("nonce-b")).unwrap();
    assert!(sweep_staging(&stage, 0).is_empty());
    assert!(stage.join("nonce-b").exists());
}

#[test]
fn sweep_staging_on_a_missing_root_is_a_noop() {
    let dir = tempdir().unwrap();
    assert!(sweep_staging(&dir.path().join("nope"), i64::MAX / 2).is_empty());
}
