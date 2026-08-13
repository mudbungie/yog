//! STORIES **S3-T3** resume-not-remint: a ball already claimed by a local
//! workspace name re-plans as a prompt into that workspace — no mint, no second
//! claim (STORIES S3.3, DESIGN §8.1). The workspace is on disk and the ball reads
//! Bound; `prepare` claims nothing and the prompt targets the bound worktree.

#![allow(clippy::unwrap_used)]

use crate::support::Recorder;
use tempfile::tempdir;
use yog::binding::{work_worktree_path, workspace_path};
use yog::cli_outbound::Cli;
use yog::names::SplitMix64;
use yog::projects::join::JoinState;
use yog::start::{self, BallSpec, Deps, Payload, StartInputs};
use yog::world::layout_under;

#[test]
fn s3_t3_bound_ball_resumes_without_claim_or_mint() {
    let (bin, state) = (tempdir().unwrap(), tempdir().unwrap());
    let (yog, balls, home, project) = (
        tempdir().unwrap(),
        tempdir().unwrap(),
        tempdir().unwrap(),
        tempdir().unwrap(),
    );
    let seed = layout_under(yog.path()).lernie;
    std::fs::create_dir_all(&seed).unwrap();
    std::fs::write(seed.join("models.yaml"), b"models: {}\n").unwrap();
    std::fs::create_dir_all(workspace_path(yog.path(), "cobalt-gecko").join("repo.git")).unwrap();
    // The ball was claimed earlier, so its worktree already exists on disk (the
    // detached prompt runs cwd = there).
    let worktree = work_worktree_path(balls.path(), project.path(), "bl-8", None);
    std::fs::create_dir_all(&worktree).unwrap();

    let lernie = Recorder::new(bin.path(), "lernie");
    // A bl that would exit non-zero if a claim ran — proof no claim is issued.
    let bl = Recorder::new(bin.path(), "bl").on_err("claim", "", "should not run", 1);
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
            ball: BallSpec::Existing {
                id: "bl-8".to_owned(),
                title: "Ongoing".to_owned(),
                body: "keep going".to_owned(),
                join: JoinState::Bound,
            },
        },
        home: home.path().to_path_buf(),
        yog_data_root: yog.path().to_path_buf(),
        balls_state_root: balls.path().to_path_buf(),
        conversation_names: Vec::new(),
    };

    let prepared = start::prepare(&deps, &inputs, "T0").unwrap();
    assert_eq!(
        prepared.name, "cobalt-gecko",
        "no mint — the claimant workspace"
    );
    assert_eq!(prepared.cwd, worktree, "prompt into the bound worktree");
    assert!(
        bl.invocations().is_empty(),
        "no second claim (resume, §8.1)"
    );

    start::execute_prompt(
        &deps.lernie,
        state.path(),
        "T0",
        &prepared,
        &prepared.goal,
        &[],
        &mut SplitMix64::from_seed(1),
    )
    .unwrap();
    let inv = lernie.wait(1);
    assert_eq!(inv[0].argv[0], "prompt", "just a new root in the workspace");
}
