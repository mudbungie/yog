//! STORIES **S0-T3** seed-failure-surfaces (abort half): a `lernie prime` that
//! exits non-zero aborts the start **before** anything else spawns — no `lernie
//! new`, no prompt — and the failure is a durable, rendered fact (its stderr in
//! the ops row). The view-model + draft-survival halves are Z5-covered; this
//! proves the abort + the ops trail (STORIES S0.4, DESIGN §8.1, §4.2).

#![allow(clippy::unwrap_used)]

use crate::support::Recorder;
use tempfile::tempdir;
use yog::binding::workspace_path;
use yog::cli_outbound::Cli;
use yog::names::DEFAULT_NAME;
use yog::start::{self, Deps, Payload, StartError, StartInputs};

#[test]
fn s0_t3_a_failed_prime_aborts_before_any_further_spawn() {
    let (bin, state) = (tempdir().unwrap(), tempdir().unwrap());
    let (yog, balls, home) = (tempdir().unwrap(), tempdir().unwrap(), tempdir().unwrap());
    // `prime` fails on stderr; `new` would exit 0 if it were ever reached.
    let lernie = Recorder::new(bin.path(), "lernie").on_err("prime", "", "no models.yaml", 2);
    let bl = Recorder::new(bin.path(), "bl");
    let deps = Deps {
        bl: Cli::new(bl.path()),
        lernie: Cli::new(lernie.path()),
        state_root: state.path().to_path_buf(),
        yog_binary: std::path::PathBuf::from("/no/yog"),
        // No answer from brazen: the §9.2 birth-template gate judges nothing.
    };
    let inputs = StartInputs {
        workspace: workspace_path(yog.path(), DEFAULT_NAME),
        payload: Payload::Bare,
        home: home.path().to_path_buf(),
        yog_data_root: yog.path().to_path_buf(),
        balls_state_root: balls.path().to_path_buf(),
        conversation_names: Vec::new(),
    };

    let err = start::prepare(&deps, &inputs, "T0").unwrap_err();
    assert!(
        matches!(err, StartError::Seed(_)),
        "the seed failure rides back"
    );

    // The abort precedes every other spawn: only `prime` ran — no `new`, no bl.
    let lernie_verbs: Vec<String> = lernie
        .invocations()
        .into_iter()
        .map(|i| i.argv[0].clone())
        .collect();
    assert_eq!(
        lernie_verbs,
        ["prime"],
        "no `lernie new` after a failed seed"
    );
    assert!(
        bl.invocations().is_empty(),
        "no bl spawn (nothing to abort on the bare rung)"
    );

    // The failure is a rendered fact: the ops row holds the exit + stderr (§4.2),
    // from which Z5's SurfaceFailure view-model paints argv + the stderr tail.
    let ops = yog::opslog::tail(state.path(), 16);
    assert_eq!(ops.len(), 1, "just the failed prime");
    assert_eq!(ops[0].exit, 2);
    assert_eq!(ops[0].stderr, "no models.yaml");
    assert_eq!(&ops[0].argv[1..], &["prime"]);
}
