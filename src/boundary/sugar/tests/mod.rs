//! The deposit-and-wait sugar's exits (§8.5): never-deposited refusals, the
//! answered path, the timeout that leaves the deposit standing, and the help
//! flag that short-circuits all three. The `--line` seat's own spellings are
//! [`line`], split off at §12's cap on the seam the sugar itself has: an argv
//! envelope and a typed slash line are two ways in to one depositor.

/// A slash line typed at the sugar's seat.
mod line;

use super::*;
use crate::boundary::deposit;
use serde_json::json;
use tempfile::tempdir;

fn args(s: &str) -> Vec<String> {
    vec![s.to_owned()]
}

fn no_wait() -> impl FnMut() {
    || panic!("must not wait")
}

#[test]
fn a_bad_invocation_never_deposits() {
    let root = tempdir().unwrap();
    assert_eq!(run(root.path(), &[], "g", 1, &mut no_wait()), USAGE_EXIT);
    assert_eq!(
        run(
            root.path(),
            &["a".into(), "b".into()],
            "g",
            1,
            &mut no_wait()
        ),
        USAGE_EXIT
    );
    assert_eq!(
        run(root.path(), &args("not json"), "g", 1, &mut no_wait()),
        USAGE_EXIT
    );
    assert_eq!(
        run(
            root.path(),
            &args(r#"{"op":"warp"}"#),
            "g",
            1,
            &mut no_wait()
        ),
        USAGE_EXIT
    );
    assert!(
        deposit::pending(root.path()).is_empty(),
        "a refused envelope never enters the inbox"
    );
}

#[test]
fn a_failed_deposit_exits_one() {
    let root = tempdir().unwrap();
    // A hand-written file squatting the name the mint hands out, then a seed
    // that is not a filename at all — both are a deposit that never happened.
    deposit::deposit(root.path(), "dup-0", &json!({"op": "balls"})).unwrap();
    for seed in ["dup", "no/seed"] {
        let exit = run(
            root.path(),
            &args(r#"{"op":"balls"}"#),
            seed,
            1,
            &mut no_wait(),
        );
        assert_eq!(exit, 1, "{seed}");
    }
}

/// bl-aa9f: a clock second and a pid are shared freely across process
/// namespaces, so two depositors can seed identically. The id is minted from
/// the world, not the seed — one seed, two ids, two replies, no crossing.
#[test]
fn one_seed_shared_across_namespaces_never_crosses_replies() {
    let root = tempdir().unwrap();
    let state = root.path().to_path_buf();
    let seed = "1786491765-2";
    // The consumer answers each deposit on its own terms: `workspaces` ok,
    // `board` refused. A crossed reply therefore flips both exit codes.
    let answer = |state: &std::path::Path| {
        for (id, path) in deposit::pending(state) {
            let sent: serde_json::Value =
                serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
            let ok = sent["op"] == json!("workspaces");
            deposit::write_reply(state, &id, &json!({"ok": ok})).unwrap();
        }
    };
    let mut other: Option<i32> = None;
    let mut wait = || {
        if other.is_none() {
            // The other namespace's depositor arrives while this one waits.
            other = Some(run(
                &state,
                &args(r#"{"op":"board"}"#),
                seed,
                2,
                &mut || answer(&state),
            ));
        }
        answer(&state);
    };
    let mine = run(
        root.path(),
        &args(r#"{"op":"workspaces"}"#),
        seed,
        3,
        &mut wait,
    );
    assert_eq!(mine, 0, "my own reply said ok");
    assert_eq!(other, Some(1), "the other caller's own reply refused");
    let ids: Vec<String> = deposit::pending(root.path())
        .into_iter()
        .map(|(id, _)| id)
        .collect();
    assert_eq!(
        ids,
        ["1786491765-2-0", "1786491765-2-1"],
        "one seed, two ids"
    );
}

#[test]
fn an_answered_gesture_prints_the_reply_and_exits_on_its_verdict() {
    let root = tempdir().unwrap();
    let state = root.path().to_path_buf();
    let mut answer_ok = || deposit::write_reply(&state, "g-ok-0", &json!({"ok": true})).unwrap();
    assert_eq!(
        run(
            root.path(),
            &args(r#"{"op":"balls"}"#),
            "g-ok",
            3,
            &mut answer_ok
        ),
        0
    );
    let state = root.path().to_path_buf();
    let mut answer_no =
        || deposit::write_reply(&state, "g-no-0", &json!({"ok": false, "error": "x"})).unwrap();
    assert_eq!(
        run(
            root.path(),
            &args(r#"{"op":"balls"}"#),
            "g-no",
            3,
            &mut answer_no
        ),
        1,
        "a refusal reply is exit 1"
    );
}

#[test]
fn an_unanswered_gesture_times_out_and_the_deposit_remains() {
    let root = tempdir().unwrap();
    let mut waited = 0u32;
    let exit = run(
        root.path(),
        &args(r#"{"op":"ops","max":4}"#),
        "g-slow",
        2,
        &mut || waited += 1,
    );
    assert_eq!(exit, TIMEOUT_EXIT);
    assert_eq!(waited, 2, "the whole poll budget was spent");
    assert_eq!(
        deposit::pending(root.path()).len(),
        1,
        "the deposit remains for the next running yog (I0)"
    );
}

/// Help is answered **in place** (§8.5): no deposit, no consumer, no wait, and
/// exit 0 — asking what a command does is an answer, not a refusal, and must
/// not depend on a yog being up.
#[test]
fn help_is_answered_without_a_consumer() {
    let root = tempdir().unwrap();
    for args in [
        vec!["--help".to_owned()],
        vec!["-h".to_owned()],
        vec!["/help".to_owned()],
        vec!["--help".to_owned(), "close".to_owned()],
        vec!["--help".to_owned(), "/close".to_owned()],
        vec!["/close --help".to_owned()],
    ] {
        assert_eq!(
            run(root.path(), &args, "g", 0, &mut no_wait()),
            0,
            "{args:?} is an answer, not a refusal"
        );
    }
    assert!(
        deposit::pending(root.path()).is_empty(),
        "help reads the interface, not the world — nothing is deposited"
    );
}

/// A help flag beside a real gesture still asks about it: `--help` rewrites the
/// invocation rather than modifying the verb (§8.5's higher-order rule).
#[test]
fn the_help_flag_wins_over_the_gesture_it_is_typed_beside() {
    let root = tempdir().unwrap();
    assert_eq!(
        run(
            root.path(),
            &["--ws".into(), "/ws".into(), "--help".into(), "scan".into()],
            "g",
            0,
            &mut no_wait()
        ),
        0
    );
    assert!(deposit::pending(root.path()).is_empty(), "nothing ran");
}
