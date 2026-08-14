//! The two boundary executors against a real project repo and a real trail:
//! what each answers, and the `ops.jsonl` line each leaves either way.

use std::path::PathBuf;
use std::sync::Arc;

use tempfile::{TempDir, tempdir};

use super::{retire, spread};
use crate::boundary::dispatch::Deps;
use crate::boundary::reply::Reply;
use crate::boundary::tests::snapshot;
use crate::cli_outbound::Cli;
use crate::fan::Obligation;
use crate::git_tree::tests::git::run_git;
use crate::opslog::{self, Origin};
use crate::start::Prepared;

const BALL: &str = "bl-1f2a";
const TS: &str = "2026-08-13T00:00:00Z";

/// A world with a real project repo and a balls space of its own, reached the
/// way production reaches it — through the composed [`Env`](crate::xdg::Env).
struct World {
    dir: TempDir,
    project: PathBuf,
}

impl World {
    fn new() -> World {
        let dir = tempdir().unwrap();
        let project = dir.path().join("proj");
        std::fs::create_dir_all(&project).unwrap();
        run_git(&project, &["init", "-q", "-b", "main"]);
        run_git(&project, &["config", "user.email", "t@t.local"]);
        run_git(&project, &["config", "user.name", "Tester"]);
        run_git(&project, &["config", "commit.gpgsign", "false"]);
        std::fs::write(project.join("README.md"), "x\n").unwrap();
        run_git(&project, &["add", "README.md"]);
        run_git(&project, &["commit", "-q", "-m", "found"]);
        World { dir, project }
    }

    fn state(&self) -> PathBuf {
        self.dir.path().join("state")
    }

    fn workspace(&self) -> PathBuf {
        self.dir.path().join("workspaces").join("cobalt-gecko")
    }

    /// `YOG_MARKS` names the balls space outright (§16.3), which is what makes
    /// `balls_layout()` — and so every attempt path — this fixture's own.
    fn deps(&self) -> Deps {
        let ws = self.workspace();
        Deps {
            lernie: Cli::new("/usr/bin/true"),
            bl: Cli::new("/no/such/bl"),
            state_root: self.state(),
            yog_binary: PathBuf::from("/no/such/yog"),
            world: crate::xdg::Env::from_pairs([
                (
                    "HOME",
                    self.dir.path().join("home").to_string_lossy().into_owned(),
                ),
                (
                    "YOG_MARKS",
                    self.dir.path().join("marks").to_string_lossy().into_owned(),
                ),
            ]),
            home: self.dir.path().join("home"),
            yog_data_root: self.dir.path().join("data"),
            balls_state_root: self.dir.path().join("balls"),
            snapshot: Arc::new({
                let mut snap = snapshot(&ws, "cobalt-gecko", Vec::new(), Vec::new());
                // The enumerated set the obligation's project name resolves
                // over (REMOTE §8).
                snap.projects = vec![self.project.clone(), self.dir.path().join("not-a-repo")];
                snap
            }),
            mint_seed: 7,
        }
    }

    fn obligation() -> Obligation {
        Obligation {
            project: "proj".to_owned(),
            ball: Some(BALL.to_owned()),
        }
    }

    fn prepared(&self) -> Prepared {
        Prepared {
            workspace: "cobalt-gecko".to_owned(),
            binding: Some(self.dir.path().join("claim")),
            goal: "Ball bl-1f2a: do it".to_owned(),
            origin: Origin::Balls,
        }
    }

    /// Declare a retention for this project in the clock's own settings file.
    fn retention(&self, keep_min: &str) {
        std::fs::create_dir_all(self.state()).unwrap();
        std::fs::write(
            self.state().join(crate::app::cadence::CADENCE_YAML),
            format!(
                "retention:\n  {}:\n    keep_min: {keep_min}\n",
                self.project.display()
            ),
        )
        .unwrap();
    }

    /// The trail's steps, newest last: `(step, exit)`.
    fn steps(&self) -> Vec<(String, i32)> {
        opslog::tail(&self.state(), usize::MAX)
            .into_iter()
            .filter_map(|e| Some((e.argv.get(1)?.clone(), e.exit)))
            .collect()
    }
}

#[test]
fn a_fan_answers_one_rebound_start_per_candidate_and_leaves_one_step() {
    let world = World::new();
    let deps = world.deps();
    let reply = spread(&deps, TS, &world.prepared(), &World::obligation(), 2).unwrap();
    let Reply::Fanned(variants) = reply else {
        panic!("a fan answers with its candidates");
    };
    assert_eq!(variants.len(), 2);
    let bindings: Vec<PathBuf> = variants
        .iter()
        .map(|v| v.binding.clone().unwrap())
        .collect();
    assert_ne!(bindings[0], bindings[1]);
    for binding in &bindings {
        assert!(binding.is_dir(), "balls placed {binding:?}");
    }
    assert_eq!(world.steps(), vec![("fan".to_owned(), 0)]);
}

#[test]
fn a_fan_that_balls_refuses_is_a_failure_line_and_a_refusal() {
    let world = World::new();
    let deps = world.deps();
    let nowhere = Obligation {
        project: "not-a-repo".to_owned(),
        ball: None,
    };
    let refusal = spread(&deps, TS, &world.prepared(), &nowhere, 2).unwrap_err();
    assert!(!refusal.is_empty());
    assert_eq!(
        world.steps(),
        vec![("fan".to_owned(), opslog::SYNTHETIC_EXIT)]
    );
}

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

/// The chokepoint routes both gestures here (§8.5): a click, a line and a
/// deposit spend one implementation, so the table's arms are asserted on the
/// table rather than on this module's doors alone.
#[test]
fn the_chokepoint_routes_a_fan_and_a_retirement_to_this_family() {
    let world = World::new();
    let deps = world.deps();
    let mut ui = crate::ui_state::UiState::open(PathBuf::from("/nonexistent/ui.json"));
    let fanned = crate::boundary::dispatch::dispatch(
        &deps,
        &mut ui,
        TS,
        &crate::boundary::Action::Fan {
            prepared: world.prepared(),
            obligation: World::obligation(),
            n: 2,
        },
    );
    let Ok(Reply::Fanned(variants)) = fanned else {
        panic!("the table's Fan arm answers with candidates, got {fanned:?}");
    };
    // The handle is the binding's leaf, which is how a seat reads it back.
    let handle = variants[0]
        .binding
        .as_ref()
        .unwrap()
        .file_name()
        .unwrap()
        .to_string_lossy()
        .into_owned();
    assert_eq!(
        crate::boundary::dispatch::dispatch(
            &deps,
            &mut ui,
            TS,
            &crate::boundary::Action::Retire {
                obligation: World::obligation(),
                handle,
            },
        ),
        Ok(Reply::Retired { discarded: false }),
    );
}
