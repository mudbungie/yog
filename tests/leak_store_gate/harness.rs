//! **The throwaway store these beats are driven over**, and the two ways a
//! verdict is asked for: the real plugin over a §7 post payload, and the
//! scanner reached directly. Split from the beats at §12's budget on the seam
//! that already exists everywhere in this suite — what a drive *sets up* is
//! one subject, what it must then *see* is another.
//!
//! Nothing here restates leak material: the probe bytes are the scanner's own
//! fixtures, read out of `scripts/leak-fixtures/`, so an example of a leak
//! lives in exactly one directory.

#![allow(clippy::unwrap_used)]

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Stdio;

use tempfile::TempDir;

/// This repository, whose plugin, scanner and fixtures are the subject.
pub(crate) fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// One of the scanner's declared fixtures, verbatim.
pub(crate) fn fixture(name: &str) -> Vec<u8> {
    fs::read(repo().join("scripts/leak-fixtures").join(name)).unwrap()
}

pub(crate) fn git(dir: &Path, args: &[&str]) {
    let status = yog::git_env::git()
        .current_dir(dir)
        .args(args)
        .status()
        .unwrap();
    assert!(status.success(), "git {args:?}");
}

pub(crate) fn head(dir: &Path) -> String {
    let out = yog::git_env::git()
        .current_dir(dir)
        .args(["rev-parse", "HEAD"])
        .output()
        .unwrap();
    String::from_utf8(out.stdout).unwrap().trim().to_owned()
}

/// An empty task store: a git repository on the store branch. It carries no
/// `scripts/` — the store never does, which is the point: the scanner brings
/// its own rule table and judges whatever tree it is run in.
pub(crate) fn store() -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    git(dir.path(), &["init", "-q", "-b", "balls/tasks", "."]);
    git(dir.path(), &["config", "user.email", "nobody@example.com"]);
    git(dir.path(), &["config", "user.name", "nobody"]);
    dir
}

/// One `bl` op: write a ball and seal it, the way every store commit is made.
/// Returns the commit — the §7 `commit` field, and the whole of what this op
/// publishes.
pub(crate) fn op(dir: &Path, id: &str, body: &[u8], message: &str) -> String {
    let at = dir.join("tasks").join(format!("{id}.md"));
    fs::create_dir_all(at.parent().unwrap()).unwrap();
    fs::write(at, body).unwrap();
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", message]);
    head(dir)
}

/// The §7 post payload balls pipes to a plugin: the store checkout, and the
/// commit the op just sealed there.
pub(crate) fn payload(store: &Path, commit: &str) -> String {
    format!(
        r#"{{"op":"update","phase":"post","actor":"tester","commit":"{commit}",
            "previous_commit":"0000000",
            "binding":{{"landing":"/nowhere","tasks_branch":"balls/tasks",
                        "store":"{}","invocation_path":"/nowhere"}}}}"#,
        store.display()
    )
}

/// Run the plugin as balls runs it: `<op> <phase>` on argv, the payload on
/// stdin, and a working directory that is NOT the store — a plugin is
/// dispatched in the change worktree, so a gate that read cwd would scan the
/// wrong repository. Returns (ok, stdout, stderr).
pub(crate) fn gate(args: &[&str], payload: &str) -> (bool, String, String) {
    let mut child = yog::git_env::command(&repo().join("scripts/yog-leak-gate"))
        .current_dir(repo())
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(payload.as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    (
        out.status.success(),
        String::from_utf8(out.stdout).unwrap(),
        String::from_utf8(out.stderr).unwrap(),
    )
}

/// The scanner's own verdict, for the no-drift comparison against the plugin's
/// and for the whole-tree question the plugin no longer asks.
pub(crate) fn scan_direct(store: &Path, args: &[&str]) -> String {
    let out = yog::git_env::command(Path::new("bash"))
        .current_dir(store)
        .arg(repo().join("scripts/leak-scan.sh"))
        .args(args)
        .output()
        .unwrap();
    String::from_utf8(out.stderr).unwrap()
}
