//! The landing repair (DESIGN §16.3, bl-7e54) against **real balls landings on
//! disk** — founded by `balls::substrate::found_landing`, damaged the way this
//! box's live world was damaged (see [`world`]), then converged.
//!
//! # Why this is a test binary of its own (bl-6bf5, amended bl-fd28)
//!
//! `World::found` calls `balls::substrate::found_landing`, which forks `git`
//! **on balls' own account** — outside `yog::git_env`. That used to be an
//! ETXTBSY hazard: a fork copies every open fd, so a peer thread holding a
//! write fd on a fixture script it had just written lost the `exec` that
//! followed (`Text file busy`), and no lock yog owns can reach a fork inside
//! another crate. Measured on a 16-core box, one filter over the lib test
//! binary (`multiplex` plus the five fixture-exec families), 16 workers × 70
//! iterations each: **8 ETXTBSY failures with these beats in the lib binary, 0
//! without.**
//!
//! **bl-fd28 dissolved that reason** — every executable fixture is now written
//! by a child, so no process holds a descriptor for any fork to copy, and the
//! lock the argument turned on is deleted. What keeps these beats here is the
//! other half: a binary running its subject **in-process** must scrub its own
//! env of `yog::git_env::INHERITED`, there being no spawn boundary to do it for
//! a fork it does not perform.
//!
//! **This binary owns its process environment** (the `tests/multiplex_bl.rs`
//! precedent, for the same reason): the balls it drives runs in-process, so a
//! hook-inherited `GIT_DIR`/`GIT_INDEX_FILE` would re-aim balls' own forks at
//! the outer repo. The list scrubbed is `yog::git_env::INHERITED`, the one the
//! spawn sites use. Everything is one `#[test]` because that scrub is a
//! process-global act with no peer thread to race — the same rule and the same
//! shape as `tests/multiplex_bl.rs`.
#![allow(clippy::unwrap_used, clippy::expect_used)]

// The executable-fixture writer, shared by every integration binary that
// writes one (bl-fd28). `#[path]` because this file IS the test target's
// crate root, and a second top-level `tests/*.rs` would be a second binary.
#[path = "support/write_exec.rs"]
mod write_exec;

use yog::multiplex::landing::{commit, converge, git};
use yog::world::tools;

// The fixture seam. `#[path]` because this file IS the test target's crate
// root, so a bare `mod` would resolve to `tests/world.rs` — and a second
// top-level `tests/*.rs` is a second test binary, not a module.
#[path = "multiplex_landing/world.rs"]
mod world;
use world::World;

#[test]
fn the_landing_repair_converges_real_landings() {
    for var in yog::git_env::INHERITED {
        // SAFETY: one `#[test]` in this binary, so no peer thread exists to
        // read the environment concurrently (crate-root doc).
        unsafe { std::env::remove_var(var) };
    }
    an_unfounded_clone_is_left_alone();
    a_landing_outside_the_world_is_never_yogs_to_rewrite();
    a_healthy_landing_is_untouched_byte_for_byte();
    a_tracker_less_landing_regains_the_whole_schedule();
    the_repair_is_idempotent();
    the_repair_spends_no_scalar_config();
    an_absent_scalar_file_is_re_derived_rather_than_restored();
    a_clean_tree_seals_nothing();
    a_failing_git_becomes_an_error_carrying_its_stderr();
}

fn an_unfounded_clone_is_left_alone() {
    // Nothing to repair before a `prime` exists — and the seed that prime is
    // about to run is already correct (bl-e47b), so touching anything here
    // would be inventing state.
    let world = World::new();
    assert!(!converge(&world.edge, &world.root).expect("converge"));
    assert!(!world.landing.exists(), "no landing was conjured");
}

/// The containment gate, and the reason it exists. `yog bl` reads the world
/// from the env it was HANDED — it does not re-compose one — so a `yog bl` typed
/// at a shell that never entered the world addresses the operator's **ambient**
/// balls state. Found the hard way: an instrumented run against a scratch
/// `XDG_DATA_HOME` resolved the operator's own landing under their state home,
/// outside any world. A tracker-less landing there is the user's file and
/// balls' own boundary governs it; yog must not reach out and rewrite it.
fn a_landing_outside_the_world_is_never_yogs_to_rewrite() {
    let world = World::new();
    world.found();
    world.damage();
    let (damaged, head) = (world.schedule(), world.head());
    // Same damaged landing, judged against a world root it does not live under
    // — exactly the ambient case.
    let elsewhere = world.landing.join("not-the-world");
    assert!(
        !converge(&world.edge, &elsewhere).expect("converge"),
        "an out-of-world landing is left alone however tracker-less"
    );
    assert_eq!(world.schedule(), damaged, "not rewritten");
    assert_eq!(world.head(), head, "not committed");
}

