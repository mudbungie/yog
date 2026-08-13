//! §8.6 — the capability control's half of the workspace ensure: it is authored
//! **outside** the create skip, so an existing workspace gains it on its next
//! start, and a drive that cannot write the policy aborts the start rather than
//! handing back a workspace whose drones nothing adjudicates.

use super::{World, fake_fail};
use crate::binding::workspace_path;
use crate::cli_outbound::Cli;
use crate::opslog::{Origin, SYNTHETIC_EXIT, YOG_STEP};
use crate::start::{Deps, StartError, execute_ensure_workspace};
use crate::test_support::spawn_guard;
use crate::world::{Layout, layout_under};

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
    let _g = spawn_guard();
    let w = World::new();
    let ws = workspace_path(w.yog.path(), "cobalt-gecko");
    // An existing workspace whose committed workflow carries no control block.
    crate::test_support::workspace::seed_workspace_workflow(&ws, "events: {}\n");
    let lernie = Cli::new(fake_fail(w.bin.path(), "lernie", "no config for you"));
    let err = execute_ensure_workspace(&deps(&w, &lernie), "TS", &ws, &layout(&w), Origin::Balls)
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
    let _g = spawn_guard();
    let w = World::new();
    let ws = workspace_path(w.yog.path(), "cobalt-gecko");
    let shim = crate::world::tools::control_path(&layout(&w).tools);
    let authored = crate::control::author::authored("events: {}\n", &shim);
    crate::test_support::workspace::seed_workspace_workflow(&ws, &authored);
    let lernie = Cli::new("/definitely/not/a/real/lernie");
    assert!(
        !execute_ensure_workspace(&deps(&w, &lernie), "TS", &ws, &layout(&w), Origin::Balls)
            .unwrap()
    );
    assert!(w.ops().is_empty(), "converged: nothing ran, nothing logged");
}
