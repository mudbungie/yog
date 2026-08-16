//! The deliver executor (VISION V3.2, bl-c2bd): what it answers, the
//! `["yog-step","deliver"]` line it leaves either way, and the chokepoint's
//! route to it.

use super::super::deliver;
use super::{TS, World};
use crate::boundary::reply::Reply;
use crate::fan::Verb;
use crate::git_tree::tests::git::run_git;
use crate::opslog;

/// One commit in a candidate's worktree — the work a variant left there.
fn work(worktree: &std::path::Path) {
    std::fs::write(worktree.join("won.rs"), "fn won() {}\n").unwrap();
    run_git(worktree, &["add", "won.rs"]);
    run_git(worktree, &["config", "user.email", "t@t.local"]);
    run_git(worktree, &["config", "user.name", "Tester"]);
    run_git(worktree, &["config", "commit.gpgsign", "false"]);
    run_git(worktree, &["commit", "-q", "-m", "candidate work"]);
}

#[test]
fn a_delivery_answers_the_four_identities_and_leaves_one_step() {
    let world = World::new();
    let deps = world.deps();
    let candidate = crate::fan::open(
        &World::obligation(),
        &world.project,
        &deps.world.balls_layout(),
        1,
    )
    .unwrap()
    .remove(0);
    work(&candidate.worktree);
    let reply = deliver(
        &deps,
        TS,
        &World::obligation(),
        &candidate.handle,
        "take it",
    )
    .unwrap();
    let Reply::Delivered(delivery) = reply else {
        panic!("a delivery answers its identities, got {reply:?}");
    };
    assert_eq!(delivery.target, format!("work/{}", super::BALL));
    assert!(delivery.commit.is_some(), "work landed: {delivery:?}");
    assert_eq!(world.steps(), vec![("deliver".to_owned(), 0)]);
}

#[test]
fn a_delivery_balls_refuses_is_a_failure_line_and_a_refusal() {
    let world = World::new();
    let deps = world.deps();
    let refusal = deliver(&deps, TS, &World::obligation(), "at-deadbeef", "nope").unwrap_err();
    assert!(refusal.contains("unknown attempt handle"), "{refusal}");
    assert_eq!(
        world.steps(),
        vec![("deliver".to_owned(), opslog::SYNTHETIC_EXIT)]
    );
}

/// The chokepoint's one `Fan` arm routes the third verb here too (§8.5).
#[test]
fn the_chokepoint_routes_a_delivery_to_this_family() {
    let world = World::new();
    let deps = world.deps();
    let mut ui = crate::ui_state::UiState::open(std::path::PathBuf::from("/nonexistent/ui.json"));
    let candidate = crate::fan::open(
        &World::obligation(),
        &world.project,
        &deps.world.balls_layout(),
        1,
    )
    .unwrap()
    .remove(0);
    work(&candidate.worktree);
    let reply = crate::boundary::dispatch::dispatch(
        &deps,
        &mut ui,
        TS,
        &crate::boundary::Action::Fan(Verb::Deliver {
            obligation: World::obligation(),
            handle: candidate.handle.clone(),
            summary: "routed".to_owned(),
        }),
    );
    let Ok(Reply::Delivered(delivery)) = reply else {
        panic!("the table's Fan arm routes a delivery, got {reply:?}");
    };
    assert_eq!(delivery.target, format!("work/{}", super::BALL));
}
