//! The fan against a **real project repo**, because what is under test is
//! balls' own attempt capability answering over real refs and real worktrees —
//! a fake would test the fake. The fixture is a project (an integration branch
//! with one commit), which is what a balls invocation path is.

mod delivery;

use std::path::PathBuf;

use tempfile::{TempDir, tempdir};

use super::spread::one_base;
use super::{Candidate, Obligation, discard, open, release, resume, spread};
use crate::git_tree::tests::git::{git_out, run_git};
use crate::opslog::Origin;
use crate::start::Prepared;
use balls::delivery_path::{attempt_branch, attempt_path};
use balls::layout::Xdg;

/// The fixture repo's integration branch.
const MAIN: &str = "main";
/// The ball whose `work/<id>` ref every ball-obligation test targets.
const BALL: &str = "bl-1f2a";

/// A project repo plus the balls layout its attempts are placed under.
struct World {
    dir: TempDir,
    project: PathBuf,
    xdg: Xdg,
}

impl World {
    fn new() -> World {
        let dir = tempdir().unwrap();
        let project = dir.path().join("proj");
        std::fs::create_dir_all(&project).unwrap();
        run_git(&project, &["init", "-q", "-b", MAIN]);
        run_git(&project, &["config", "user.email", "t@t.local"]);
        run_git(&project, &["config", "user.name", "Tester"]);
        run_git(&project, &["config", "commit.gpgsign", "false"]);
        std::fs::write(project.join("README.md"), "the project\n").unwrap();
        run_git(&project, &["add", "README.md"]);
        run_git(&project, &["commit", "-q", "-m", "found"]);
        let xdg = Xdg::with(
            &dir.path().join("home"),
            None,
            Some(&dir.path().join("state").to_string_lossy()),
        );
        World { dir, project, xdg }
    }

    fn obligation(ball: Option<&str>) -> Obligation {
        Obligation {
            project: "proj".to_owned(),
            ball: ball.map(str::to_owned),
        }
    }

    /// The commit a ref names right now.
    fn tip(&self, refname: &str) -> String {
        git_out(&self.project, &["rev-parse", refname])
    }

    fn branch_exists(&self, refname: &str) -> bool {
        crate::git_env::git()
            .arg("-C")
            .arg(&self.project)
            .args(["rev-parse", "--verify", "-q", refname])
            .output()
            .unwrap()
            .status
            .success()
    }
}

/// The ordinary prepared start a fan spreads.
fn prepared(dir: &TempDir) -> Prepared {
    Prepared {
        workspace: "cobalt-gecko".to_owned(),
        binding: Some(dir.path().join("claim")),
        goal: "Ball bl-1f2a: do the thing".to_owned(),
        origin: Origin::Balls,
    }
}

#[test]
fn a_ball_fan_forks_every_candidate_off_the_one_work_ref_tip() {
    let world = World::new();
    let candidates = open(
        &World::obligation(Some(BALL)),
        &world.project,
        &world.xdg,
        3,
    )
    .unwrap();
    assert_eq!(candidates.len(), 3);
    // The target balls minted is the ball's own `work/<id>` — the very ref
    // `bl close` later delivers onto the integration branch (§4.10 item 1).
    let target = world.tip(&format!("work/{BALL}"));
    let handles: Vec<&str> = candidates.iter().map(|c| c.handle.as_str()).collect();
    for candidate in &candidates {
        assert_eq!(candidate.base, target, "every sibling forks at one commit");
        assert!(candidate.worktree.is_dir(), "balls placed the worktree");
        assert!(
            candidate.handle.starts_with("at-"),
            "balls mints the handle, and it is unmistakable for a ball id: {}",
            candidate.handle
        );
        // The path is balls', not yog's: its own formula reproduces it.
        assert_eq!(
            candidate.worktree,
            attempt_path(
                &world.xdg,
                &world.project.to_string_lossy(),
                &candidate.handle
            )
        );
        assert!(world.branch_exists(&attempt_branch(&candidate.handle)));
    }
    let mut unique = handles.clone();
    unique.sort_unstable();
    unique.dedup();
    assert_eq!(
        unique.len(),
        3,
        "no two candidates share a handle: {handles:?}"
    );
}

#[test]
fn a_bare_obligation_targets_the_projects_own_integration_branch() {
    let world = World::new();
    let candidates = open(&World::obligation(None), &world.project, &world.xdg, 1).unwrap();
    assert_eq!(candidates[0].base, world.tip(MAIN));
    assert!(
        !world.branch_exists(&format!("work/{BALL}")),
        "a bare fan mints no ball ref"
    );
}

