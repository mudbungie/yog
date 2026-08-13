//! The `bl`-facing executors (§4.2, Z5): `bl create`, `bl claim` and the
//! claim cross-check — each leaving a durable ops row, and a
//! `["yog-step",…]` line for the non-spawn failures. The workspace-and-name
//! executors are [`super::ensure`]'s concern.

use super::{World, fake_bl, fake_fail};
use crate::binding::work_worktree_path;
use crate::cli_outbound::Cli;
use crate::opslog::{SYNTHETIC_EXIT, YOG_STEP};
use crate::start::{StartError, cross_check_claim, execute_claim, execute_create};
use crate::test_support::spawn_guard;

#[test]
fn create_stamps_the_workspace_name_and_captures_the_id() {
    let _g = spawn_guard();
    let w = World::new();
    let bl = Cli::new(fake_bl(w.bin.path(), "bl-new9", w.balls.path()));
    let id = execute_create(
        &bl,
        w.state.path(),
        "TS",
        w.project.path(),
        "Wire",
        "the body",
        "cobalt-gecko",
    )
    .unwrap();
    assert_eq!(id, "bl-new9");
    assert_eq!(
        &w.ops()[0].argv[1..],
        &[
            "create",
            "Wire",
            "--as",
            "cobalt-gecko",
            "--body",
            "the body"
        ]
    );
}

#[test]
fn create_elides_an_empty_body() {
    let _g = spawn_guard();
    let w = World::new();
    let bl = Cli::new(fake_bl(w.bin.path(), "bl-1", w.balls.path()));
    execute_create(&bl, w.state.path(), "TS", w.project.path(), "T", "", "n").unwrap();
    assert_eq!(&w.ops()[0].argv[1..], &["create", "T", "--as", "n"]);
}

#[test]
fn create_errors_on_a_nonzero_exit() {
    let _g = spawn_guard();
    let w = World::new();
    let bl = Cli::new(fake_fail(w.bin.path(), "bl", "boom"));
    let err =
        execute_create(&bl, w.state.path(), "TS", w.project.path(), "T", "", "n").unwrap_err();
    assert!(matches!(err, StartError::VerbFailed { verb: "create", .. }));
}

#[test]
fn claim_stamps_the_name_and_cross_checks_the_canonical_worktree() {
    let _g = spawn_guard();
    let w = World::new();
    let canonical = work_worktree_path(w.balls.path(), w.project.path(), "bl-7", None);
    let bl = Cli::new(fake_bl(w.bin.path(), "x", &canonical));
    let r = execute_claim(
        &bl,
        w.state.path(),
        "TS",
        w.project.path(),
        "bl-7",
        "cobalt-gecko",
        w.balls.path(),
    )
    .unwrap();
    assert_eq!(r.worktree, canonical);
    assert!(!r.suffixed);
    assert_eq!(
        &w.ops()[0].argv[1..],
        &["claim", "bl-7", "--as", "cobalt-gecko"]
    );
}

#[test]
fn claim_errors_on_a_nonzero_exit() {
    let _g = spawn_guard();
    let w = World::new();
    let bl = Cli::new(fake_fail(w.bin.path(), "bl", "blocked"));
    let err = execute_claim(
        &bl,
        w.state.path(),
        "TS",
        w.project.path(),
        "bl-7",
        "n",
        w.balls.path(),
    )
    .unwrap_err();
    let StartError::VerbFailed { verb, outcome } = err else {
        panic!("expected VerbFailed, got {err:?}");
    };
    assert_eq!(verb, "claim");
    assert_eq!(outcome.stderr, "blocked\n");
}

#[test]
fn cross_check_accepts_the_claimant_suffix_variant() {
    let w = World::new();
    let suffixed = work_worktree_path(w.balls.path(), w.project.path(), "bl-9", Some("n"));
    let r = cross_check_claim(
        &suffixed.display().to_string(),
        w.balls.path(),
        w.project.path(),
        "bl-9",
        "n",
        w.state.path(),
        "TS",
    )
    .unwrap();
    assert!(r.suffixed);
    assert_eq!(r.worktree, suffixed);
    assert!(w.ops().is_empty(), "a matched claim logs no step-failure");
}

#[test]
fn cross_check_drift_is_surfaced_loudly_and_logged() {
    let w = World::new();
    let err = cross_check_claim(
        "/somewhere/unexpected",
        w.balls.path(),
        w.project.path(),
        "bl-9",
        "n",
        w.state.path(),
        "TS",
    )
    .unwrap_err();
    assert!(matches!(err, StartError::Drift { .. }));
    // The Drift is a rendered fact: a `["yog-step","cross-check"]` row (Z5).
    let e = &w.ops()[0];
    assert_eq!(e.argv, [YOG_STEP, "cross-check"]);
    assert_eq!(e.exit, SYNTHETIC_EXIT);
    assert!(e.stderr.contains("drift"));
}
