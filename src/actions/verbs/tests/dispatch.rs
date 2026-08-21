//! Tests for the dispatch/logging core ([`super::super::dispatch`]): the
//! attempted-outcome ops line, the synthetic spawn-failure line, the yog-step
//! encoding, and the failure view-models both surfaces read (§4.2, §7.3, INV-2).

use super::{OK_BODY, World};
use crate::actions::verbs::{Outcome, collect, log_step_failure, run_logged};
use crate::cli_outbound::Cli;
use crate::opslog::{self, Origin};
use std::path::Path;
use tempfile::tempdir;

#[test]
fn collect_propagates_a_spawn_error() {
    // `collect` (the no-marks `bl conf` read seam) surfaces a spawn failure as
    // Err — nothing ran, so there is nothing to drain.
    let cli = Cli::new("/definitely/not/a/real/bin");
    assert!(collect(cli.run(&["x"])).is_err());
}

#[test]
fn outcome_ok_tracks_exit_zero() {
    let mk = |exit| Outcome {
        exit,
        stdout: String::new(),
        stderr: String::new(),
    };
    assert!(mk(0).ok());
    assert!(!mk(1).ok());
}

#[test]
fn run_logged_captures_streams_and_appends_the_entry() {
    let w = World::new("lernie", OK_BODY);
    let out = run_logged(
        &w.cli,
        w.state.path(),
        "TS",
        &w.cwd,
        &["scan", "/ws"],
        Origin::Conversation,
    )
    .unwrap();
    assert!(out.ok());
    assert_eq!(out.stdout, "OUT\n");
    assert_eq!(out.stderr, "ERR\n");
    let e = w.logged();
    assert_eq!(e.ts, "TS");
    assert_eq!(e.cwd, w.cwd.display().to_string());
    assert_eq!(e.exit, 0);
    assert_eq!(e.stdout, "OUT\n");
    assert_eq!(e.stderr, "ERR\n");
    // argv[0] is the resolved binary; the verb args follow.
    assert_eq!(e.argv[0], w.cli.binary().display().to_string());
    assert_eq!(&e.argv[1..], &["scan".to_string(), "/ws".to_string()]);
}

#[test]
fn run_logged_logs_a_nonzero_gate_failure_verbatim() {
    // §8.2: a bl-close gate failure is a completed action — logged, not dropped.
    let w = World::new(
        "bl",
        "#!/bin/sh\nprintf 'gate: hook failed\\n' 1>&2\nexit 1\n",
    );
    let out = run_logged(
        &w.cli,
        w.state.path(),
        "TS",
        &w.cwd,
        &["close", "bl-1"],
        Origin::Balls,
    )
    .unwrap();
    assert!(!out.ok());
    assert_eq!(out.exit, 1);
    assert_eq!(out.stderr, "gate: hook failed\n");
    assert_eq!(w.logged().exit, 1);
}

#[test]
fn run_logged_errs_but_appends_a_synthetic_spawn_failure_line() {
    // INV-2 / §4.2 amended: a spawn that never launched is a *rendered fact*, not
    // a dropped error — a synthetic line with the intended argv, the error in
    // stderr, exit SYNTHETIC; the call still returns Err.
    let state = tempdir().unwrap();
    let cwd = tempdir().unwrap();
    let cli = Cli::new("/definitely/not/a/real/bl-xyz");
    let r = run_logged(
        &cli,
        state.path(),
        "TS",
        cwd.path(),
        &["close", "bl-1"],
        Origin::Balls,
    );
    assert!(r.is_err(), "a spawn failure still returns Err");
    let e = opslog::tail(state.path(), 8).pop().unwrap();
    assert_eq!(e.exit, opslog::SYNTHETIC_EXIT);
    assert_eq!(e.argv[0], "/definitely/not/a/real/bl-xyz");
    assert_eq!(&e.argv[1..], &["close".to_string(), "bl-1".to_string()]);
    assert!(!e.stderr.is_empty(), "the spawn error rides in stderr");
    assert!(e.stdout.is_empty());
    assert!(opslog::OpRow::from(&e).failed());
}

#[test]
fn s0_t3_failed_step_entry_carries_stderr_and_view_model_renders_argv_and_tail() {
    // S0-T3 (ops-row + view-model halves): a failed step (fake exits 2 with
    // stderr) leaves a rendered fact — the ops entry carries the stderr, and the
    // surface failure view-model renders argv + the stderr tail (§7.3).
    let w = World::new(
        "lernie",
        "#!/bin/sh\nprintf 'prime failed\\nmodels.yaml missing\\n' 1>&2\nexit 2\n",
    );
    let out = run_logged(
        &w.cli,
        w.state.path(),
        "TS",
        &w.cwd,
        &["prime"],
        Origin::Conversation,
    )
    .unwrap();
    assert_eq!(out.exit, 2);
    let e = w.logged();
    assert_eq!(e.exit, 2);
    assert_eq!(e.stderr, "prime failed\nmodels.yaml missing\n");
    // Both view-models project the one durable entry — surface RAM never diverges.
    let row = opslog::OpRow::from(&e);
    assert!(row.failed());
    let failure = opslog::SurfaceFailure::from(&row);
    assert_eq!(failure.argv, format!("{} prime", w.cli.binary().display()));
    assert_eq!(failure.stderr_tail, "prime failed\nmodels.yaml missing");
}

#[test]
fn log_step_failure_writes_a_yog_step_line() {
    // The non-spawn error class (mint/mkdir/cross-check) that names no binary
    // still leaves a rendered fact: a ["yog-step", <name>] line (§4.2) whose
    // failure text rides in stderr — the covered encoding Z3 logs its aborts through.
    let state = tempdir().unwrap();
    log_step_failure(
        state.path(),
        "TS",
        Path::new("/proj"),
        "mint",
        "name pool exhausted",
        Origin::Conversation,
    )
    .unwrap();
    let e = opslog::tail(state.path(), 8).pop().unwrap();
    assert_eq!(
        e.argv,
        vec![opslog::YOG_STEP.to_string(), "mint".to_string()]
    );
    assert_eq!(e.exit, opslog::SYNTHETIC_EXIT);
    assert_eq!(e.cwd, "/proj");
    assert_eq!(e.stderr, "name pool exhausted");
    assert!(opslog::OpRow::from(&e).failed());
}
