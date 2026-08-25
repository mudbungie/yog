//! The fixture every fan beat spreads over: a **real project repo** (an
//! integration branch with one commit, which is what a balls invocation path
//! is), the balls layout its attempts are placed under, and the ordinary
//! prepared start. Split from the beats at §12's budget on the seam between
//! *the world a fan runs in* and *what the fan does in it* — a fake would test
//! the fake, so there is exactly one world and it has one home.

use std::path::PathBuf;

use tempfile::{TempDir, tempdir};

use super::super::Obligation;
use crate::git_tree::tests::git::{git_out, run_git};
use crate::opslog::Origin;
use crate::start::Prepared;
use balls::layout::Xdg;

/// The fixture repo's integration branch.
pub(super) const MAIN: &str = "main";
/// The ball whose `work/<id>` ref every ball-obligation test targets.
pub(super) const BALL: &str = "bl-1f2a";

/// A project repo plus the balls layout its attempts are placed under.
pub(super) struct World {
    pub(super) dir: TempDir,
    pub(super) project: PathBuf,
    pub(super) xdg: Xdg,
}

impl World {
    pub(super) fn new() -> World {
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

    pub(super) fn obligation(ball: Option<&str>) -> Obligation {
        Obligation {
            project: "proj".to_owned(),
            ball: ball.map(str::to_owned),
        }
    }

    /// The commit a ref names right now.
    pub(super) fn tip(&self, refname: &str) -> String {
        git_out(&self.project, &["rev-parse", refname])
    }

    pub(super) fn branch_exists(&self, refname: &str) -> bool {
        crate::git_env::output(crate::git_env::git().arg("-C").arg(&self.project).args([
            "rev-parse",
            "--verify",
            "-q",
            refname,
        ]))
        .unwrap()
        .status
        .success()
    }
}

/// The ordinary prepared start a fan spreads.
pub(super) fn prepared(dir: &TempDir) -> Prepared {
    Prepared {
        workspace: "cobalt-gecko".to_owned(),
        binding: Some(dir.path().join("claim")),
        goal: "Ball bl-1f2a: do the thing".to_owned(),
        origin: Origin::Balls,
        lineage: None,
    }
}
