//! §8.6 — the capability control's half of the workspace ensure: it is authored
//! **outside** the create skip, so an existing workspace gains it on its next
//! start, and a drive that cannot write the policy aborts the start rather than
//! handing back a workspace whose drones nothing adjudicates.

use super::{World, fake_fail};
use crate::binding::workspace_path;
use crate::cli_outbound::Cli;
use crate::opslog::{Origin, SYNTHETIC_EXIT, YOG_STEP};
use crate::start::{Deps, StartError, execute_ensure_workspace};
use crate::world::{Layout, layout_under};

/// lernie 0.0.8's shipped worker manifest, reduced: it composes no
/// `instructions/**`, which is exactly why §3.7 authors the glob.
const MANIFEST: &str = "roles:\n  worker:\n    pinned:\n      - goal.md\n\
    order: []\n    budget_tokens: 150000\n    overflow: drop\n";

/// The world layout anchored on this world's yog data root.
fn layout(w: &World) -> Layout {
    layout_under(w.yog.path())
}

/// Start deps whose `lernie` is the only binary these rungs reach.
fn deps(w: &World, lernie: &Cli) -> Deps {
    Deps {
        bl: Cli::new("/no/bl"),
        lernie: lernie.clone(),
        state_root: w.state.path().to_path_buf(),
        yog_binary: std::path::PathBuf::from("/no/yog"),
    }
}

/// §8.6: the ensure authors the capability control **outside** the create skip,
/// so an existing workspace gains it on its next start. A drive that cannot
/// write the policy aborts the start — a workspace whose drones nothing
/// adjudicates must not be handed back as ready.
#[test]
fn ensure_aborts_when_the_capability_control_cannot_be_authored() {
    let w = World::new();
    let ws = workspace_path(w.yog.path(), "cobalt-gecko");
    // An existing workspace whose committed workflow carries no control block.
    crate::test_support::workspace::seed_workspace_workflow(&ws, "events: {}\n");
    let lernie = Cli::new(fake_fail(w.bin.path(), "lernie", "no config for you"));
    let err = execute_ensure_workspace(
        &deps(&w, &lernie),
        "TS",
        &ws,
        "default",
        &layout(&w),
        Origin::Balls,
    )
    .unwrap_err();
    assert!(matches!(err, StartError::Control(_)), "{err}");
    let ops = w.ops();
    assert_eq!(
        ops.last().unwrap().argv,
        [YOG_STEP, "control"],
        "the abort leaves its own step-failure row (Z5)"
    );
    assert_eq!(ops.last().unwrap().exit, SYNTHETIC_EXIT);
}

/// The converged half of the same rung: a tip that already names the shim is
/// read once out of git and drives nothing, so a resume start stays spawnless.
#[test]
fn ensure_leaves_an_already_controlled_workspace_alone() {
    let w = World::new();
    let ws = workspace_path(w.yog.path(), "cobalt-gecko");
    let shim = crate::world::tools::control_path(&layout(&w).tools);
    let authored = crate::control::author::authored("events: {}\n", &shim);
    crate::test_support::workspace::seed_workspace_workflow(&ws, &authored);
    let lernie = Cli::new("/definitely/not/a/real/lernie");
    assert!(
        !execute_ensure_workspace(
            &deps(&w, &lernie),
            "TS",
            &ws,
            "default",
            &layout(&w),
            Origin::Balls
        )
        .unwrap()
    );
    assert!(w.ops().is_empty(), "converged: nothing ran, nothing logged");
}

/// §3.7 item 4 — **two files, one drive.** The `tool_control:` block and the
/// `instructions/**` glob are two control files of one yog policy, so a
/// workspace missing both converges in a *single* `lernie config` pass: one
/// checkout, one commit, one ops row, both files staged whole.
#[test]
fn ensure_converges_the_control_and_the_instruction_glob_in_one_drive() {
    let w = World::new();
    let ws = workspace_path(w.yog.path(), "cobalt-gecko");
    crate::test_support::workspace::seed_workspace_config(
        &ws,
        &[
            ("workflow.yaml", "events: {}\n"),
            ("manifest.yaml", MANIFEST),
        ],
    );
    // A `lernie` that records the staging dir the scripted editor is pointed at.
    let record = w.bin.path().join("staged");
    let lernie = Cli::new(super::write_exec(
        w.bin.path(),
        "lernie",
        &format!(
            "#!/bin/sh\nprintf '%s\\n' \"$YOG_EDIT_SRC\" > '{}'\nexit 0\n",
            record.display()
        ),
    ));
    execute_ensure_workspace(
        &deps(&w, &lernie),
        "TS",
        &ws,
        "default",
        &layout(&w),
        Origin::Balls,
    )
    .unwrap();
    assert_eq!(w.ops().len(), 1, "one drive, one row: {:?}", w.verbs());
    let staged = std::path::PathBuf::from(std::fs::read_to_string(&record).unwrap().trim());
    let workflow = std::fs::read_to_string(staged.join("workflow.yaml")).unwrap();
    assert!(workflow.contains("tool_control:"), "{workflow}");
    let manifest = std::fs::read_to_string(staged.join("manifest.yaml")).unwrap();
    assert!(manifest.contains("- instructions/**"), "{manifest}");
    assert!(manifest.contains("budget_tokens: 150000"), "{manifest}");
}

/// The other half of the same rung: a tip already carrying both stages nothing
/// and spawns nothing — the steady state of every start after the first.
#[test]
fn ensure_leaves_an_already_converged_workspace_alone() {
    let w = World::new();
    let ws = workspace_path(w.yog.path(), "cobalt-gecko");
    let shim = crate::world::tools::control_path(&layout(&w).tools);
    crate::test_support::workspace::seed_workspace_config(
        &ws,
        &[
            (
                "workflow.yaml",
                &crate::control::author::authored("events: {}\n", &shim),
            ),
            (
                "manifest.yaml",
                &crate::start::instructions::manifest::authored(MANIFEST),
            ),
        ],
    );
    let lernie = Cli::new("/definitely/not/a/real/lernie");
    assert!(
        !execute_ensure_workspace(
            &deps(&w, &lernie),
            "TS",
            &ws,
            "default",
            &layout(&w),
            Origin::Balls
        )
        .unwrap()
    );
    assert!(w.ops().is_empty(), "converged: nothing ran, nothing logged");
}
