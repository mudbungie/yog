//! The streamed [`LoginRun`] view-model and its one outcome row, driven
//! deterministically through [`Streamed::from_rx`] so every `poll` arm is
//! covered without racing a process (the real spawn path is the S0-T5 story).
//! The spawn-failure path uses a genuinely absent binary.
//!
//! The §8.3 auth heuristic — the other half this file's doc used to name — is
//! [`auth`], split off at §12's cap on that same seam.

mod auth;

use super::by_hand;
use std::path::Path;
use std::sync::mpsc;

use tempfile::tempdir;

use super::{LoginRun, start};
use crate::cli_outbound::{Chunk, Cli, ExitInfo, Streamed};
use crate::opslog::{self, SYNTHETIC_EXIT};

fn argv() -> Vec<String> {
    ["bz", "--login", "--provider", "openai"]
        .into_iter()
        .map(String::from)
        .collect()
}

/// The rendered text of a view's lines, in arrival order (the pane paints
/// exactly this, whichever stream each line came from).
fn texts(view: &super::LoginView) -> Vec<String> {
    view.lines.iter().map(|l| l.text.clone()).collect()
}

#[test]
fn login_run_streams_lines_live_then_logs_a_clean_outcome_row() {
    let dir = tempdir().unwrap();
    let (tx, rx) = mpsc::channel();
    let mut run = LoginRun::from_streamed(Streamed::from_rx(rx), argv(), dir.path());

    // Before any output: still running, no lines yet (the Pending arm).
    assert!(run.poll());
    assert!(run.view().lines.is_empty());

    // A device flow writes its whole sign-in to STDERR — the verification URL
    // and the user code — and it must paint verbatim, live (bl-b4e5 defect 3:
    // a stdout-only view left the pane blank).
    tx.send(Chunk::Stderr(
        b"To authorize, open https://x/device\n".to_vec(),
    ))
    .unwrap();
    tx.send(Chunk::Stderr(b"and enter CODE-1234\n".to_vec()))
        .unwrap();
    assert!(run.poll());
    assert_eq!(
        texts(&run.view()),
        vec!["To authorize, open https://x/device", "and enter CODE-1234"]
    );

    // Clean exit settles the run (the Done arm): outcome 0, no fallback — and
    // nothing on stdout, so this is a sign-in and not a run waiting for a code.
    tx.send(Chunk::Exited(ExitInfo::Code(0))).unwrap();
    assert!(!run.poll());
    let view = run.view();
    assert_eq!(view.outcome, Some(0));
    assert_eq!(view.fallback, None);
    assert_eq!(view.authorize, None);
    // Idempotent after settle — the guard short-circuits, no second row.
    assert!(!run.poll());

    // Exactly one outcome row, its argv the login command, stdout never re-logged.
    let ops = opslog::tail(dir.path(), 8);
    assert_eq!(ops.len(), 1);
    assert_eq!(ops[0].argv, argv());
    assert_eq!(ops[0].exit, 0);
    assert_eq!(ops[0].stdout, "");
}

/// bl-b589 — the run-by-hand spelling is the one the hatch actually accepts,
/// and it names the workspace, because a sign-in belongs to a sphere: the
/// unbound `bz --login` the pane used to print exits 64 in any ordinary shell.
/// Outside a workspace there is no lawful command at all, so the fallback says
/// what to fix rather than offering one that would refuse. No flow word in it
/// (bl-0127): the row's own brazen picks the flow, by hand as under yog.
#[test]
fn the_run_by_hand_spelling_binds_the_workspace_or_says_there_is_none() {
    assert_eq!(
        by_hand(Some(Path::new("/spheres/corp")), "anthropic"),
        "yog exec --ws /spheres/corp bz --login --provider anthropic"
    );
    let none = by_hand(None, "anthropic");
    assert!(none.contains("no workspace"), "{none}");
    assert!(none.ends_with("bz --login --provider anthropic"), "{none}");
    assert!(
        !none.starts_with("yog exec"),
        "not offered as runnable: {none}"
    );
}

