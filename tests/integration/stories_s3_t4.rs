//! STORIES **S3-T4** close-gate-verbatim: a `bl close` that fails its pre-commit
//! gate rides its stderr back verbatim — into the `ops.jsonl` row and the
//! surface-failure view-model the originating pane paints (STORIES S3.4, DESIGN
//! §8.2, §4.2, §7.3). The claim + worktree stay up (bl's own semantics).

#![allow(clippy::unwrap_used)]

use crate::support::Recorder;
use tempfile::tempdir;
use yog::actions::verbs;
use yog::cli_outbound::Cli;
use yog::opslog::{self, OpRow, SurfaceFailure};

#[test]
fn s3_t4_close_gate_failure_is_verbatim_in_ops_and_view_model() {
    let (bin, state, project) = (tempdir().unwrap(), tempdir().unwrap(), tempdir().unwrap());
    let gate = "pre-commit: coverage 92% < 100%\naborting close";
    let bl = Recorder::new(bin.path(), "bl").on_err("close", "", gate, 1);
    let cli = Cli::new(bl.path());
    // A permissive gate — this story exercises the gate-failure-verbatim axis of
    // `bl close`, not the capability refusal.

    let out = verbs::close(
        &cli,
        state.path(),
        "T0",
        project.path(),
        "bl-4db6",
        "cobalt-gecko",
    )
    .unwrap();
    assert_eq!(out.exit, 1, "the gate failure's non-zero exit");
    assert_eq!(out.stderr, gate, "stderr verbatim from bl");

    // The durable ops row carries the whole failure (§4.2).
    let ops = opslog::tail(state.path(), 16);
    assert_eq!(ops.len(), 1);
    assert_eq!(ops[0].stderr, gate);
    assert_eq!(
        &ops[0].argv[1..],
        &["close", "bl-4db6", "--as", "cobalt-gecko"]
    );

    // The surface view-model (Z5) renders argv + the stderr tail in ichor red.
    let row = OpRow::from(&ops[0]);
    assert!(row.failed(), "a non-zero close is a rendered failure");
    let surface = SurfaceFailure::from(&row);
    assert!(
        surface.argv.ends_with("close bl-4db6 --as cobalt-gecko"),
        "the attempted verb is shown: {}",
        surface.argv
    );
    assert!(
        surface.stderr_tail.contains("aborting close"),
        "the gate cause is shown"
    );
}
