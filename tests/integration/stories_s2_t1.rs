//! STORIES **S2-T1** path-rung: submitting with a work directory appends the
//! §3.3 target preamble (the directory named verbatim) and sets the driver cwd to
//! the path; no ball, so no `bl` spawns (STORIES S2.1, DESIGN §3.3/§3.4).

#![allow(clippy::unwrap_used)]

use crate::support::Recorder;
use lernie::mint::SplitMix64;
use tempfile::tempdir;
use yog::binding::workspace_path;
use yog::cli_outbound::Cli;
use yog::start::{self, Deps, Payload, StartInputs};
use yog::world::layout_under;

#[test]
fn s2_t1_path_rung_targets_the_directory() {
    let (bin, state) = (tempdir().unwrap(), tempdir().unwrap());
    let (yog, balls, home) = (tempdir().unwrap(), tempdir().unwrap(), tempdir().unwrap());
    let seed = layout_under(yog.path()).lernie;
    std::fs::create_dir_all(&seed).unwrap();
    std::fs::write(seed.join("models.yaml"), b"models: {}\n").unwrap();
    let ws = workspace_path(yog.path(), "cobalt-gecko");
    std::fs::create_dir_all(ws.join("repo.git")).unwrap();
    let dir = home.path().join("project-x");
    std::fs::create_dir_all(&dir).unwrap();

    let lernie = Recorder::new(bin.path(), "lernie");
    let bl = Recorder::new(bin.path(), "bl");
    let deps = Deps {
        bl: Cli::new(bl.path()),
        lernie: Cli::new(lernie.path()),
        state_root: state.path().to_path_buf(),
        yog_binary: std::path::PathBuf::from("/no/yog"),
        // No answer from brazen: the §9.2 birth-template gate judges nothing.
    };
    let inputs = StartInputs {
        workspace: workspace_path(yog.path(), "cobalt-gecko"),
        repo: None,
        payload: Payload::Path { dir: dir.clone() },
        home: home.path().to_path_buf(),
        yog_data_root: yog.path().to_path_buf(),
        balls_state_root: balls.path().to_path_buf(),
        conversation_names: Vec::new(),
    };

    let prepared = start::prepare(&deps, &inputs, "T0").unwrap();
    assert_eq!(
        prepared.binding.as_deref(),
        Some(dir.as_path()),
        "the path rung binds the directory typed (§3.3, bl-6654)"
    );
    assert!(
        prepared
            .goal
            .starts_with(&format!("Working directory: {}", dir.display())),
        "the prefill leads with the dir verbatim (§3.3 headline-first)"
    );
    start::execute_prompt(
        &deps.lernie,
        state.path(),
        "T0",
        &start::Fire {
            workspace: ws.clone(),
            prepared: prepared.clone(),
            goal: prepared.goal.clone(),
        },
        &[],
        &SplitMix64::from_seed(1),
    )
    .unwrap();

    let inv = lernie.wait(1);
    assert_eq!(inv[0].argv[0], "prompt");
    assert_eq!(
        inv[0].argv[3..5],
        ["--cwd".to_owned(), dir.display().to_string()],
        "the dir reaches lernie as the typed binding, not as the process cwd"
    );
    assert!(bl.invocations().is_empty(), "the path rung mutates no ball");
}
