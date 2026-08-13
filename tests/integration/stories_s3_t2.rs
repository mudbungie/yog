//! STORIES **S3-T2** new-ball-converges: a new ball from the composer is `bl
//! create` then the existing-ball path — the new→existing transition is the
//! convergence, yielding exactly one claim (STORIES S3.2, DESIGN §8.1). A seeded
//! world with the workspace present isolates the `bl` sequence.

#![allow(clippy::unwrap_used)]

use crate::support::Recorder;
use tempfile::tempdir;
use yog::binding::{work_worktree_path, workspace_path};
use yog::cli_outbound::Cli;
use yog::start::{self, BallSpec, Deps, Payload, StartInputs};
use yog::world::layout_under;

#[test]
fn s3_t2_new_ball_creates_then_converges_to_one_claim() {
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
    let canonical = work_worktree_path(balls.path(), project.path(), "bl-mint", None);

    let lernie = Recorder::new(bin.path(), "lernie");
    let bl = Recorder::new(bin.path(), "bl")
        .on("create", "bl-mint\n", 0)
        .on("claim", &canonical.to_string_lossy(), 0);
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
                body: "New body".to_owned(),
            },
        },
        home: home.path().to_path_buf(),
        yog_data_root: yog.path().to_path_buf(),
        balls_state_root: balls.path().to_path_buf(),
        conversation_names: Vec::new(),
    };

    let prepared = start::prepare(&deps, &inputs, "T0").unwrap();
    assert!(
        prepared.goal.starts_with("Ball bl-mint: Fresh"),
        "re-planned as the minted ball"
    );

    let invs = bl.invocations();
    let bl_verbs: Vec<&str> = invs.iter().map(|i| i.argv[0].as_str()).collect();
    assert_eq!(
        bl_verbs,
        ["create", "claim"],
        "create then one claim (the convergence)"
    );
    assert_eq!(invs[1].argv, ["claim", "bl-mint", "--as", "cobalt-gecko"]);
}
