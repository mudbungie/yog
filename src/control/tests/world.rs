//! The world one consult is asked against: a workspace, a state root carrying
//! `ops.jsonl`, the balls state root the delivery formula mirrors, and the
//! `capability.yaml` a workspace commits onto `config/default`. Split from the
//! beats at §12's budget on the seam between *the world a test builds* and
//! *what the control decides in it* — every file in this corpus builds the
//! same one, so it has exactly one home.

use crate::control::Consult;
use crate::control::policy::{CAPABILITY_YAML, Policy};
use crate::opslog::{OpEntry, Origin, append};
use serde_json::json;
use std::path::PathBuf;
use tempfile::{TempDir, tempdir};

/// A world on disk: a workspace named like a yog workspace, a state root
/// carrying `ops.jsonl`, and a balls state root the delivery formula mirrors.
pub(super) struct World {
    pub(super) dir: TempDir,
}

impl World {
    pub(super) fn new() -> World {
        World {
            dir: tempdir().unwrap(),
        }
    }

    pub(super) fn workspace(&self) -> PathBuf {
        self.dir.path().join("workspaces").join("cobalt-gecko")
    }

    pub(super) fn state(&self) -> PathBuf {
        self.dir.path().join("state").join("yog")
    }

    pub(super) fn balls(&self) -> PathBuf {
        self.dir.path().join("state").join("balls")
    }

    pub(super) fn consult(&self) -> Consult {
        Consult {
            workspace: self.workspace(),
            balls: balls::layout::Xdg::with(
                &self.dir.path().join("home"),
                None,
                Some(&self.dir.path().join("state").to_string_lossy()),
            ),
            state_root: self.state(),
            home: self.dir.path().join("home"),
            cwd: None,
            policy: Policy::default(),
        }
    }

    /// Log a `bl claim` row exactly as the start flow writes it.
    pub(super) fn claim(&self, project: &str, id: &str, claimant: &str) {
        self.row(
            &["bl", "claim", id, "--as", claimant],
            project,
            Origin::Balls,
        );
    }

    /// Log one capability-answer row (what bl-765d's boundary action writes).
    pub(super) fn answer(&self, words: &[&str]) {
        self.row(words, "", Origin::World);
    }

    /// Commit `capability.yaml` onto `config/default` — the workspace's
    /// standing policy, at the tip the control reads it from.
    pub(super) fn policy(&self, text: &str) {
        let repo = self.workspace().join("repo.git");
        std::fs::create_dir_all(&repo).unwrap();
        let git = |args: &[&str]| {
            let out = crate::git_env::output(
                crate::git_env::git()
                    .arg("--git-dir")
                    .arg(&repo)
                    .args(args)
                    .env("GIT_AUTHOR_DATE", "2026-08-04T00:00:00Z")
                    .env("GIT_COMMITTER_DATE", "2026-08-04T00:00:00Z")
                    .env("GIT_AUTHOR_NAME", "t")
                    .env("GIT_AUTHOR_EMAIL", "t@t")
                    .env("GIT_COMMITTER_NAME", "t")
                    .env("GIT_COMMITTER_EMAIL", "t@t")
                    .env("GIT_CONFIG_GLOBAL", "/dev/null")
                    .env("GIT_CONFIG_SYSTEM", "/dev/null"),
            )
            .unwrap();
            assert!(out.status.success(), "git {args:?}");
            String::from_utf8_lossy(&out.stdout).trim().to_owned()
        };
        git(&["init", "--bare", "-q"]);
        let staged = self.dir.path().join(CAPABILITY_YAML);
        std::fs::write(&staged, text).unwrap();
        let blob = git(&["hash-object", "-w", "--", &staged.to_string_lossy()]);
        git(&[
            "update-index",
            "--add",
            "--cacheinfo",
            "100644",
            &blob,
            CAPABILITY_YAML,
        ]);
        let tree = git(&["write-tree"]);
        let commit = git(&["commit-tree", &tree, "-m", "policy"]);
        git(&["update-ref", "refs/heads/config/default", &commit]);
    }

    fn row(&self, words: &[&str], cwd: &str, origin: Origin) {
        append(
            &self.state(),
            &OpEntry {
                ts: "TS".to_owned(),
                argv: words.iter().map(|s| (*s).to_owned()).collect(),
                cwd: cwd.to_owned(),
                exit: 0,
                stdout: String::new(),
                stderr: String::new(),
                origin,
            },
        )
        .unwrap();
    }
}

/// A stdout that refuses every write — the closed-pipe half of failing closed.
pub(super) struct Closed;

impl std::io::Write for Closed {
    fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
        Err(std::io::Error::other("closed"))
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// One request as litany's seam serializes it.
pub(super) fn request(name: &str, input: serde_json::Value) -> String {
    json!({
        "id": "toolu_01",
        "name": name,
        "input": input,
        "role": "worker",
        "agent_id": "amber",
    })
    .to_string()
}
