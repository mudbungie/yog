//! The task-store gate's regression suite (bl-1043, rescoped bl-1007).
//!
//! `bl` keeps this project's balls in a SEPARATE repository — `tasks/*.md` on
//! `balls/tasks`, pushed to the same remote as the source — so ball bodies are
//! published prose that `make leak-scan` has never seen: it reads the index of
//! *this* tree. `scripts/yog-leak-gate` closes that at the one moment the ball
//! exists and has not been published (`<op>.post`, before `bl-tracker` pushes),
//! by running the repo's own scanner over WHAT THE OP WROTE — its own commit,
//! blobs plus message. Not the store: bl-1043 scanned the whole checkout, and
//! one polluted body then refused every agent's every `bl` op there, `create`
//! included, so the defect about the wedge could not be filed (bl-1007). Tests
//! 1 and 2 are the two halves of that scope.
//!
//! bl-167d's defect was a gate with no test that it caught anything. These
//! tests drive the REAL plugin and the REAL scanner over throwaway stores, and
//! the probe material is the scanner's own fixtures — never restated here, so
//! examples of a leak live in exactly one directory (`scripts/leak-fixtures/`,
//! where `--self-test` holds every line to the `notreal` marker).
//!
//! Deliberately untested because deliberately unpromised: that the gate cannot
//! be bypassed. It can — `bl conf remove <op>.post yog-leak-gate`, or a hand
//! `git push` in the store clone, exactly as `--no-verify` defeats the source
//! hook. `.github/workflows/store-scan.yml` is the half the author cannot
//! switch off AND the half still asking the whole-ref question, and it runs
//! after the push; test 8 pins that chain, AGENTS.md states the boundary.

#![allow(clippy::unwrap_used)]
// The harness half. `#[path]` because this file IS the test target's crate
// root, so a bare `mod` would resolve to `tests/harness.rs` — and a second
// top-level `tests/*.rs` is a second test binary, not a module.
#[path = "leak_store_gate/harness.rs"]
mod harness;
use harness::{fixture, gate, git, op, payload, repo, scan_direct, store};

use std::fs;
use std::os::unix::fs::PermissionsExt;

// 1. The headline: a ball body carrying leak material refuses the op that
//    would publish it, and says which ball and which rule.
#[test]
fn a_leaking_ball_body_refuses_the_op_that_would_publish_it() {
    let dir = store();
    op(dir.path(), "bl-0001", &fixture("clean.txt"), "a clean ball");
    let leak = fixture("quoted-dialogue.txt");
    let sealed = op(dir.path(), "bl-0002", &leak, "a ball with pasted talk");
    let (ok, out, err) = gate(&["update", "post"], &payload(dir.path(), &sealed));
    assert!(!ok, "the gate passed an op it must refuse:\n{err}");
    assert!(out.is_empty(), "stdout is the user channel: {out:?}");
    for expected in ["[quoted-dialogue]", "tasks/bl-0002.md", "REFUSED", "own"] {
        assert!(err.contains(expected), "no {expected:?} in:\n{err}");
    }
    // One table, one mechanism: the plugin's findings ARE the scanner's. A
    // second copy of the rules for the store would drift from this one.
    for line in scan_direct(dir.path(), &["--commit", &sealed])
        .lines()
        .filter(|l| l.contains(" ["))
    {
        assert!(err.contains(line), "the plugin lost {line:?}:\n{err}");
    }
}

// 2. The bl-1007 half, and the reason this gate is scoped at all: the store
//    checkout is shared and long-lived, so somebody ELSE's polluted body must
//    not refuse your op. It refused every op in the checkout, `create`
//    included — which is how a wedge outlives the attempt to file it.
#[test]
fn another_agents_polluted_ball_does_not_refuse_this_op() {
    let dir = store();
    op(dir.path(), "bl-0003", &fixture("home-path.txt"), "not mine");
    let mine = op(dir.path(), "bl-0004", &fixture("clean.txt"), "mine, clean");
    let (ok, out, err) = gate(&["update", "post"], &payload(dir.path(), &mine));
    assert!(ok, "a clean op was refused for another ball:\n{err}");
    assert!(out.is_empty(), "stdout is the user channel: {out:?}");
    assert!(
        !err.contains("bl-0003"),
        "a foreign ball was judged:\n{err}"
    );
    // Not lost, reassigned: the standing state is the daily whole-ref scan's
    // question (test 8), and the tree mode still answers it here.
    let whole = scan_direct(dir.path(), &[]);
    assert!(whole.contains("[home-path]"), "tree mode lost it:\n{whole}");
}

