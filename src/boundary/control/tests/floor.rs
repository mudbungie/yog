//! The §4.9 fifth rung's writer (bl-94b4): one row that is the fold's memory,
//! a whole descent under it, latest-row-wins in both directions, and a receipt
//! that re-derives rather than echoes.

use super::{AGENT, World};
use crate::boundary::control::set_floor;
use crate::boundary::reply::Reply;
use crate::control::classify::Effect;
use crate::control::judge::{Answers, Ruling};
use crate::control::policy::Policy;
use crate::opslog::tail;
use std::path::PathBuf;

/// [`AGENT`]'s own descent child — the §2.3 hyphenated chain one generation on.
const CHILD: &str = "20260101T000000Z-a1-20260102T000000Z-b2";
/// A *sibling* whose id merely opens with [`AGENT`]'s characters. The fold
/// matches on the hyphen boundary, so this one is not below it.
const LOOKALIKE: &str = "20260101T000000Z-a10";

/// What the control would answer for `agent` under the trail as it stands.
fn ruling(world: &World, agent: &str, effect: Effect) -> Ruling {
    Answers::fold(&tail(&world.state(), usize::MAX)).ruling(
        "toolu_x",
        agent,
        effect,
        &Policy::default(),
    )
}

#[test]
fn revoking_writes_the_row_the_control_folds_and_drives_nothing() {
    let world = World::new();
    let reply = set_floor(&world.deps(), "1000", &world.workspace(), AGENT, true);
    assert_eq!(reply.expect("written"), Reply::Floored { standing: true });

    let rows = tail(&world.state(), usize::MAX);
    assert_eq!(rows.len(), 1, "a floor is policy, not a drive");
    assert_eq!(rows[0].argv, vec!["yog-control", "floor", AGENT, "raise"]);

    // Everything above a read now waits; a read is still the job.
    assert_eq!(ruling(&world, AGENT, Effect::TargetWrite), Ruling::Hold);
    assert_eq!(ruling(&world, AGENT, Effect::Read), Ruling::Pass);
    // And a refusal stays a refusal: the floor raises, it never lowers.
    assert_eq!(ruling(&world, AGENT, Effect::Secret), Ruling::Refuse);
    // A conversation nobody floored is untouched.
    assert_eq!(ruling(&world, "b-1", Effect::TargetWrite), Ruling::Pass);
}

/// The subtree match is the point: one row covers children, including ones
/// that do not exist when it is written.
#[test]
fn the_floor_stands_over_the_whole_descent() {
    let world = World::new();
    set_floor(&world.deps(), "1000", &world.workspace(), AGENT, true).expect("written");
    assert_eq!(ruling(&world, CHILD, Effect::Process), Ruling::Hold);
    assert_eq!(
        ruling(&world, LOOKALIKE, Effect::Process),
        Ruling::Pass,
        "a sibling whose id merely starts with the same letters is not below it"
    );
}

#[test]
fn restoring_is_the_same_gesture_the_other_way_and_the_latest_row_wins() {
    let world = World::new();
    let deps = world.deps();
    set_floor(&deps, "1000", &world.workspace(), AGENT, true).expect("written");
    let back = set_floor(&deps, "1001", &world.workspace(), AGENT, false);
    assert_eq!(back.expect("written"), Reply::Floored { standing: false });

    let rows = tail(&world.state(), usize::MAX);
    assert_eq!(rows.len(), 2, "two rows, nothing else");
    assert_eq!(rows[1].argv, vec!["yog-control", "floor", AGENT, "lower"]);
    assert_eq!(ruling(&world, AGENT, Effect::TargetWrite), Ruling::Pass);
}

/// The receipt is read back off the trail, so it cannot claim a restore the
/// ancestor's standing floor did not allow.
#[test]
fn a_child_restored_under_a_floored_parent_is_told_it_is_still_floored() {
    let world = World::new();
    let deps = world.deps();
    set_floor(&deps, "1000", &world.workspace(), AGENT, true).expect("written");
    let child = set_floor(&deps, "1001", &world.workspace(), CHILD, false);
    assert_eq!(child.expect("written"), Reply::Floored { standing: true });
    assert_eq!(ruling(&world, CHILD, Effect::TargetWrite), Ruling::Hold);
}

/// And through the boundary's own chokepoint, which is the door every seat
/// actually uses.
#[test]
fn the_floor_is_reachable_from_the_chokepoint_every_seat_enters() {
    let world = World::new();
    let mut ui = crate::ui_state::UiState::open(PathBuf::from("/nonexistent/ui.json"));
    let through = crate::boundary::dispatch::dispatch(
        &world.deps(),
        &mut ui,
        "1000",
        &crate::boundary::Action::Floor {
            workspace: crate::naming::leaf(&(world.workspace())),
            agent: AGENT.to_owned(),
            raised: true,
        },
    );
    assert_eq!(through.expect("written"), Reply::Floored { standing: true });
}
