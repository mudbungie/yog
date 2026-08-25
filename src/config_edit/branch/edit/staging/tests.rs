//! Tests for the staged half (§9.3 step 1, §5.2 step 5): the nonce, the nested
//! draft writes under it, and the sweep's 24 h boundary — including the entries
//! the enumeration must skip rather than delete.

use super::*;
use std::fs;
use tempfile::tempdir;

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
