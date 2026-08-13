//! The task-store gate's regression suite (bl-1043).
//!
//! `bl` keeps this project's balls in a SEPARATE repository — `tasks/*.md` on
//! `balls/tasks`, pushed to the same remote as the source — so ball bodies are
//! published prose that `make leak-scan` has never seen: it reads the index of
//! *this* tree. `scripts/yog-leak-gate` closes that at the one moment the ball
//! exists and has not been published (`<op>.post`, before `bl-tracker` pushes),
//! by running the repo's own scanner over the store checkout.
//!
//! bl-167d's defect was a gate with no test that it caught anything. These
//! tests drive the REAL plugin and the REAL scanner over throwaway stores, and
//! the probe material is the scanner's own fixtures — never restated here, so
//! examples of a leak still live in exactly one directory
//! (`scripts/leak-fixtures/`, where `--self-test` holds every line to the
//! `notreal` marker).
//!
//! What is deliberately untested, because it is deliberately unpromised: that
//! the gate cannot be bypassed. It can — `bl conf remove <op>.post
//! yog-leak-gate`, or a hand `git push` in the store clone, exactly as
//! `--no-verify` defeats the source hook. `.github/workflows/store-scan.yml` is
//! the half the author cannot switch off, and it runs after the push; the last
//! test pins that chain, and AGENTS.md states the boundary.

#![allow(clippy::unwrap_used)]

use std::fs;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Stdio;

use tempfile::TempDir;

/// This repository, whose plugin, scanner and fixtures are the subject.
fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// One of the scanner's declared fixtures, verbatim.
fn fixture(name: &str) -> Vec<u8> {
    fs::read(repo().join("scripts/leak-fixtures").join(name)).unwrap()
}

fn git(dir: &Path, args: &[&str]) {
    let status = yog::git_env::git()
        .current_dir(dir)
        .args(args)
        .status()
        .unwrap();
    assert!(status.success(), "git {args:?}");
}

/// A throwaway task store: a git repository holding `tasks/<id>.md`, staged.
/// It carries no `scripts/` — the store never does, which is the point: the
/// scanner brings its own rule table and judges whatever tree it is run in.
fn store(balls: &[(&str, Vec<u8>)]) -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    git(dir.path(), &["init", "-q", "-b", "balls/tasks", "."]);
    for (id, body) in balls {
        let at = dir.path().join("tasks").join(format!("{id}.md"));
        fs::create_dir_all(at.parent().unwrap()).unwrap();
        fs::write(at, body).unwrap();
    }
    git(dir.path(), &["add", "-A"]);
    dir
}

/// The §7 payload balls pipes to a plugin, carrying the store checkout.
fn payload(store: &Path) -> String {
    format!(
        r#"{{"op":"update","phase":"post","actor":"tester",
            "binding":{{"landing":"/nowhere","tasks_branch":"balls/tasks",
                        "store":"{}","invocation_path":"/nowhere"}}}}"#,
        store.display()
    )
}

