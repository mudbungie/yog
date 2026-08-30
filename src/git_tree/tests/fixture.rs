//! Shared test fixture: **the workspace itself** — a tempdir-backed ARCH §2.2
//! layout (bare `repo.git` on `config/default`, its orphan root commit) and the
//! one helper that advances that config lineage.
//!
//! Tests hit a real git binary rather than mocking; fixtures are cheap
//! to spin up and the renderer's contract is explicitly with the CLI,
//! so mocking would mean testing our mock.
//!
//! Everything keyed to an **agent id** — the `agents/<id>` branch, its `name`
//! blob, its `refs/litany/*` marks and its descent forks — is
//! [`super::agent_fixture`]; the plain files git never sees (step records,
//! inbox deposits) are [`super::disk_fixture`]; the second config lineages are
//! [`super::config_fixture`]. Four files, one `Fixture`, split on the seams
//! ARCH itself draws.

use super::git::run_git;
use std::fs;
use std::path::PathBuf;
use tempfile::{TempDir, tempdir};

pub(crate) struct Fixture {
    _dir: TempDir,
    /// Workspace root — the dir holding `repo.git/`, `steps/`,
    /// `inbox/`, and the `agents/` worktrees. This is what
    /// `GitTree::from_repo` is passed in production (ARCH §2.2).
    pub(crate) path: PathBuf,
    /// The bare workspace repository (`<workspace>/repo.git`). All
    /// ref-level git commands run against it, mirroring the harness.
    pub(super) repo: PathBuf,
}

impl Fixture {
    pub(crate) fn new() -> Self {
        let dir = tempdir().unwrap();
        let path = dir.path().to_path_buf();
        let repo = path.join("repo.git");
        fs::create_dir_all(&repo).unwrap();
        run_git(&repo, &["init", "-q", "--bare", "-b", "config/default"]);
        run_git(&repo, &["config", "user.email", "t@t.local"]);
        run_git(&repo, &["config", "user.name", "Tester"]);
        run_git(&repo, &["config", "commit.gpgsign", "false"]);
        let fx = Self {
            _dir: dir,
            path,
            repo,
        };
        // The first config commit (orphan root, §2.2).
        let author = fx.path.join(".author");
        let author_str = author.to_string_lossy().to_string();
        run_git(
            &fx.repo,
            &[
                "worktree",
                "add",
                "-q",
                "--orphan",
                "-b",
                "config/default",
                author_str.as_str(),
            ],
        );
        fs::write(author.join("version"), "1\n").unwrap();
        run_git(&author, &["add", "version"]);
        run_git(
            &author,
            &["commit", "-q", "-m", "config: init [config/default]"],
        );
        run_git(&fx.repo, &["worktree", "remove", author_str.as_str()]);
        fx
    }

    /// Advance the config lineage with one edit (§2.3 branch
    /// advancement: only user config edits move a config branch).
    pub(crate) fn commit_other(&self, file: &str, body: &str) {
        let author = self.path.join(".amend");
        let author_str = author.to_string_lossy().to_string();
        run_git(
            &self.repo,
            &[
                "worktree",
                "add",
                "-q",
                author_str.as_str(),
                "config/default",
            ],
        );
        fs::write(author.join(file), body).unwrap();
        run_git(&author, &["add", file]);
        run_git(&author, &["commit", "-q", "-m", &format!("add {file}")]);
        run_git(&self.repo, &["worktree", "remove", author_str.as_str()]);
    }
}
