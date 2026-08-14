//! STORIES **INV-3** convergence: re-running a start plan after a simulated
//! mid-plan kill converges — every step is idempotent-or-convergent (STORIES
//! "Invariant tests", DESIGN §8.1). Two halves: the pure planner re-derives the
//! remaining steps as join state advances (Ready → Bound drops the claim), and
//! the effectful `prepare` runs only what remains on a re-run (no double-claim,
//! no re-`prime`, no re-`new` once the substrate materialized).

#![allow(clippy::unwrap_used)]

use crate::support::Recorder;
use tempfile::tempdir;
use yog::binding::{work_worktree_path, workspace_path};
use yog::cli_outbound::Cli;
use yog::opslog;
use yog::projects::join::JoinState;
use yog::start::{self, BallSpec, Deps, Payload, StartInputs, Step};
use yog::world::layout_under;

const NAME: &str = "cobalt-gecko";

/// The pure half: `plan` re-derives a shrinking remainder as join state advances
/// (§8.1 "the remaining work is a function of disk + join state") — no spawn.
#[test]
fn inv3_plan_rederives_the_shrinking_remainder() {
    let (yog, balls, project) = (tempdir().unwrap(), tempdir().unwrap(), tempdir().unwrap());
    let plan_inputs = |join| StartInputs {
        workspace: workspace_path(yog.path(), NAME),
        repo: Some(project.path().to_path_buf()),
        payload: Payload::Ball {
            project: yog::naming::leaf(project.path()),
            ball: BallSpec::Existing {
                id: "bl-9".to_owned(),
                title: "T".to_owned(),
                body: "B".to_owned(),
                join,
            },
        },
        home: yog.path().to_path_buf(),
        yog_data_root: yog.path().to_path_buf(),
        balls_state_root: balls.path().to_path_buf(),
        conversation_names: Vec::new(),
    };

    let full = start::plan(&plan_inputs(JoinState::ReadyStartable));
    assert!(
        full.iter().any(|s| matches!(s, Step::Claim { .. })),
        "claim planned"
    );
    assert!(full.iter().any(|s| matches!(s, Step::EnsureSeeded)));
    assert!(
        full.iter()
            .any(|s| matches!(s, Step::EnsureWorkspace { .. }))
    );
    assert!(full.iter().any(|s| matches!(s, Step::Prompt { .. })));

    // After a kill leaves it bound to its workspace, the re-plan drops the claim
    // (resume, §8.1); the substrate steps stay — their skip is the executor's,
    // disk-driven, not the planner's.
    let remainder = start::plan(&plan_inputs(JoinState::Bound));
    assert!(
        !remainder.iter().any(|s| matches!(s, Step::Claim { .. })),
        "claim dropped on re-plan"
    );
    assert!(remainder.iter().any(|s| matches!(s, Step::Prompt { .. })));
}

/// The effectful half: re-running `prepare` after a mid-plan kill runs only what
/// remains — no double-claim, no re-`prime`/`new` once the substrate exists.
#[test]
fn inv3_prepare_converges_after_a_midplan_kill() {
    let (bin, state) = (tempdir().unwrap(), tempdir().unwrap());
    let (yog, balls, project) = (tempdir().unwrap(), tempdir().unwrap(), tempdir().unwrap());

    let id = "bl-42";
    let canonical = work_worktree_path(balls.path(), project.path(), id, None);
    let bl = Recorder::new(bin.path(), "bl").on("claim", &canonical.to_string_lossy(), 0);
    let lernie = Recorder::new(bin.path(), "lernie").authoring_workspaces();
    let deps = Deps {
        bl: Cli::new(bl.path()),
        lernie: Cli::new(lernie.path()),
        state_root: state.path().to_path_buf(),
        yog_binary: std::path::PathBuf::from("/no/yog"),
        // No answer from brazen: the §9.2 birth-template gate judges nothing.
    };
    let inputs = |join| StartInputs {
        workspace: workspace_path(yog.path(), NAME),
        repo: Some(project.path().to_path_buf()),
        payload: Payload::Ball {
            project: yog::naming::leaf(project.path()),
            ball: BallSpec::Existing {
                id: id.to_owned(),
                title: "Wire".to_owned(),
                body: "the body".to_owned(),
                join,
            },
        },
        home: yog.path().to_path_buf(),
        yog_data_root: yog.path().to_path_buf(),
        balls_state_root: balls.path().to_path_buf(),
        conversation_names: Vec::new(),
    };

    // Run 1 — fresh world: seed (`prime`), ensure workspace (`new`), then claim.
    let first = start::prepare(&deps, &inputs(JoinState::ReadyStartable), "T1").unwrap();
    let ws = workspace_path(yog.path(), NAME);
    assert_eq!(bl.invocations().len(), 1, "one claim");
    assert_eq!(bl.invocations()[0].argv, ["claim", id, "--as", NAME]);
    let l1: Vec<Vec<String>> = lernie.invocations().into_iter().map(|i| i.argv).collect();
    assert_eq!(
        l1,
        vec![
            vec!["prime".to_owned()],
            vec!["new".to_owned(), ws.to_string_lossy().into_owned()],
        ],
        "seed, then new — the §8.1 order"
    );

    // The kill lands after `new`: the effects the killed steps left persist
    // (§8.1). Materialize exactly that — the seed marker and workspace exist, and
    // the ball now reads bound to its workspace.
    let lernie_home = layout_under(yog.path()).lernie;
    std::fs::create_dir_all(&lernie_home).unwrap();
    std::fs::write(lernie_home.join("models.yaml"), b"models: {}\n").unwrap();
    std::fs::create_dir_all(ws.join("repo.git")).unwrap();

    // Run 2 — the re-run converges: claim dropped (bound), seed skipped (marker
    // present), `new` skipped (repo.git present). No new spawn.
    let second = start::prepare(&deps, &inputs(JoinState::Bound), "T2").unwrap();
    assert_eq!(bl.invocations().len(), 1, "no second claim");
    assert_eq!(lernie.invocations().len(), 2, "no re-prime / re-new");
    assert_eq!(
        first.binding, second.binding,
        "the same worktree either way"
    );
    assert_eq!(first.goal, second.goal, "the composed goal is stable");
    assert_eq!(
        opslog::tail(state.path(), 16).len(),
        3,
        "prime + new + claim, logged once each"
    );
}
