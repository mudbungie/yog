//! Answering a park: the row that is the fold's memory, the release that
//! actually lifts the hold, and the two refusals — nothing parked, and a
//! workspace that requires a confinement layer nobody has. The fixture every
//! beat here and in the sibling files is written against is [`world`], split
//! off at §12's cap: a world is built once and read by four files, a beat is
//! read by none.

/// The bare workspace repo every beat answers a park inside.
mod world;

use world::{AGENT, World};

use super::*;
use crate::boundary::dispatch::Deps;
use crate::cli_outbound::Cli;
use crate::control::judge::Answers;
use crate::control::policy::CAPABILITY_YAML;
use crate::opslog::{DETACHED_EXIT, tail};
use std::path::{Path, PathBuf};

#[test]
fn an_answer_writes_the_row_the_control_folds_and_launches_the_release() {
    let world = World::new();
    world.repo();
    world.park(AGENT, "toolu_42");
    let reply = answer_hold(
        &world.deps(),
        "1000",
        &world.workspace(),
        AGENT,
        Ruling::Pass,
    )
    .expect("something is parked");
    assert_eq!(
        reply,
        Reply::Answered {
            tool_use: "toolu_42".to_owned(),
            tool: "bash".to_owned(),
            ruling: Ruling::Pass,
            advanced: true,
        }
    );
    let rows = tail(&world.state(), usize::MAX);
    // The row is the grammar the fold reads — and the fold reads it back.
    let answer = rows.first().expect("the answer row");
    assert_eq!(
        answer.argv,
        vec!["yog-control", "answer", "toolu_42", "pass"]
    );
    assert_eq!(
        Answers::fold(&rows).ruling(
            "toolu_42",
            AGENT,
            crate::control::classify::Effect::Destructive,
            &crate::control::policy::Policy::default(),
        ),
        Ruling::Pass,
    );
    // …and the release was launched, detached, as its own logged row.
    let advance = rows.get(1).expect("the advance row");
    assert_eq!(advance.argv.get(1).map(String::as_str), Some("advance"));
    assert_eq!(advance.exit, DETACHED_EXIT);
}

/// And through the boundary's own chokepoint, which is the door every seat
/// actually uses — the family must be reachable from there, not only from its
/// own module.
#[test]
fn the_answer_is_reachable_from_the_chokepoint_every_seat_enters() {
    let world = World::new();
    world.repo();
    world.park(AGENT, "toolu_1");
    let mut ui = crate::ui_state::UiState::open(PathBuf::from("/nonexistent/ui.json"));
    let through = crate::boundary::dispatch::dispatch(
        &world.deps(),
        &mut ui,
        "1000",
        &crate::boundary::Action::AnswerHold {
            workspace: crate::naming::leaf(&(world.workspace())),
            agent: AGENT.to_owned(),
            ruling: Ruling::Hold,
        },
    );
    assert!(matches!(through, Ok(Reply::Answered { .. })));
}

#[test]
fn keeping_it_parked_writes_the_row_and_launches_nothing() {
    let world = World::new();
    world.repo();
    world.park(AGENT, "toolu_7");
    let reply = answer_hold(
        &world.deps(),
        "1000",
        &world.workspace(),
        AGENT,
        Ruling::Hold,
    )
    .expect("something is parked");
    assert!(matches!(
        reply,
        Reply::Answered {
            advanced: false,
            ruling: Ruling::Hold,
            ..
        }
    ));
    let rows = tail(&world.state(), usize::MAX);
    assert_eq!(rows.len(), 1, "a hold answer drives nothing");
}

#[test]
fn a_refusal_releases_too_because_a_decline_is_in_band() {
    let world = World::new();
    world.repo();
    world.park(AGENT, "toolu_8");
    let reply = answer_hold(
        &world.deps(),
        "1000",
        &world.workspace(),
        AGENT,
        Ruling::Refuse,
    )
    .expect("something is parked");
    assert!(matches!(reply, Reply::Answered { advanced: true, .. }));
}

#[test]
fn a_failed_launch_is_still_an_answer_and_still_a_row() {
    let world = World::new();
    world.repo();
    world.park(AGENT, "toolu_9");
    let mut deps = world.deps();
    deps.lernie = Cli::new("/no/such/lernie");
    let reply = answer_hold(&deps, "1000", &world.workspace(), AGENT, Ruling::Pass)
        .expect("the answer is durable whatever the launch does");
    assert!(matches!(
        reply,
        Reply::Answered {
            advanced: false,
            ..
        }
    ));
    // Both rows land: the answer, then the §4.2 synthetic failure for the fork
    // that never happened.
    let rows = tail(&world.state(), usize::MAX);
    assert_eq!(rows.len(), 2);
    assert!(!rows[1].stderr.is_empty());
}

#[test]
fn answering_where_nothing_is_parked_refuses_and_writes_nothing() {
    let world = World::new();
    world.repo();
    let err = answer_hold(
        &world.deps(),
        "1000",
        &world.workspace(),
        AGENT,
        Ruling::Pass,
    )
    .expect_err("an answer aimed at nothing says so");
    assert!(err.contains("nothing is held"), "{err}");
    assert!(tail(&world.state(), usize::MAX).is_empty());
}

/// The §4.11 item-8 confinement refusal — its own file at §12's cap, on the
/// seam the ruling draws: answering a park is what this module *does*, and
/// refusing a birth for a wall that is not there is a gate it also carries.
mod confinement;

/// The §4.9 fifth rung's floor, beside the answer it shares a fold with — its
/// own file on the same seam its writer is split along (bl-94b4).
mod floor;

/// The §8.2 nudge over this family's own launch (bl-9bef): the same detached
/// `advance`, with no park in front of it.
mod nudge;
