//! The watcher's detection beats: what surfaces, what is ignored, and how
//! rapid or renamed writes coalesce. The polling harness they all run through
//! — the arming observation and the wait budget — is [`wait`].

use super::*;
use std::fs;
use std::time::Duration;
use tempfile::tempdir;

mod wait;
pub(crate) use wait::workspace;
use wait::{expect_touch, poll_until, wait_for, wait_quiet};

#[test]
fn poll_until_returns_the_first_some() {
    let mut calls = 0;
    let got = poll_until(
        || {
            calls += 1;
            (calls == 2).then_some(calls)
        },
        Duration::from_secs(5),
        Duration::from_millis(1),
    );
    assert_eq!(got, Some(2));
}

#[test]
fn poll_until_times_out_to_none() {
    let got = poll_until(
        || None::<()>,
        Duration::from_millis(30),
        Duration::from_millis(5),
    );
    assert_eq!(got, None);
}

#[test]
fn new_errors_on_missing_repo() {
    let root = tempdir().unwrap();
    let err = Watcher::new(&root.path().join("nope")).err().unwrap();
    assert!(err.to_string().contains("filesystem watcher"));
}

#[test]
fn tick_is_empty_when_nothing_changed() {
    let (_dir, root) = workspace();
    let watcher = Watcher::new(&root).unwrap();
    wait_quiet(&watcher);
    assert!(watcher.tick().is_empty());
}

#[test]
fn detects_step_request_creation_at_conv_repo_root() {
    // Step records: <conv-repo>/steps/<conv-id>/<NNN>/, outside every worktree.
    let (_dir, root) = workspace();
    expect_touch(&root, &root.join("steps/abc-1/001/request.json"));
}

#[test]
fn detects_subagent_step_record_at_conv_repo_root() {
    // Subagents share the conv-repo-root `steps/` tree, namespaced by descent.
    let (_dir, root) = workspace();
    expect_touch(&root, &root.join("steps/aa-bb/001/request.json"));
}

#[test]
fn detects_inbox_deposits_at_workspace_root() {
    // Inboxes: `<workspace>/inbox/<agent-id>/` (ARCH §2.11).
    let (_dir, root) = workspace();
    expect_touch(&root, &root.join("inbox/aa-bb/user-001.md"));
}

#[test]
fn detects_goal_md_update_in_an_agent_worktree() {
    let (_dir, root) = workspace();
    expect_touch(&root, &root.join("agents/aa-bb/goal.md"));
}

#[test]
fn detects_removal_under_summary() {
    let (_dir, root) = workspace();
    let target = root.join("agents/aa-bb/summary/001.md");
    fs::create_dir_all(target.parent().unwrap()).unwrap();
    fs::write(&target, b"hi").unwrap();
    let watcher = Watcher::new(&root).unwrap();
    wait_quiet(&watcher);
    fs::remove_file(&target).unwrap();
    let changes = wait_for(&watcher, |c| c.path == target);
    let hit = changes.iter().find(|c| c.path == target).expect("event");
    assert_eq!(hit.kind, ChangeKind::Removed);
}

#[test]
fn ignores_paths_outside_allowlist() {
    let (_dir, root) = workspace();
    let watcher = Watcher::new(&root).unwrap();
    wait_quiet(&watcher);
    fs::write(root.join("README.md"), b"x").unwrap();
    fs::create_dir_all(root.join("random")).unwrap();
    fs::write(root.join("random/x.txt"), b"x").unwrap();
    fs::create_dir_all(root.join("agents/aa-bb/random")).unwrap();
    fs::write(root.join("agents/aa-bb/random/x.txt"), b"x").unwrap();
    std::thread::sleep(Duration::from_millis(200));
    assert!(watcher.tick().is_empty());
}

#[test]
fn coalesces_rapid_writes_to_one_event() {
    let (_dir, root) = workspace();
    fs::create_dir_all(root.join("steps")).unwrap();
    let target = root.join("steps/out.log");
    let watcher = Watcher::new(&root).unwrap();
    wait_quiet(&watcher);
    for i in 0..5 {
        fs::write(&target, format!("line {i}")).unwrap();
    }
    let changes = wait_for(&watcher, |c| c.path == target);
    let hits: Vec<_> = changes.iter().filter(|e| e.path == target).collect();
    assert_eq!(hits.len(), 1, "got {changes:?}");
    assert_eq!(hits[0].kind, ChangeKind::Touched);
}

#[test]
fn coalesces_atomic_rename_to_destination() {
    let (_dir, root) = workspace();
    fs::create_dir_all(root.join("steps/abc/001")).unwrap();
    let tmp = root.join("steps/abc/001/request.json.tmp");
    let final_path = root.join("steps/abc/001/request.json");
    let watcher = Watcher::new(&root).unwrap();
    wait_quiet(&watcher);
    fs::write(&tmp, b"{}").unwrap();
    fs::rename(&tmp, &final_path).unwrap();
    let changes = wait_for(&watcher, |c| c.path == final_path);
    let finals: Vec<_> = changes.iter().filter(|e| e.path == final_path).collect();
    let tmps: Vec<_> = changes.iter().filter(|e| e.path == tmp).collect();
    assert_eq!(finals.len(), 1, "one event for destination: {changes:?}");
    assert_eq!(finals[0].kind, ChangeKind::Touched);
    assert!(tmps.is_empty(), "rename source leaked: {tmps:?}");
}

mod drift;
mod hub;