// 3. The scan reads the COMMIT, not the worktree and not the index — bl-167d's
//    property for the source tree, and it matters more here: a store checkout
//    is written by concurrent ops, so both hold other agents' in-flight text.
#[test]
fn the_store_scan_reads_what_was_sealed_not_what_is_on_disk() {
    let dir = store();
    let artifact = fixture("session-artifact.txt");
    let sealed = op(dir.path(), "bl-0005", &artifact, "a sealed ball");
    // A clean copy on disk AND staged over it: neither is what publishes.
    fs::write(dir.path().join("tasks/bl-0005.md"), fixture("clean.txt")).unwrap();
    git(dir.path(), &["add", "-A"]);
    let (ok, _, err) = gate(&["update", "post"], &payload(dir.path(), &sealed));
    assert!(!ok, "the worktree or index copy was scanned:\n{err}");
    assert!(err.contains("[session-artifact]"), "{err}");
}

// 4. A `-m` note is published prose that lands in NO FILE: it is the whole of
//    what a close writes to the store's journal. AGENTS.md governs it like a
//    body, and only the commit-scoped scan can read it.
#[test]
fn a_note_that_lands_only_in_the_commit_message_is_caught() {
    let dir = store();
    let note = String::from_utf8(fixture("home-path.txt")).unwrap();
    let note = note.lines().find(|l| !l.starts_with('#')).unwrap();
    let sealed = op(dir.path(), "bl-0006", &fixture("clean.txt"), note);
    let (ok, _, err) = gate(&["close", "post"], &payload(dir.path(), &sealed));
    assert!(!ok, "the op's own message went unscanned:\n{err}");
    assert!(
        err.contains("[home-path]") && err.contains("message"),
        "{err}"
    );
}

// 5. Fail CLOSED, on either half of the payload. A payload this plugin cannot
//    read is not a clean store, and each refusal names its own removal so a
//    wire change cannot wedge a box with no way out.
#[test]
fn an_unreadable_payload_is_refused_not_waved_through() {
    let dir = store();
    let sealed = op(dir.path(), "bl-0007", &fixture("clean.txt"), "clean");
    let no_store = r#"{"op":"update"}"#.to_owned();
    let no_commit = payload(dir.path(), &sealed).replace("\"commit\"", "\"commit_\"");
    for (broken, expected) in [
        (no_store, "no readable store checkout"),
        (no_commit, "no store commit"),
    ] {
        let (ok, _, err) = gate(&["update", "post"], &broken);
        assert!(!ok, "an unscanned op passed:\n{err}");
        assert!(err.contains(expected), "no {expected:?} in:\n{err}");
        assert!(err.contains("bl conf remove"), "no escape named:\n{err}");
    }
}

// 6. Why `post` and not `pre`: at `pre` the task file is not written yet, so a
//    gate there scans the previous state and passes the body being added.
//    Every phase but `post` abstains — on the same op test 1 refuses.
#[test]
fn every_phase_but_post_abstains() {
    let dir = store();
    let sealed = op(dir.path(), "bl-0008", &fixture("home-path.txt"), "a body");
    for phase in ["pre", "abort"] {
        let (ok, _, err) = gate(&["update", phase], &payload(dir.path(), &sealed));
        assert!(ok, "the {phase} phase judged an op it cannot see:\n{err}");
    }
    let (ok, _, _) = gate(&["update", "post"], &payload(dir.path(), &sealed));
    assert!(!ok, "the same op passed at post — the probe proves nothing");
}

// 7. The handshake, and the executable bit `bl install --bin` binds against.
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

// 8. The remote half — also the half that still asks the whole-ref question
//    this gate stopped asking — and the rule no gate can enforce. AGENTS.md is
//    where the unmechanizable half lives, the operator's standing permission
//    for the maintainer's own identity included.
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
