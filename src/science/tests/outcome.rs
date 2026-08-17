//! The four outcome arms over their three git facts (§3.9) — against a real
//! project repo and a real fan, because every one of them is git's answer and a
//! mocked repo would test the mock.

use std::path::Path;

use super::{BALL, CONV, claimed_project, layout, snap, trail};
use crate::git_tree::tests::git::run_git;
use crate::science::{Attempt, Outcome, project};
use crate::workdiff::{Change, tests::Project};

/// A fan of `n` candidates over the claimed ball, and the trail that binds them.
struct Fan {
    _dir: tempfile::TempDir,
    project: Project,
    xdg: balls::layout::Xdg,
    balls_root: std::path::PathBuf,
    ws: std::path::PathBuf,
    obligation: crate::fan::Obligation,
    candidates: Vec<crate::fan::Candidate>,
}

impl Fan {
    fn open(n: usize) -> Fan {
        let project = claimed_project();
        let dir = tempfile::tempdir().unwrap();
        let (xdg, balls_root) = layout(dir.path());
        let ws = dir.path().join("ws");
        std::fs::create_dir_all(&ws).unwrap();
        let obligation = crate::fan::Obligation {
            project: "proj".to_owned(),
            ball: Some(BALL.to_owned()),
        };
        let candidates = crate::fan::open(&obligation, &project.path, &xdg, n).unwrap();
        Fan {
            _dir: dir,
            project,
            xdg,
            balls_root,
            ws,
            obligation,
            candidates,
        }
    }

    /// One commit of one file in a candidate's own worktree.
    fn work(&self, i: usize, file: &str) {
        let worktree = &self.candidates[i].worktree;
        std::fs::write(worktree.join(file), "fn f() {}\n").unwrap();
        run_git(worktree, &["add", file]);
        run_git(worktree, &["config", "user.email", "t@t.local"]);
        run_git(worktree, &["config", "user.name", "Tester"]);
        run_git(worktree, &["config", "commit.gpgsign", "false"]);
        run_git(worktree, &["commit", "-q", "-m", "candidate work"]);
    }

    fn rows(&self) -> Vec<Attempt> {
        let bindings: Vec<(&str, &Path)> = self
            .candidates
            .iter()
            .map(|c| (CONV, c.worktree.as_path()))
            .collect();
        let entries = trail(&self.ws, &self.project.path, &bindings);
        let snap = snap(&self.ws, &self.project.path, vec![], vec![]);
        project(&snap, &self.ws, &entries, &self.xdg, &self.balls_root)
    }

    fn deliver(&self, i: usize) -> Option<String> {
        crate::fan::deliver(
            &self.obligation,
            &self.project.path,
            &self.xdg,
            &self.candidates[i].handle,
            "take it",
        )
        .unwrap()
        .commit
    }
}

/// Nothing delivered anywhere: every candidate is pending, and so is the claim.
#[test]
fn undelivered_and_unopposed_is_pending() {
    let fan = Fan::open(2);
    fan.work(0, "won.rs");
    for row in fan.rows() {
        assert_eq!(row.outcome, Outcome::Pending, "{row:?}");
    }
}

/// One candidate delivers: it reads accepted at the commit the target's history
/// records, its sibling reads rejected *by that handle*, and the claim attempt —
/// a different target, so not a sibling — stays pending.
#[test]
fn a_delivery_accepts_one_and_rejects_its_sibling() {
    let fan = Fan::open(2);
    fan.work(0, "won.rs");
    let commit = fan.deliver(0).expect("a worked candidate lands a commit");
    let rows = fan.rows();
    assert_eq!(
        rows[0].outcome,
        Outcome::Pending,
        "the claim: {:?}",
        rows[0]
    );
    assert_eq!(rows[1].outcome, Outcome::Accepted { commit });
    assert_eq!(
        rows[2].outcome,
        Outcome::Rejected {
            by: Some(fan.candidates[0].handle.clone())
        }
    );
}

/// The loser reworks — it incorporates the advanced target in its own worktree —
/// and reads reworked rather than rejected: balls' delivery would no longer
/// refuse it as stale, which is the whole content of the word.
#[test]
fn incorporating_the_advanced_target_reads_as_reworked() {
    let fan = Fan::open(2);
    fan.work(0, "won.rs");
    fan.work(1, "lost.rs");
    fan.deliver(0).expect("the first candidate lands");
    let stale = &fan.candidates[1].worktree;
    assert!(matches!(
        fan.rows()[2].outcome,
        Outcome::Rejected { by: Some(_) }
    ));
    run_git(
        stale,
        &[
            "merge",
            "--no-edit",
            "-q",
            &balls::delivery_path::work_branch(BALL),
        ],
    );
    assert_eq!(fan.rows()[2].outcome, Outcome::Reworked);
}

/// A discarded attempt is rejected with nobody named: its source ref is gone,
/// which the diff row already states, and there was no sibling to lose to.
#[test]
fn a_discarded_attempt_is_rejected_by_nobody() {
    let fan = Fan::open(1);
    crate::fan::discard(
        &fan.obligation,
        &fan.project.path,
        &fan.xdg,
        &fan.candidates[0].handle,
    )
    .unwrap();
    let rows = fan.rows();
    assert!(
        matches!(rows[1].diff.change, Change::Absent { .. }),
        "{:?}",
        rows[1]
    );
    assert_eq!(rows[1].outcome, Outcome::Rejected { by: None });
}

/// The claim attempt's own acceptance is the ball's delivery onto the branch it
/// closes into — the same tag-scan, one level up, so N = 1 is not the one
/// attempt whose outcome cannot be read.
#[test]
fn the_claim_attempt_reads_its_own_delivery() {
    let fan = Fan::open(1);
    // A squash onto the integration branch tagged with the ball, exactly as
    // `bl close` mints it.
    fan.project.commit("closed.rs", "fn closed() {}\n");
    run_git(
        &fan.project.path,
        &[
            "commit",
            "--amend",
            "-q",
            "-m",
            &balls::delivery_path::subject("ship it", BALL),
        ],
    );
    let rows = fan.rows();
    let Outcome::Accepted { commit } = &rows[0].outcome else {
        panic!("the claim reads accepted: {:?}", rows[0]);
    };
    assert_eq!(Some(commit.clone()), rows[0].diff.delivered);
}
