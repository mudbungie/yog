//! Config-branch fixture builders extending [`Fixture`] (§9.3 / §5.1 #17–#18
//! coverage): second config lineages, orphan branches, and agents forked off
//! chosen commits — the shapes the governing-config fold must resolve. Split
//! out of `fixture.rs` to keep both files under the 300-line source cap.
//!
//! [`Fixture`]: super::fixture::Fixture

use super::fixture::Fixture;
use super::git::run_git;

impl Fixture {
    /// Fork `agents/<id>` off `start` (a config ref, an agent ref, or an oid)
    /// with one dispatch commit, then drop the worktree — the minimal agent
    /// shape whose tip has advanced past its fork point.
    pub(crate) fn agent_off(&self, id: &str, start: &str) {
        self.spawn_committed("agents", id, Some(start), "goal.md", id, "dispatch");
    }

    /// Fork `config/<name>` off `start` with one marker commit — a second
    /// config lineage whose head sits nearer an agent tip than `config/default`.
    pub(crate) fn config_off(&self, name: &str, start: &str) {
        let marker = format!("{name}.marker");
        self.spawn_committed("config", name, Some(start), &marker, name, "config: edit");
    }

    /// Point `config/<name>` at an existing commit without advancing it — two
    /// config refs sharing one tip (the fold's equal-candidate arm).
    pub(crate) fn config_alias(&self, name: &str, start: &str) {
        run_git(&self.repo, &["branch", &format!("config/{name}"), start]);
    }

    /// An orphan `config/<name>` lineage sharing no history with the workspace
    /// — a `merge-base` miss the fold skips.
    pub(crate) fn orphan_config(&self, name: &str) {
        let marker = format!("{name}.marker");
        self.spawn_committed("config", name, None, &marker, name, "config: island");
    }

    /// An orphan `agents/<id>` branch — no config commit forks it, so the
    /// governing derivation declines (§2.2).
    pub(crate) fn orphan_agent(&self, id: &str) {
        self.spawn_committed("agents", id, None, "goal.md", id, "orphan");
    }

    /// Merge an unrelated `config/<name>` into `agents/<id>` — both config
    /// heads become incomparable ancestors of the tip (the ambiguity decline).
    pub(crate) fn cross_merge(&self, id: &str, config_name: &str) {
        let wt = self.path.join(format!(".merge-{id}"));
        let wt_str = wt.to_string_lossy().to_string();
        let branch = format!("agents/{id}");
        let other = format!("config/{config_name}");
        run_git(&self.repo, &["worktree", "add", "-q", &wt_str, &branch]);
        run_git(
            &wt,
            &[
                "merge",
                "-q",
                "--allow-unrelated-histories",
                "-m",
                "cross",
                &other,
            ],
        );
        run_git(&self.repo, &["worktree", "remove", &wt_str]);
    }

    /// Create `<prefix>/<name>` (`start` = fork point, or `None` for an orphan
    /// root), write one file, commit, and drop the worktree — the one git
    /// shape every builder above shares.
    fn spawn_committed(
        &self,
        prefix: &str,
        name: &str,
        start: Option<&str>,
        file: &str,
        body: &str,
        subject: &str,
    ) {
        let wt = self.path.join(format!(".build-{prefix}-{name}"));
        let wt_str = wt.to_string_lossy().to_string();
        let branch = format!("{prefix}/{name}");
        let mut add: Vec<&str> = vec!["worktree", "add", "-q"];
        match start {
            Some(s) => add.extend(["-b", branch.as_str(), wt_str.as_str(), s]),
            None => add.extend(["--orphan", "-b", branch.as_str(), wt_str.as_str()]),
        }
        run_git(&self.repo, &add);
        std::fs::write(wt.join(file), body).unwrap();
        run_git(&wt, &["add", file]);
        run_git(&wt, &["commit", "-q", "-m", subject]);
        run_git(&self.repo, &["worktree", "remove", &wt_str]);
    }
}
