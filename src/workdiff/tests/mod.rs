//! The work-diff's headless tests — **S11**'s whole rung, split by what each
//! half asserts: [`plan`] the pure derivation (which attempts, which target,
//! how numstat reads), [`read`] the git read against a real project repo, and
//! [`paint`] the tab under a windowless egui context.
//!
//! The fixture is a **project** repo — an ordinary working repo with an
//! integration branch and `work/<id>` branches on it, which is what a balls
//! invocation path is. It is not the workspace fixture: nothing here reads a
//! litany workspace.

mod candidates;
mod paint;
mod plan;
mod read;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use tempfile::{TempDir, tempdir};

use crate::app::Snapshot;
use crate::binding::{Workspace, WorkspaceKind};
use crate::git_tree::tests::git::run_git;
use crate::projects::balls::{Ball, Blocker};

/// The integration branch every fixture repo is founded on — `bl close`'s
/// target for a flat ball, and what `git symbolic-ref HEAD` names.
pub(crate) const MAIN: &str = "main";

/// A tempdir-backed project repo: a real git working repo, because the read
/// under test *is* the git CLI's answer and mocking it would test the mock.
pub(crate) struct Project {
    _dir: TempDir,
    pub(crate) path: PathBuf,
}

impl Project {
    /// A repo on [`MAIN`] with one commit.
    pub(crate) fn new() -> Self {
        let dir = tempdir().unwrap();
        let path = dir.path().to_path_buf();
        run_git(&path, &["init", "-q", "-b", MAIN]);
        run_git(&path, &["config", "user.email", "t@t.local"]);
        run_git(&path, &["config", "user.name", "Tester"]);
        run_git(&path, &["config", "commit.gpgsign", "false"]);
        let project = Self { _dir: dir, path };
        project.commit("README.md", "the project\n");
        project
    }

    /// Write `file` and commit it on the current branch.
    pub(crate) fn commit(&self, file: &str, body: &str) {
        let full = self.path.join(file);
        if let Some(parent) = full.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(&full, body).unwrap();
        run_git(&self.path, &["add", file]);
        run_git(&self.path, &["commit", "-q", "-m", &format!("add {file}")]);
    }

    /// Branch to `name` off the current HEAD and check it out.
    pub(crate) fn switch(&self, name: &str) {
        run_git(&self.path, &["checkout", "-q", "-b", name]);
    }

    /// Check out an existing branch.
    pub(crate) fn checkout(&self, name: &str) {
        run_git(&self.path, &["checkout", "-q", name]);
    }
}

/// A live ball, claimed by `claimant`, optionally a child of `parent`.
pub(crate) fn ball(id: &str, claimant: Option<&str>, parent: Option<&str>) -> Ball {
    Ball {
        id: id.to_owned(),
        title: format!("ball {id}"),
        body: String::new(),
        claimant: claimant.map(str::to_owned),
        blockers: Vec::new(),
        parent: parent.map(str::to_owned),
        priority: 0,
        tags: Vec::new(),
        created: None,
        updated: None,
        root_commit: None,
    }
}

/// The close-gate edge that makes a child ball deliver onto its parent's
/// branch: `{child, on: close}` carried by the **parent**.
pub(super) fn close_gate(mut parent: Ball, child: &str) -> Ball {
    parent.blockers.push(Blocker {
        id: child.to_owned(),
        on: "close".to_owned(),
    });
    parent
}

/// The balls layout the candidate rows resolve attempt paths under — pointed
/// at a throwaway root, because a test with no fire rows reads nothing off it.
pub(crate) fn xdg(root: &Path) -> balls::layout::Xdg {
    balls::layout::Xdg::with(
        &root.join("home"),
        None,
        Some(&root.join("state").to_string_lossy()),
    )
}

/// [`crate::workdiff::read`] with an empty trail: the claim rows alone, which
/// is every read that predates the fan's candidate rows (bl-c2bd).
pub(super) fn read0(snap: &Snapshot, ws: &Path) -> Vec<crate::workdiff::Attempt> {
    let dir = tempfile::tempdir().unwrap();
    crate::workdiff::read(snap, ws, &[], &xdg(dir.path()))
}

/// A snapshot carrying one named workspace and one project's live balls —
/// the two facts [`crate::workdiff::read`] joins.
pub(crate) fn snap(ws: &Path, name: &str, project: &Path, balls: Vec<Ball>) -> Snapshot {
    let mut snap = Snapshot::empty(0);
    snap.workspaces = vec![Workspace {
        path: ws.to_path_buf(),
        kind: WorkspaceKind::Named {
            name: name.to_owned(),
        },
    }];
    // The enumerated project set, beside the ball map it keys — the production
    // shape (`refresh_balls` fills `projects` from every clone and
    // `balls_by_project` from the visible ones), and what makes an
    // [`Attempt`](crate::workdiff::Attempt)'s project NAME resolve back to a
    // repository a `git` read can run in (REMOTE §8, bl-ccf7).
    snap.projects = vec![project.to_path_buf()];
    snap.balls_by_project = HashMap::from([(project.to_path_buf(), balls)]);
    snap
}
