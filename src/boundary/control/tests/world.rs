//! The world every beat in this directory answers a park inside: a bare
//! workspace repo the hold mark and the capability policy are written into, and
//! a state root the control's own row lands in.

use crate::boundary::dispatch::Deps;
use crate::boundary::tests::snapshot;
use crate::cli_outbound::Cli;
use crate::control::policy::CAPABILITY_YAML;
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::{TempDir, tempdir};

/// The conversation every beat here addresses. **Id-shaped** (ARCH §2.3's
/// compact stamp) because the §8.5 chokepoint resolves the conversation a
/// gesture names (bl-49bc), and an id reads as one on its own rather than
/// through an enumeration.
pub(super) const AGENT: &str = "20260101T000000Z-a1";

/// A world with a bare workspace repo the mark can be written into, and a
/// state root the row lands in.
pub(super) struct World {
    pub(super) dir: TempDir,
}

impl World {
    pub(super) fn new() -> World {
        World {
            dir: tempdir().expect("tempdir"),
        }
    }

    pub(super) fn workspace(&self) -> PathBuf {
        self.dir.path().join("names").join("alba")
    }

    pub(super) fn state(&self) -> PathBuf {
        self.dir.path().join("state")
    }

    pub(super) fn deps(&self) -> Deps {
        Deps {
            // `true` exists on every platform the suite runs on and exits 0 —
            // enough to prove the detached launch happened without driving a
            // real conversation.
            litany: Cli::new("/usr/bin/true"),
            bl: Cli::new("/no/such/bl"),
            state_root: self.state(),
            home: self.dir.path().join("home"),
            yog_data_root: self.dir.path().join("data"),
            balls_state_root: self.dir.path().join("balls"),
            yog_binary: PathBuf::from("/no/such/yog"),
            world: crate::xdg::Env::from_env(),
            snapshot: Arc::new(snapshot(&self.workspace(), "alba", Vec::new(), Vec::new())),
            caller: crate::boundary::dispatch::Caller::default(),
        }
    }

    /// The fixture's own `git`, carrying **its own identity** (bl-e492). A
    /// bare `commit-tree` takes the author from the machine's git config, and a
    /// CI runner has none — it wrote nothing, [`World::policy`]'s `update-ref`
    /// then got an empty oid, the policy never landed, and the confinement
    /// tests read a workspace that declares nothing. The suite must not ask the
    /// host who it is; every other fixture in the tree already says so itself
    /// (`test_support::workspace`, `control::tests`).
    fn git(&self, args: &[&str]) -> std::process::Output {
        crate::git_env::output(
            crate::git_env::git()
                .args(["-c", "user.email=t@t.local", "-c", "user.name=T"])
                .arg("--git-dir")
                .arg(self.workspace().join("repo.git"))
                .args(args),
        )
        .expect("git runs")
    }

    pub(super) fn repo(&self) {
        std::fs::create_dir_all(self.workspace().join("repo.git")).unwrap();
        self.git(&["init", "--bare", "-q"]);
    }

    /// Park `agent` on `tool_use`, exactly as litany's seam does.
    pub(super) fn park(&self, agent: &str, tool_use: &str) {
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
            &format!("refs/litany/held/{agent}"),
            oid.as_str(),
        ]);
    }

    /// Commit `capability.yaml` onto `config/default`.
    pub(super) fn policy(&self, text: &str) {
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
        let written = self.git(&["commit-tree", &tree, "-m", "policy"]);
        let commit = String::from_utf8_lossy(&written.stdout).trim().to_owned();
        // A fixture that half-worked is worse than one that failed: an empty
        // oid here used to sail into `update-ref`, leave `config/default`
        // unborn, and hand the tests a workspace that declares no policy — so
        // the gate answered `Ok(())` and the assertion blamed the gate.
        let why = String::from_utf8_lossy(&written.stderr).into_owned();
        assert!(!commit.is_empty(), "commit-tree wrote nothing: {why}");
        self.git(&["update-ref", "refs/heads/config/default", &commit]);
    }
}
