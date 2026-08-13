//! STORIES **S3-T6** abort-before-claim: on the ball rung — the rung that *has*
//! a `bl` mutation to abort — a failed substrate step (`lernie prime`) precedes
//! every `bl` mutation, so **no `bl create`/`bl claim` is recorded** (STORIES
//! S3.6, DESIGN §8.1 — the load-bearing order that closes the orphaned-claim
//! wound). S0-T3's no-`bl` assertion is vacuous on the bare rung; this proves it.

#![allow(clippy::unwrap_used)]

use crate::support::Recorder;
use tempfile::tempdir;
use yog::binding::workspace_path;
use yog::cli_outbound::Cli;
use yog::start::{self, BallSpec, Deps, Payload, StartError, StartInputs};

#[test]
fn s3_t6_a_failed_substrate_aborts_before_any_bl_mutation() {
    let (bin, state) = (tempdir().unwrap(), tempdir().unwrap());
    let (yog, balls, home, project) = (
        tempdir().unwrap(),
        tempdir().unwrap(),
        tempdir().unwrap(),
        tempdir().unwrap(),
    );
    // `prime` fails; `bl create`/`bl claim` would both succeed if ever reached.
    let lernie = Recorder::new(bin.path(), "lernie").on_err("prime", "", "no seed", 2);
    let bl = Recorder::new(bin.path(), "bl")
        .on("create", "bl-x\n", 0)
        .on("claim", "/wt/bl-x", 0);
    let deps = Deps {
        bl: Cli::new(bl.path()),
        lernie: Cli::new(lernie.path()),
        state_root: state.path().to_path_buf(),
        yog_binary: std::path::PathBuf::from("/no/yog"),
        // No answer from brazen: the §9.2 birth-template gate judges nothing.
    };
    let inputs = StartInputs {
        workspace: workspace_path(yog.path(), "cobalt-gecko"),
        payload: Payload::Ball {
            project: project.path().to_path_buf(),
            ball: BallSpec::New {
                title: "Fresh".to_owned(),
                body: "body".to_owned(),
            },
        },
        home: home.path().to_path_buf(),
        yog_data_root: yog.path().to_path_buf(),
        balls_state_root: balls.path().to_path_buf(),
        conversation_names: Vec::new(),
    };

    let err = start::prepare(&deps, &inputs, "T0").unwrap_err();
    assert!(matches!(err, StartError::Seed(_)));
    assert!(
        bl.invocations().is_empty(),
        "no bl create / bl claim after a failed substrate (§8.1)"
    );
}
