//! STORIES **S3-T1** ball-rung: a ready ball's ▶ Start claims it `--as <name>`
//! *after* `lernie new` (the amended §8.1 order), the composer prefills the ball
//! title, body, and worktree preamble, and the driver cwd is the work worktree
//! (STORIES S3.1, DESIGN §3.2/§3.3/§8.1).

#![allow(clippy::unwrap_used)]

use crate::support::Recorder;
use tempfile::tempdir;
use yog::binding::{work_worktree_path, workspace_path};
use yog::cli_outbound::Cli;
use yog::projects::join::JoinState;
use yog::start::{self, BallSpec, Deps, Payload, StartInputs};

#[test]
fn s3_t1_ready_ball_claims_after_new_and_binds_the_worktree() {
    let (bin, state) = (tempdir().unwrap(), tempdir().unwrap());
    let (yog, balls, home, project) = (
        tempdir().unwrap(),
        tempdir().unwrap(),
        tempdir().unwrap(),
        tempdir().unwrap(),
    );
    let canonical = work_worktree_path(balls.path(), project.path(), "bl-7", None);
    let lernie = Recorder::new(bin.path(), "lernie").authoring_workspaces();
    let bl = Recorder::new(bin.path(), "bl").on("claim", &canonical.to_string_lossy(), 0);
    let deps = Deps {
        bl: Cli::new(bl.path()),
        lernie: Cli::new(lernie.path()),
        state_root: state.path().to_path_buf(),
        yog_binary: std::path::PathBuf::from("/no/yog"),
        // No answer from brazen: the §9.2 birth-template gate judges nothing.
    };
    let inputs = StartInputs {
        workspace: workspace_path(yog.path(), "cobalt-gecko"),
        repo: Some(project.path().to_path_buf()),
        payload: Payload::Ball {
            project: yog::naming::leaf(project.path()),
            ball: BallSpec::Existing {
                id: "bl-7".to_owned(),
                title: "Wire it".to_owned(),
                body: "Do the thing.".to_owned(),
                join: JoinState::ReadyStartable,
                tags: Vec::new(),
            },
        },
        home: home.path().to_path_buf(),
        yog_data_root: yog.path().to_path_buf(),
        balls_state_root: balls.path().to_path_buf(),
        conversation_names: Vec::new(),
    };

    let prepared = start::prepare(&deps, &inputs, "T0").unwrap();
    assert_eq!(
        prepared.binding.as_deref(),
        Some(canonical.as_path()),
        "the ball rung binds the claim's work worktree typed (§3.3, bl-6654)"
    );
    assert_eq!(
        prepared.goal, "Ball bl-7: Wire it\n\nDo the thing.",
        "the goal is payload: header and body, no location prose (bl-6654)"
    );
    assert!(
        !prepared.goal.contains(&canonical.display().to_string()),
        "the worktree path is the typed binding, never goal text"
    );
    assert_eq!(
        bl.invocations()[0].argv,
        ["claim", "bl-7", "--as", "cobalt-gecko"]
    );

    // The single ops log proves the amended order: seed → new → claim.
    let ops = yog::opslog::tail(state.path(), 16);
    let verbs: Vec<&str> = ops.iter().map(|e| e.argv[1].as_str()).collect();
    assert_eq!(
        verbs,
        ["prime", "new", "claim"],
        "claim after lernie new (§8.1)"
    );
}
