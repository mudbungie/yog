//! The §9.2 retention the retirement reads: what the clock's own declaration
//! does to the source ref an attempt was opened from — undeclared, declared
//! and expired, and declared and still standing.

use super::super::retire;
use super::{TS, World};
use crate::boundary::reply::Reply;
use crate::opslog;

#[test]
fn an_undeclared_retention_releases_the_worktree_and_keeps_the_source_ref() {
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
    let reply = retire(&deps, TS, &World::obligation(), &candidate.handle).unwrap();
    assert_eq!(reply, Reply::Retired { discarded: false });
    assert!(!candidate.worktree.exists(), "the worktree went");
    // The ref stayed: a second retirement still finds the attempt.
    assert!(retire(&deps, TS, &World::obligation(), &candidate.handle).is_ok());
    assert_eq!(
        world.steps(),
        vec![("retire".to_owned(), 0), ("retire".to_owned(), 0)],
    );
}

#[test]
fn a_declared_and_expired_retention_takes_the_source_ref_too() {
    let world = World::new();
    let deps = world.deps();
    world.retention("0");
    let candidate = crate::fan::open(
        &World::obligation(),
        &world.project,
        &deps.world.balls_layout(),
        1,
    )
    .unwrap()
    .remove(0);
    let reply = retire(&deps, TS, &World::obligation(), &candidate.handle).unwrap();
    assert_eq!(reply, Reply::Retired { discarded: true });
    // The ref is gone, so the handle is refused rather than re-minted — and
    // that refusal is a failure line, not a silence.
    let refusal = retire(&deps, TS, &World::obligation(), &candidate.handle).unwrap_err();
    assert!(refusal.contains("unknown attempt handle"), "{refusal}");
    assert_eq!(
        world.steps(),
        vec![
            ("retire".to_owned(), 0),
            ("retire".to_owned(), opslog::SYNTHETIC_EXIT),
        ],
    );
}

/// A retention declared but not yet expired keeps the ref: the policy is a
/// keep, not a switch.
#[test]
fn a_retention_that_has_not_expired_keeps_the_ref() {
    let world = World::new();
    let deps = world.deps();
    // Ten years of keep over a fixture commit that is months old at most.
    world.retention("5256000");
    let candidate = crate::fan::open(
        &World::obligation(),
        &world.project,
        &deps.world.balls_layout(),
        1,
    )
    .unwrap()
    .remove(0);
    let reply = retire(&deps, TS, &World::obligation(), &candidate.handle).unwrap();
    assert_eq!(reply, Reply::Retired { discarded: false });
}