/// Run the plugin as balls runs it: `<op> <phase>` on argv, the payload on
/// stdin, and a working directory that is NOT the store — a plugin is
/// dispatched in the change worktree, so a gate that read cwd would scan the
/// wrong repository. Returns (ok, stdout, stderr).
fn gate(args: &[&str], payload: &str) -> (bool, String, String) {
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

/// The scanner's own verdict on the same store, for the no-drift comparison.
fn scan_direct(store: &Path) -> String {
    let out = yog::git_env::command(Path::new("bash"))
        .current_dir(store)
        .arg(repo().join("scripts/leak-scan.sh"))
        .output()
        .unwrap();
    String::from_utf8(out.stderr).unwrap()
}

// 1. The headline: a ball body carrying leak material refuses the op that
//    would publish it, and says which ball and which rule.
#[test]
fn a_leaking_ball_body_refuses_the_op_that_would_publish_it() {
    let dir = store(&[
        ("bl-0001", fixture("quoted-dialogue.txt")),
        ("bl-0002", fixture("home-path.txt")),
        ("bl-0003", fixture("clean.txt")),
    ]);
    let (ok, out, err) = gate(&["update", "post"], &payload(dir.path()));
    assert!(!ok, "the gate passed a store it must refuse:\n{err}");
    assert!(out.is_empty(), "stdout is the user channel: {out:?}");
    for expected in [
        "[quoted-dialogue]",
        "[home-path]",
        "tasks/bl-0001.md",
        "tasks/bl-0002.md",
        "REFUSED",
        "rolled back",
    ] {
        assert!(err.contains(expected), "no {expected:?} in:\n{err}");
    }
    assert!(!err.contains("bl-0003"), "a clean ball was flagged:\n{err}");
    // One table, one mechanism: the plugin's findings ARE the scanner's. A
    // second copy of the rules for the store would drift from this one.
    for line in scan_direct(dir.path()).lines().filter(|l| l.contains(" [")) {
        assert!(err.contains(line), "the plugin lost {line:?}:\n{err}");
    }
}

// 2. The other direction. A gate that cries wolf gets unwired, and an unwired
//    gate is no gate.
#[test]
fn a_clean_store_passes() {
    let dir = store(&[("bl-0004", fixture("clean.txt"))]);
    let (ok, out, err) = gate(&["update", "post"], &payload(dir.path()));
    assert!(ok, "a clean store was refused:\n{err}");
    assert!(out.is_empty(), "stdout is the user channel: {out:?}");
}

// 3. The store scan reads INDEX BLOBS, not the worktree — the property
//    bl-167d landed for the source tree, inherited here because it is the same
//    scanner. A leak staged behind a clean copy on disk is still caught.
#[test]
fn the_store_scan_reads_what_is_committed_not_what_is_on_disk() {
    let dir = store(&[("bl-0005", fixture("session-artifact.txt"))]);
    fs::write(dir.path().join("tasks/bl-0005.md"), fixture("clean.txt")).unwrap();
    let (ok, _, err) = gate(&["update", "post"], &payload(dir.path()));
    assert!(!ok, "the worktree copy was scanned instead:\n{err}");
    assert!(err.contains("[session-artifact]"), "{err}");
}

// 4. Fail CLOSED. A payload the plugin cannot read is not a clean store, and
//    the refusal names its own removal so a wire change cannot wedge a box.
#[test]
fn a_payload_naming_no_store_is_refused_not_waved_through() {
    let (ok, _, err) = gate(&["update", "post"], r#"{"op":"update"}"#);
    assert!(!ok, "an unscanned store passed:\n{err}");
    assert!(err.contains("no readable store checkout"), "{err}");
    assert!(err.contains("bl conf remove"), "no escape named:\n{err}");
}

// 5. Why `post` and not `pre`: at `pre` the task file is not written yet, so a
//    gate there scans the previous state and passes the body being added.
//    Every phase but `post` abstains — on the same store test 1 refuses.
#[test]
fn every_phase_but_post_abstains() {
    let dir = store(&[("bl-0006", fixture("home-path.txt"))]);
    for phase in ["pre", "abort"] {
        let (ok, _, err) = gate(&["update", phase], &payload(dir.path()));
        assert!(ok, "the {phase} phase judged a store it cannot see:\n{err}");
    }
    let (ok, _, _) = gate(&["update", "post"], &payload(dir.path()));
    assert!(
        !ok,
        "the same store passed at post — the probe proves nothing"
    );
}

// 6. The handshake, and the executable bit `bl install --bin` binds against.
#[test]
fn the_handshake_declares_the_ops_the_publisher_runs_on() {
    let (ok, out, err) = gate(&["protocol"], "");
    assert!(ok, "the handshake failed:\n{err}");
    assert!(out.contains(r#""protocol":[1]"#), "{out}");
    // Exactly the ops the landing runs `bl-tracker` on: the gate goes
    // immediately before the publisher, everywhere the publisher runs.
    for op in ["create", "update", "claim", "unclaim", "close", "drop"] {
        assert!(out.contains(&format!("\"{op}\"")), "{op} unserved:\n{out}");
    }
    let mode = fs::metadata(repo().join("scripts/yog-leak-gate"))
        .unwrap()
        .permissions()
        .mode();
    assert!(mode & 0o111 != 0, "the plugin is not executable: {mode:o}");
}

// 7. The remote half, and the rule the gate cannot enforce. The workflow is
//    the only check the agent writing the ball cannot switch off; AGENTS.md is
//    where the unmechanizable half of the rule lives, including the operator's
//    standing permission for the maintainer's own identity.
#[test]
fn the_published_ref_and_the_stated_rule_are_both_pinned() {
    let flow = fs::read_to_string(repo().join(".github/workflows/store-scan.yml")).unwrap();
    for expected in [
        "ref: balls/tasks",
        "working-directory: store",
        "scripts/leak-scan.sh",
        "contents: read",
    ] {
        assert!(flow.contains(expected), "the store scan lost {expected:?}");
    }
    let agents = fs::read_to_string(repo().join("AGENTS.md")).unwrap();
    for expected in [
        "## What may never enter a ball body",
        "are explicitly permitted",
        "bl conf prepend $op.post yog-leak-gate",
        "Verbatim transcript prose",
        "Provider auth state",
    ] {
        assert!(agents.contains(expected), "AGENTS.md lost {expected:?}");
    }
}
