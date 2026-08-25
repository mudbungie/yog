//! **Deliver candidate** against a real project repo (VISION V3.2, §4.10
//! items 5–6): acceptance is balls' one delivery law, the acceptance mark is
//! the target's own history, and a stale sibling refuses until it has
//! incorporated what landed — the rework rule, exercised rather than believed.

use super::world::{BALL, World};
use crate::fan::{Obligation, deliver, delivered_commit, open};
use crate::git_tree::tests::git::{git_out, run_git};

use std::path::Path;

/// One commit of one file in a candidate's own worktree — the work a variant
/// agent would have left there.
fn work(worktree: &Path, file: &str, body: &str) {
    std::fs::write(worktree.join(file), body).unwrap();
    run_git(worktree, &["add", file]);
    run_git(worktree, &["config", "user.email", "t@t.local"]);
    run_git(worktree, &["config", "user.name", "Tester"]);
    run_git(worktree, &["config", "commit.gpgsign", "false"]);
    run_git(worktree, &["commit", "-q", "-m", "candidate work"]);
}

#[test]
fn acceptance_is_the_delivery_and_the_targets_history_is_the_only_mark() {
    let world = World::new();
    let obligation = World::obligation(Some(BALL));
    let target_ref = format!("work/{BALL}");
    let candidates = open(&obligation, &world.project, &world.xdg, 2).unwrap();
    let before = world.tip(&target_ref);
    work(&candidates[0].worktree, "won.rs", "fn won() {}\n");

    let delivery = deliver(
        &obligation,
        &world.project,
        &world.xdg,
        &candidates[0].handle,
        "take the winning candidate",
    )
    .unwrap();

    // The four identities are the delivery's own: the ball's ref advanced from
    // the pinned base, and the commit is the target's new tip.
    assert_eq!(delivery.target, target_ref);
    assert_eq!(delivery.base, before);
    let after = world.tip(&target_ref);
    assert_ne!(after, before, "acceptance advanced the ball's own branch");
    assert_eq!(delivery.commit.as_deref(), Some(after.as_str()));
    assert_eq!(
        delivery.source.as_deref(),
        Some(
            world
                .tip(&format!("attempt/{}", candidates[0].handle))
                .as_str()
        ),
    );
    // The squash subject carries the summary tagged with the handle — which is
    // exactly what the derived mark reads back (§4.10 item 6): the winner from
    // the history, the loser from its absence, and nothing stored anywhere.
    let subject = git_out(&world.project, &["log", "-n1", "--format=%s", &target_ref]);
    assert_eq!(
        subject,
        format!("take the winning candidate [{}]", candidates[0].handle),
    );
    assert_eq!(
        delivered_commit(&world.project, &target_ref, &candidates[0].handle),
        delivery.commit,
    );
    assert_eq!(
        delivered_commit(&world.project, &target_ref, &candidates[1].handle),
        None,
        "rejection is the absence of a delivery",
    );
    // The parent obligation is untouched one level up: delivery advanced
    // `work/<id>`, never the integration branch the ball's close delivers to.
    assert_eq!(world.tip("main"), delivery.base);
}

#[test]
fn a_stale_sibling_refuses_until_it_incorporates_what_landed() {
    let world = World::new();
    let obligation = World::obligation(Some(BALL));
    let target_ref = format!("work/{BALL}");
    let candidates = open(&obligation, &world.project, &world.xdg, 2).unwrap();
    work(&candidates[0].worktree, "first.rs", "fn first() {}\n");
    work(&candidates[1].worktree, "second.rs", "fn second() {}\n");
    deliver(
        &obligation,
        &world.project,
        &world.xdg,
        &candidates[0].handle,
        "first lands",
    )
    .unwrap();

    // The sibling is now stale by construction (§4.10 item 5): its source does
    // not contain the advanced target tip, and delivery refuses before
    // anything merges, gates or moves — yog never reconciles it.
    let refusal = deliver(
        &obligation,
        &world.project,
        &world.xdg,
        &candidates[1].handle,
        "second lands",
    )
    .unwrap_err()
    .to_string();
    assert!(!refusal.is_empty(), "the refusal says something");
    assert_eq!(
        delivered_commit(&world.project, &target_ref, &candidates[1].handle),
        None,
        "the refused delivery changed no target ref",
    );

    // Rework is source-owned: incorporate the current target in the sibling's
    // own worktree, then the same delivery lands — sequential synthesis out of
    // the law, no primitive anywhere.
    run_git(
        &candidates[1].worktree,
        &["merge", "-q", "--no-edit", &target_ref],
    );
    let delivery = deliver(
        &obligation,
        &world.project,
        &world.xdg,
        &candidates[1].handle,
        "second lands, reworked",
    )
    .unwrap();
    assert_eq!(
        delivered_commit(&world.project, &target_ref, &candidates[1].handle),
        delivery.commit,
    );
}

#[test]
fn a_mark_scan_over_an_unresolvable_target_is_an_absence_not_an_error() {
    let world = World::new();
    assert_eq!(
        delivered_commit(&world.project, "work/never-minted", "at-00000000"),
        None,
    );
}

#[test]
fn delivering_an_unknown_handle_is_refused_in_balls_own_voice() {
    let world = World::new();
    let err = deliver(
        &World::obligation(Some(BALL)),
        &world.project,
        &world.xdg,
        "at-deadbeef",
        "nope",
    )
    .unwrap_err()
    .to_string();
    assert!(err.contains("unknown attempt handle"), "{err}");
}

/// A bare project-repo obligation delivers onto the integration branch itself
/// (§4.10 item 8) — the same law with the target the project names.
#[test]
fn a_bare_obligation_delivers_onto_the_integration_branch() {
    let world = World::new();
    let obligation: Obligation = World::obligation(None);
    let candidates = open(&obligation, &world.project, &world.xdg, 2).unwrap();
    work(&candidates[0].worktree, "bare.rs", "fn bare() {}\n");
    let delivery = deliver(
        &obligation,
        &world.project,
        &world.xdg,
        &candidates[0].handle,
        "bare fan winner",
    )
    .unwrap();
    assert_eq!(delivery.target, "main");
    assert_eq!(
        delivered_commit(&world.project, "main", &candidates[0].handle),
        delivery.commit,
    );
}
