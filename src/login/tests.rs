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
    ["bz", "--login", "--provider", "openai", "--browser"]
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

    // bz writes its whole sign-in flow to STDERR — the authorize URL included —
    // and it must paint verbatim, live (bl-b4e5 defect 3: a stdout-only view
    // left the pane blank). A stdout line interleaves in arrival order.
    tx.send(Chunk::Stderr(
        b"To authorize, open https://x/auth\n".to_vec(),
    ))
    .unwrap();
    tx.send(Chunk::Stdout(b"noise\n".to_vec())).unwrap();
    assert!(run.poll());
    assert_eq!(
        texts(&run.view()),
        vec!["To authorize, open https://x/auth", "noise"]
    );

    // Clean exit settles the run (the Done arm): outcome 0, no fallback.
    tx.send(Chunk::Exited(ExitInfo::Code(0))).unwrap();
    assert!(!run.poll());
    let view = run.view();
    assert_eq!(view.outcome, Some(0));
    assert_eq!(view.fallback, None);
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
/// what to fix rather than offering one that would refuse.
#[test]
fn the_run_by_hand_spelling_binds_the_workspace_or_says_there_is_none() {
    assert_eq!(
        by_hand(Some(Path::new("/spheres/corp")), "anthropic"),
        "yog exec --ws /spheres/corp bz --login --provider anthropic --browser"
    );
    let none = by_hand(None, "anthropic");
    assert!(none.contains("no workspace"), "{none}");
    assert!(
        !none.starts_with("yog exec"),
        "not offered as runnable: {none}"
    );
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
    // it. `--browser` included, so the offered command is one that succeeds.
    assert_eq!(
        view.fallback.as_deref(),
        Some("yog exec --ws /ws bz --login --provider openai --browser")
    );

    let ops = opslog::tail(dir.path(), 8);
    assert_eq!(ops[0].exit, 78);
    // The logged stderr is derived from the very lines the pane painted.
    assert_eq!(ops[0].stderr, texts(&view).join("\n"));
}

#[test]
fn start_logs_a_synthetic_row_when_bz_cannot_spawn() {
    // Hold the binary-wide spawn lock across the fork (the `test_support`
    // discipline): even a failed exec forks, and its copied write-fd would race a
    // peer's recorder-script write into ETXTBSY without this.
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
            "--browser",
        ]
    );
    assert!(!ops[0].stderr.is_empty());
}
