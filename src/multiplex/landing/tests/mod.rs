//! The landing repair (§16.3, bl-7e54), against **real balls landings** on
//! disk — founded by `balls::substrate::found_landing`, damaged the way this
//! box's live world was damaged (see [`world`]), then converged.

mod world;

use super::*;
use world::World;

#[test]
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
/// `XDG_DATA_HOME` resolved
/// `/home/…/.local/state/balls/clones/…` — the operator's own landing, outside
/// any world. A tracker-less landing there is the user's file and balls' own
/// boundary governs it; yog must not reach out and rewrite it.
#[test]
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

#[test]
fn a_healthy_landing_is_untouched_byte_for_byte() {
    let world = World::new();
    world.found();
    let (before, head) = (world.schedule(), world.head());
    assert!(!converge(&world.edge, &world.root).expect("converge"));
    assert_eq!(world.schedule(), before, "no rewrite");
    assert_eq!(world.head(), head, "no commit");
}

#[test]
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

#[test]
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

#[test]
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

#[test]
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

#[test]
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

#[test]
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

/// The instrumentation bl-1ce0 asked for, on the failure shape it was filed
/// from: a bare `NotFound` out of any of converge's reads or forks used to
/// reach the operator as one word, naming neither the step nor the path.
#[test]
fn a_sited_error_keeps_its_kind_and_names_the_step_and_the_path() {
    let bare = io::Error::new(io::ErrorKind::NotFound, "No such file or directory");
    let err = sited(
        "read the landing schedule",
        Path::new("/home/u/w"),
        Err::<(), _>(bare),
    )
    .expect_err("the error survives");
    assert_eq!(err.kind(), io::ErrorKind::NotFound, "matchable as before");
    assert_eq!(
        err.to_string(),
        "read the landing schedule (/home/u/w): No such file or directory"
    );
}

/// The pass-through half: a site costs nothing on the path everything takes.
#[test]
fn a_sited_success_is_the_value_itself() {
    assert_eq!(
        sited("read the landing schedule", Path::new("/home/u/w"), Ok(7)).expect("ok"),
        7
    );
}

#[test]
fn every_report_arm_is_quiet_about_the_verb() {
    // Reporting never returns a verdict — the verb's exit is balls', whatever
    // the repair did. All three arms run for the branch, not for an assertion.
    report(Ok(true));
    report(Ok(false));
    report(Err(io::Error::other("boom")));
}
