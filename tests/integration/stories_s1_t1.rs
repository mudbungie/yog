//! STORIES **S1-T1** prompt-into-existing: with a focused workspace in a seeded
//! world, Enter is just a new root — `lernie prompt` only, no mint, no `new`, no
//! `prime` (re-opening is the same gesture as opening, §3.4). Driven through the
//! start dispatch with the substrate already on disk (STORIES S1.2, DESIGN §3.4,
//! §8.1).

#![allow(clippy::unwrap_used)]

use crate::support::Recorder;
use lernie::mint::SplitMix64;
use tempfile::tempdir;
use yog::binding::workspace_path;
use yog::cli_outbound::Cli;
use yog::start::{self, Deps, Payload, StartInputs};
use yog::world::layout_under;

#[test]
fn s1_t1_focused_workspace_enter_is_prompt_only() {
    let (bin, state) = (tempdir().unwrap(), tempdir().unwrap());
    let (yog, balls, home) = (tempdir().unwrap(), tempdir().unwrap(), tempdir().unwrap());
    // A seeded world with the focused workspace already on disk.
    let seed = layout_under(yog.path()).lernie;
    std::fs::create_dir_all(&seed).unwrap();
    std::fs::write(seed.join("models.yaml"), b"models: {}\n").unwrap();
    let ws = workspace_path(yog.path(), "cobalt-gecko");
    std::fs::create_dir_all(ws.join("repo.git")).unwrap();

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
        payload: Payload::Bare,
        home: home.path().to_path_buf(),
        yog_data_root: yog.path().to_path_buf(),
        balls_state_root: balls.path().to_path_buf(),
        conversation_names: Vec::new(),
    };

    let prepared = start::prepare(&deps, &inputs, "T0").unwrap();
    assert_eq!(
        prepared.workspace, "cobalt-gecko",
        "no mint — the focused workspace"
    );
    assert!(
        lernie.invocations().is_empty(),
        "seeded + present → prepare spawns nothing"
    );
    start::execute_prompt(
        &deps.lernie,
        state.path(),
        "T0",
        &start::Fire {
            workspace: ws.clone(),
            prepared: prepared.clone(),
            goal: "next step".to_owned(),
        },
        &[],
        &SplitMix64::from_seed(1),
    )
    .unwrap();

    let inv = lernie.wait(1);
    assert_eq!(inv.len(), 1, "prompt only — no prime, no new (S1-T1)");
    assert_eq!(inv[0].argv[0], "prompt");
    assert!(bl.invocations().is_empty(), "no ball, no bl");
}