/// **A zero exit that wrote to stdout is a run waiting for a code** (REMOTE
/// §8.3, bl-0127): brazen's pasted-code sign-in prints its authorize URL on
/// stdout and exits clean with nothing stored. The run states no outcome,
/// carries the URL, and stops polling; the code a human copied is fired into
/// the SAME run as a second command line, whose lines append and whose exit
/// is the outcome. Each command line leaves its own `ops.jsonl` row, and the
/// second's stderr is its own — never the first's again.
#[test]
fn a_run_that_printed_a_url_waits_and_a_redeemed_code_finishes_it() {
    let dir = tempdir().unwrap();
    // The fake brazen: a URL on stdout for the bare line, a verdict for a code.
    let bz = dir.path().join("bz");
    crate::test_support::write_exec(
        &bz,
        "#!/bin/sh\ncase \"$*\" in\n\
         *--code=good*) echo 'logged in to `openai`' 1>&2 ;;\n\
         *--code=*) echo 'login failed: wrong code' 1>&2; exit 77 ;;\n\
         *) echo 'https://x/authorize?state=s'; echo 'open the URL above' 1>&2 ;;\nesac\n",
    );
    let bz = Cli::new(bz);
    let settle = |run: &mut LoginRun| {
        for _ in 0..600 {
            if !run.poll() {
                return run.view();
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        panic!("the command line never ended");
    };
    let mut run = start(&bz, "openai", dir.path(), "TS", Some(Path::new("/ws"))).expect("spawned");
    let waiting = settle(&mut run);
    assert_eq!(waiting.outcome, None, "nobody is signed in yet");
    assert_eq!(waiting.fallback, None);
    assert_eq!(
        waiting.authorize.as_deref(),
        Some("https://x/authorize?state=s")
    );
    assert!(!run.poll(), "a waiting run has no child to read");

    // A refused code settles the run non-zero, and what is offered by hand is
    // the command that starts over — never the spent code again.
    run.redeem(&bz, "bad", "T2").expect("spawned");
    assert_eq!(run.view().authorize, None, "redeeming is no longer waiting");
    let refused = settle(&mut run);
    assert_eq!(refused.outcome, Some(77));
    assert_eq!(
        refused.fallback.as_deref(),
        Some("yog exec --ws /ws bz --login --provider openai")
    );
    assert_eq!(
        texts(&refused).last().map(String::as_str),
        Some("login failed: wrong code"),
        "the second command's lines append to the first's"
    );
    assert_eq!(texts(&refused).len(), 3);

    let ops = opslog::tail(dir.path(), 8);
    assert_eq!(ops.len(), 2, "one row per command line");
    assert_eq!(ops[0].exit, 0);
    assert_eq!(ops[0].stderr, "open the URL above");
    assert_eq!(ops[1].ts, "T2");
    assert_eq!(ops[1].argv.last().map(String::as_str), Some("--code=bad"));
    assert_eq!(ops[1].stderr, "login failed: wrong code");

    // And a code the row's brazen accepts is the sign-in.
    let mut run = start(&bz, "openai", dir.path(), "TS", Some(Path::new("/ws"))).expect("spawned");
    settle(&mut run);
    run.redeem(&bz, "good", "T3").expect("spawned");
    let done = settle(&mut run);
    assert_eq!(
        (done.outcome, done.fallback, done.authorize),
        (Some(0), None, None)
    );

    // A finish that cannot spawn leaves the run waiting exactly as it stood.
    let mut run = start(&bz, "openai", dir.path(), "TS", Some(Path::new("/ws"))).expect("spawned");
    let before = settle(&mut run);
    assert!(
        run.redeem(&Cli::new("/definitely/not/a/real/bz-xyz"), "c", "T4")
            .is_err()
    );
    assert_eq!(run.view(), before);
}

#[test]
fn login_run_nonzero_exit_carries_the_fallback_command_and_stderr() {
    let dir = tempdir().unwrap();
    let (tx, rx) = mpsc::channel();
    let mut run = LoginRun::from_streamed(Streamed::from_rx(rx), argv(), dir.path());
    // The exact shape of the live 78: bz names the reason and the remedy on
    // stderr and yog must show it, not swallow it (bl-b4e5 defect 3).
    tx.send(Chunk::Stderr(
        b"provider `local` has no `oauth` config; add an `oauth` block to its row\n".to_vec(),
    ))
    .unwrap();
    tx.send(Chunk::Exited(ExitInfo::Code(78))).unwrap();
    assert!(!run.poll());

    let view = run.view();
    assert_eq!(view.outcome, Some(78));
    assert_eq!(
        texts(&view),
        vec!["provider `local` has no `oauth` config; add an `oauth` block to its row"],
        "bz's terminal reason/remedy line reaches the pane"
    );
    // §8.3 fallback: the command to run by hand — the **workspace-bound**
    // spelling (bl-b589), because the bare `bz --login` yog itself spawns
    // carries the wall in the child's env and an ordinary shell cannot inherit
    // it.
    assert_eq!(
        view.fallback.as_deref(),
        Some("yog exec --ws /ws bz --login --provider openai")
    );

    let ops = opslog::tail(dir.path(), 8);
    assert_eq!(ops[0].exit, 78);
    // The logged stderr is derived from the very lines the pane painted.
    assert_eq!(ops[0].stderr, texts(&view).join("\n"));
}

#[test]
fn start_logs_a_synthetic_row_when_bz_cannot_spawn() {
    let dir = tempdir().unwrap();
    let bz = Cli::new("/definitely/not/a/real/bz-xyz");
    // `.err()` sidesteps `unwrap_err`'s `T: Debug` bound (LoginRun holds a live
    // Stream and is deliberately not Debug).
    let err = start(&bz, "openai", dir.path(), "TS", Some(Path::new("/ws")))
        .err()
        .unwrap();
    assert_eq!(err.kind(), std::io::ErrorKind::Other);

    // The failed spawn still leaves a rendered fact: one synthetic ops row.
    let ops = opslog::tail(dir.path(), 8);
    assert_eq!(ops.len(), 1);
    assert_eq!(ops[0].exit, SYNTHETIC_EXIT);
    assert_eq!(
        ops[0].argv,
        [
            "/definitely/not/a/real/bz-xyz",
            "--login",
            "--provider",
            "openai",
        ]
    );
    assert!(!ops[0].stderr.is_empty());
}