fn a_healthy_landing_is_untouched_byte_for_byte() {
    let world = World::new();
    world.found();
    let (before, head) = (world.schedule(), world.head());
    assert!(!converge(&world.edge, &world.root).expect("converge"));
    assert_eq!(world.schedule(), before, "no rewrite");
    assert_eq!(world.head(), head, "no commit");
}

fn a_tracker_less_landing_regains_the_whole_schedule() {
    let world = World::new();
    world.found();
    world.damage();
    let damaged_head = world.head();
    // The premise the repair exists for: the tracker is gone AND so is `show`.
    assert!(!world.schedule().contains(tools::BL_TRACKER));
    assert!(!world.schedule().contains("show"));

    assert!(converge(&world.edge, &world.root).expect("converge"));

    let after = world.schedule();
    // balls' own default is back — the tracker at its phases, and the `show`
    // read hook whose absence is why `bl show` printed no worktree line.
    assert!(
        after.contains(tools::BL_TRACKER),
        "tracker restored: {after}"
    );
    assert!(after.contains("show"), "show hook restored: {after}");
    // The retired phase vocabulary is gone with it, not merged into the new.
    assert!(!after.contains("drop.post"), "stale phase dropped: {after}");
    assert_ne!(world.head(), damaged_head, "sealed as a landing commit");
}

fn the_repair_is_idempotent() {
    let world = World::new();
    world.found();
    world.damage();
    assert!(converge(&world.edge, &world.root).expect("first"));
    let (settled, head) = (world.schedule(), world.head());
    // A second pass takes the cheap way out — the gate sees a schedule that
    // names every provided plugin and stops before the seed.
    assert!(!converge(&world.edge, &world.root).expect("second"));
    assert_eq!(world.schedule(), settled);
    assert_eq!(world.head(), head, "no empty commit");
}

fn the_repair_spends_no_scalar_config() {
    let world = World::new();
    world.found();
    // A knob an operator set through `bl conf` — the repair must restore the
    // capability schedule without reverting it.
    let mine = "tasks_branch = \"balls/mine\"\nlog_level = \"debug\"\n";
    std::fs::write(world.scalars(), mine).expect("write scalars");
    world.damage();
    assert!(converge(&world.edge, &world.root).expect("converge"));
    assert_eq!(
        std::fs::read_to_string(world.scalars()).unwrap_or_default(),
        mine,
        "balls.toml carried across the re-seed"
    );
}

fn an_absent_scalar_file_is_re_derived_rather_than_restored() {
    let world = World::new();
    world.found();
    world.damage();
    std::fs::remove_file(world.scalars()).expect("remove scalars");
    assert!(converge(&world.edge, &world.root).expect("converge"));
    // Nothing to carry across, so balls' seed supplies its own default.
    assert!(
        std::fs::read_to_string(world.scalars())
            .unwrap_or_default()
            .contains("tasks_branch"),
        "the seed's balls.toml is back"
    );
}

fn a_clean_tree_seals_nothing() {
    // `commit`'s early return, reached directly: the convergence gate normally
    // guarantees a dirty tree, so this is the guard that keeps the repair
    // idempotent independently of that gate.
    let world = World::new();
    world.found();
    let head = world.head();
    commit(&world.landing, "tester").expect("commit on a clean tree");
    assert_eq!(world.head(), head, "no empty commit");
}

fn a_failing_git_becomes_an_error_carrying_its_stderr() {
    let world = World::new();
    world.found();
    let err = git(&world.landing, &["rev-parse", "--verify", "no/such/ref"])
        .expect_err("a missing ref fails");
    assert!(!err.to_string().is_empty(), "git's own words ride along");
    // …under the site, so the three forks are told apart in a warning line.
    let said = err.to_string();
    assert!(said.starts_with("git rev-parse ("), "sited: {said}");
    assert!(
        said.contains(&world.landing.display().to_string()),
        "the cwd is named: {said}"
    );
}
