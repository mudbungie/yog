//! The landing repair (§16.3, bl-7e54), against **real balls landings** on
//! disk — founded by `balls::substrate::found_landing`, damaged the way this
//! box's live world was damaged, then converged.
//!
//! The damage fixture is copied from the live world rather than invented: a
//! schedule wiring only `bl-delivery`, over an OLDER balls' phase vocabulary
//! (`drop.post`, `claim.pre`, `unclaim.pre` — three keys balls 0.5.9's default
//! does not contain) and with no `show` hook at all. That is what the operator's
//! stale seed template actually produced, and it is why the repair has to
//! re-derive the whole schedule instead of patching two names into it.

use super::*;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use tempfile::TempDir;

/// The live world's damaged schedule, verbatim (see the ball's premise-check
/// comment): only `bl-delivery`, on phase keys from a retired balls, and no
/// `show`.
const STALE: &str = r#"[hooks]
"claim.post" = ["bl-delivery"]
"claim.pre" = ["bl-delivery"]
"close.post" = ["bl-delivery"]
"close.pre" = ["bl-delivery"]
"drop.post" = ["bl-delivery"]
"prime.post" = ["bl-delivery"]
"unclaim.post" = ["bl-delivery"]
"unclaim.pre" = ["bl-delivery"]
"#;

/// A scratch world: balls' two homes, a tools dir carrying the `bl` sibling
/// roster yog's world always seeds, and a project directory to address.
struct World {
    _dir: TempDir,
    edge: Edge,
    landing: PathBuf,
    /// `<yog-data-root>/world` — the containment gate's subject.
    root: PathBuf,
}

impl World {
    /// Lay the scratch world. Nothing is founded yet — the landing path is
    /// computed by balls' own `clone_dir` fold, never spelled here.
    fn new() -> World {
        let dir = tempfile::tempdir().expect("scratch world");
        // The real shape: balls' two homes live INSIDE the world subtree, which
        // is what earns the repair the right to rewrite a landing there.
        let anchor = dir.path();
        let root = anchor.join("yog").join("world");
        std::fs::create_dir_all(&root).expect("world root");
        let tools = root.join("tools");
        std::fs::create_dir_all(&tools).expect("tools dir");
        // The sibling binaries balls' seed binds and prunes against. Content is
        // irrelevant — `seed::sibling` only asks whether the path exists — but
        // they must be present or the seed prunes the very entries under test.
        for name in [tools::BL, tools::BL_DELIVERY, tools::BL_TRACKER] {
            let path = tools.join(name);
            std::fs::write(&path, "#!/bin/sh\nexit 0\n").expect("sibling");
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755))
                .expect("sibling mode");
        }
        let project = anchor.join("proj");
        std::fs::create_dir_all(&project).expect("project dir");
        let edge = Edge::resolve(
            anchor.to_path_buf(),
            Some(root.join("config").to_string_lossy().into_owned()),
            Some(root.join("state").to_string_lossy().into_owned()),
            project,
            Some("tester".to_owned()),
            None,
            Some(tools.join(tools::BL)),
            None,
            None,
            false,
            None,
        );
        let landing = edge.xdg.clone_dir(&edge.invocation_path).landing();
        World {
            _dir: dir,
            edge,
            landing,
            root,
        }
    }

    /// Found the landing the way `bl prime` does — balls' own seed, against
    /// this world's tools dir, so it comes up HEALTHY.
    fn found(&self) {
        substrate::found_landing(
            &self.landing,
            &self.edge.xdg,
            self.edge.exe_dir.as_deref(),
            "tester",
        )
        .expect("found landing");
    }

    /// Overwrite the founded landing's schedule with the live world's damage
    /// and commit it, reproducing a landing seeded from the stale template.
    fn damage(&self) {
        std::fs::write(self.plugins(), STALE).expect("write stale schedule");
        git(&self.landing, &["add", "-A"]).expect("stage");
        git(&self.landing, &["commit", "-q", "-m", "stale seed"]).expect("commit");
    }

    fn plugins(&self) -> PathBuf {
        self.landing.join("config").join("plugins.toml")
    }

    fn scalars(&self) -> PathBuf {
        self.landing.join("config").join("balls.toml")
    }

    fn schedule(&self) -> String {
        std::fs::read_to_string(self.plugins()).unwrap_or_default()
    }

    fn head(&self) -> String {
        git(&self.landing, &["rev-parse", "HEAD"]).expect("head")
    }
}

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
}

#[test]
fn every_report_arm_is_quiet_about_the_verb() {
    // Reporting never returns a verdict — the verb's exit is balls', whatever
    // the repair did. All three arms run for the branch, not for an assertion.
    report(Ok(true));
    report(Ok(false));
    report(Err(io::Error::other("boom")));
}
