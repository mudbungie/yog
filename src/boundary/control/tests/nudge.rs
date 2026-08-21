//! The §8.2 **nudge** (bl-9bef) over this family's own launch: `lernie advance`
//! fired detached with no park in front of it — the operator's "run it again
//! from where it stands".
//!
//! Its own file at §12's cap, on the seam the module itself draws: [`super`]
//! answers *one parked invocation* and drives on, while this is the drive with
//! nothing to answer. One body, two callers, and each caller's own beats.

use super::*;

/// The §8.2 nudge (bl-9bef) is the same launch with no park in front of it:
/// one detached `lernie advance` row and nothing else — no answer row, because
/// there is no invocation to answer. Driven through the chokepoint every seat
/// enters, which is the whole claim the boundary makes.
#[test]
fn a_nudge_launches_the_driver_and_writes_only_that_row() {
    let world = World::new();
    world.repo();
    let mut ui = crate::ui_state::UiState::open(PathBuf::from("/nonexistent/ui.json"));
    let reply = crate::boundary::dispatch::dispatch(
        &world.deps(),
        &mut ui,
        "1000",
        &crate::boundary::Action::Nudge {
            workspace: crate::naming::leaf(&(world.workspace())),
            agent: AGENT.to_owned(),
        },
    );
    assert_eq!(reply, Ok(Reply::Nudged));
    let rows = tail(&world.state(), usize::MAX);
    assert_eq!(rows.len(), 1, "the launch is the whole trail");
    let advance = rows.first().expect("the advance row");
    assert_eq!(advance.argv.get(1).map(String::as_str), Some("advance"));
    assert_eq!(advance.argv.get(3).map(String::as_str), Some(AGENT));
    assert_eq!(advance.exit, DETACHED_EXIT);
}

/// A launch that never happened is a refusal, not a silent success — and the
/// §4.2 trail carries the synthetic-failure row saying so.
#[test]
fn a_nudge_whose_fork_never_landed_refuses_and_leaves_the_row() {
    let world = World::new();
    world.repo();
    let deps = Deps {
        lernie: Cli::new("/no/such/lernie"),
        ..world.deps()
    };
    let refused = advance(&deps, "1000", &world.workspace(), AGENT);
    assert!(refused.is_err(), "{refused:?}");
    let rows = tail(&world.state(), usize::MAX);
    assert_eq!(rows.len(), 1);
    assert_ne!(rows[0].exit, DETACHED_EXIT, "a fork that never landed");
}
