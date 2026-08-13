//! Answering a park: the row that is the fold's memory, the release that
//! actually lifts the hold, and the two refusals — nothing parked, and a
//! workspace that requires a confinement layer nobody has.

use super::*;
use crate::boundary::dispatch::Deps;
use crate::boundary::tests::snapshot;
use crate::cli_outbound::Cli;
use crate::control::judge::Answers;
use crate::control::policy::CAPABILITY_YAML;
use crate::opslog::{DETACHED_EXIT, tail};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tempfile::{TempDir, tempdir};

/// A world with a bare workspace repo the mark can be written into, and a
/// state root the row lands in.
struct World {
    dir: TempDir,
}

impl World {
    fn new() -> World {
        World {
            dir: tempdir().expect("tempdir"),
        }
    }

    fn workspace(&self) -> PathBuf {
        self.dir.path().join("names").join("alba")
    }

    fn state(&self) -> PathBuf {
        self.dir.path().join("state")
    }

    fn deps(&self) -> Deps {
        Deps {
            // `true` exists on every platform the suite runs on and exits 0 —
            // enough to prove the detached launch happened without driving a
            // real conversation.
            lernie: Cli::new("/usr/bin/true"),
            bl: Cli::new("/no/such/bl"),
            state_root: self.state(),
            home: self.dir.path().join("home"),
            yog_data_root: self.dir.path().join("data"),
            balls_state_root: self.dir.path().join("balls"),
            yog_binary: PathBuf::from("/no/such/yog"),
            world: crate::xdg::Env::from_env(),
            snapshot: Arc::new(snapshot(&self.workspace(), "alba", Vec::new(), Vec::new())),
            mint_seed: 7,
        }
    }

    fn git(&self, args: &[&str]) -> std::process::Output {
        crate::git_env::git()
            .arg("--git-dir")
            .arg(self.workspace().join("repo.git"))
            .args(args)
            .output()
            .expect("git runs")
    }

    fn repo(&self) {
        std::fs::create_dir_all(self.workspace().join("repo.git")).unwrap();
        self.git(&["init", "--bare", "-q"]);
    }

    /// Park `agent` on `tool_use`, exactly as lernie's seam does.
    fn park(&self, agent: &str, tool_use: &str) {
        let staged = self.dir.path().join("mark.json");
        std::fs::write(
            &staged,
            format!(r#"{{"tool_use_id":"{tool_use}","tool":"bash","reason":"open-world"}}"#),
        )
        .unwrap();
        let hashed = self.git(&["hash-object", "-w", "--", &staged.to_string_lossy()]);
        let oid = String::from_utf8_lossy(&hashed.stdout).trim().to_owned();
        self.git(&[
            "update-ref",
            &format!("refs/lernie/held/{agent}"),
            oid.as_str(),
        ]);
    }

    /// Commit `capability.yaml` onto `config/default`.
    fn policy(&self, text: &str) {
        let staged = self.dir.path().join(CAPABILITY_YAML);
        std::fs::write(&staged, text).unwrap();
        let hashed = self.git(&["hash-object", "-w", "--", &staged.to_string_lossy()]);
        let blob = String::from_utf8_lossy(&hashed.stdout).trim().to_owned();
        self.git(&[
            "update-index",
            "--add",
            "--cacheinfo",
            "100644",
            &blob,
            CAPABILITY_YAML,
        ]);
        let tree = self.git(&["write-tree"]);
        let tree = String::from_utf8_lossy(&tree.stdout).trim().to_owned();
        let commit = self.git(&["commit-tree", &tree, "-m", "policy"]);
        let commit = String::from_utf8_lossy(&commit.stdout).trim().to_owned();
        self.git(&["update-ref", "refs/heads/config/default", &commit]);
    }
}

#[test]
fn an_answer_writes_the_row_the_control_folds_and_launches_the_release() {
    let world = World::new();
    world.repo();
    world.park("a-1", "toolu_42");
    let guard = crate::test_support::spawn_guard();
    let reply = answer_hold(
        &world.deps(),
        "1000",
        &world.workspace(),
        "a-1",
        Ruling::Pass,
    )
    .expect("something is parked");
    drop(guard);
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
            "a-1",
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
    world.park("a-1", "toolu_1");
    let mut ui = crate::ui_state::UiState::open(PathBuf::from("/nonexistent/ui.json"));
    let through = crate::boundary::dispatch::dispatch(
        &world.deps(),
        &mut ui,
        "1000",
        &crate::boundary::Action::AnswerHold {
            workspace: world.workspace(),
            agent: "a-1".to_owned(),
            ruling: Ruling::Hold,
        },
    );
    assert!(matches!(through, Ok(Reply::Answered { .. })));
}

#[test]
fn keeping_it_parked_writes_the_row_and_launches_nothing() {
    let world = World::new();
    world.repo();
    world.park("a-1", "toolu_7");
    let reply = answer_hold(
        &world.deps(),
        "1000",
        &world.workspace(),
        "a-1",
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
    world.park("a-1", "toolu_8");
    let guard = crate::test_support::spawn_guard();
    let reply = answer_hold(
        &world.deps(),
        "1000",
        &world.workspace(),
        "a-1",
        Ruling::Refuse,
    )
    .expect("something is parked");
    drop(guard);
    assert!(matches!(reply, Reply::Answered { advanced: true, .. }));
}

#[test]
fn a_failed_launch_is_still_an_answer_and_still_a_row() {
    let world = World::new();
    world.repo();
    world.park("a-1", "toolu_9");
    let mut deps = world.deps();
    deps.lernie = Cli::new("/no/such/lernie");
    let guard = crate::test_support::spawn_guard();
    let reply = answer_hold(&deps, "1000", &world.workspace(), "a-1", Ruling::Pass)
        .expect("the answer is durable whatever the launch does");
    drop(guard);
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
        "a-1",
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