#[test]
fn a_fan_of_none_materializes_nothing() {
    let world = World::new();
    assert_eq!(
        open(
            &World::obligation(Some(BALL)),
            &world.project,
            &world.xdg,
            0
        )
        .unwrap(),
        vec![]
    );
}

#[test]
fn a_path_that_is_no_repo_is_refused_in_balls_own_voice() {
    let world = World::new();
    let nowhere = Obligation {
        project: "not-a-repo".to_owned(),
        ball: None,
    };
    let repo = world.dir.path().join("not-a-repo");
    let err = open(&nowhere, &repo, &world.xdg, 1)
        .unwrap_err()
        .to_string();
    assert!(!err.is_empty(), "the refusal says something");
}

#[test]
fn members_that_forked_at_different_commits_are_not_a_cohort() {
    let candidate = |base: &str| Candidate {
        handle: format!("at-{base}"),
        worktree: PathBuf::from("/w"),
        base: base.to_owned(),
    };
    // Equal bases pass through untouched, in order.
    let same = vec![candidate("aaa"), candidate("aaa")];
    assert_eq!(one_base(same.clone()).unwrap(), same);
    let err = one_base(vec![candidate("aaa"), candidate("bbb")])
        .unwrap_err()
        .to_string();
    assert!(err.contains("2 different base commits"), "{err}");
    assert!(err.contains("aaa, bbb"), "the refusal names them: {err}");
}

#[test]
fn a_fan_of_one_is_the_ordinary_claim_binding_and_materializes_no_attempt() {
    let world = World::new();
    let start = prepared(&world.dir);
    for n in [0, 1] {
        assert_eq!(
            spread(
                &start,
                &World::obligation(Some(BALL)),
                &world.project,
                &world.xdg,
                n
            )
            .unwrap(),
            vec![start.clone()],
        );
    }
    assert!(
        !world.branch_exists(&format!("work/{BALL}")),
        "nothing was asked of balls at all"
    );
}

#[test]
fn above_one_every_variant_is_the_same_start_bound_to_its_own_worktree() {
    let world = World::new();
    let start = prepared(&world.dir);
    let variants = spread(
        &start,
        &World::obligation(Some(BALL)),
        &world.project,
        &world.xdg,
        2,
    )
    .unwrap();
    assert_eq!(variants.len(), 2);
    let bindings: Vec<&PathBuf> = variants
        .iter()
        .map(|v| v.binding.as_ref().unwrap())
        .collect();
    assert_ne!(bindings[0], bindings[1], "no two share a mutable checkout");
    for variant in &variants {
        assert_ne!(
            variant.binding, start.binding,
            "the claim is not a candidate"
        );
        // Everything else is the start the operator prepared, unchanged: the
        // fan varies the binding and nothing else.
        assert_eq!(
            Prepared {
                binding: start.binding.clone(),
                ..variant.clone()
            },
            start
        );
    }
}

#[test]
fn release_keeps_the_source_ref_and_discard_takes_it() {
    let world = World::new();
    let obligation = World::obligation(Some(BALL));
    let repo = world.project.clone();
    let candidate = open(&obligation, &repo, &world.xdg, 1).unwrap().remove(0);
    let branch = attempt_branch(&candidate.handle);

    release(&obligation, &repo, &world.xdg, &candidate.handle).unwrap();
    assert!(!candidate.worktree.exists(), "the worktree went");
    assert!(
        world.branch_exists(&branch),
        "a rejected candidate stays addressable"
    );

    discard(&obligation, &repo, &world.xdg, &candidate.handle).unwrap();
    assert!(!candidate.worktree.exists());
    assert!(!world.branch_exists(&branch), "retention expired: both go");

    // And a discarded handle is refused rather than quietly re-minted.
    let err = release(&obligation, &repo, &world.xdg, &candidate.handle)
        .unwrap_err()
        .to_string();
    assert!(err.contains("unknown attempt handle"), "{err}");
}

#[test]
fn a_resumed_attempt_is_the_same_candidate_re_materialized() {
    let world = World::new();
    let obligation = World::obligation(Some(BALL));
    let repo = world.project.clone();
    let candidate = open(&obligation, &repo, &world.xdg, 1).unwrap().remove(0);
    release(&obligation, &repo, &world.xdg, &candidate.handle).unwrap();
    let again = resume(&obligation, &repo, &world.xdg, &candidate.handle).unwrap();
    assert_eq!(again.handle(), candidate.handle);
    assert_eq!(again.worktree(), candidate.worktree);
    assert_eq!(again.base(), candidate.base);
    assert!(candidate.worktree.is_dir(), "what was missing was remade");
}
