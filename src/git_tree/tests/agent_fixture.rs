//! The [`Fixture`] half keyed to **an agent id**: the `agents/<id>` branch a
//! dispatch founds, the `name` blob it wears (lernie ARCH §2.3, DESIGN §3.3),
//! the `refs/lernie/*` marks hung beside it (§6, §8.6), and the hyphenated-
//! descent forks the §7.1 tree render is drawn from.
//!
//! Split from [`super::fixture`] at §12's pre-split band on the seam the
//! workspace layout already draws: that module founds the **workspace** —
//! `repo.git` and its `config/default` lineage (ARCH §2.2) — and this one
//! everything a conversation adds inside it. Same discipline as
//! [`super::disk_fixture`] (the files git never sees) and
//! [`super::config_fixture`] (the second config lineages).
//!
//! [`Fixture`]: super::fixture::Fixture

use super::fixture::Fixture;
use super::git::run_git;
use std::fs;

impl Fixture {
    /// Build an agent branch `agents/<conv_id>` with its worktree at
    /// `agents/<conv_id>/`: a dispatch commit plus a compaction merge
    /// (the one merge, §2.6); the user-message step record lands on
    /// disk at `<workspace>/steps/<conv-id>/001/request.json`, outside
    /// every worktree (ARCH §2.3).
    pub(crate) fn build_agent(&self, conv_id: &str, user_message: &str) {
        let wt = self.path.join("agents").join(conv_id);
        let wt_str = wt.to_string_lossy().to_string();
        let branch = format!("agents/{conv_id}");
        run_git(
            &self.repo,
            &[
                "worktree",
                "add",
                "-q",
                "-b",
                branch.as_str(),
                wt_str.as_str(),
                "config/default",
            ],
        );
        // Dispatch commit (ARCH §2.3 step 2): goal.md + soul.md; the
        // control files leave the tree (only `version` here).
        fs::write(wt.join("goal.md"), user_message).unwrap();
        fs::write(wt.join("soul.md"), "you are a tester\n").unwrap();
        run_git(&wt, &["rm", "-q", "--ignore-unmatch", "version"]);
        run_git(&wt, &["add", "goal.md", "soul.md"]);
        run_git(
            &wt,
            &["commit", "-q", "-m", &format!("dispatch [{conv_id}]")],
        );
        // Compactor child: hyphenated descent off the agent branch
        // (ARCH §2.3); one summary commit, merged --no-ff back into the
        // agent branch — the compaction merge (§2.6).
        let cmp_id = format!("{conv_id}-c");
        let cmp_branch = format!("agents/{cmp_id}");
        let cmp_wt = self.path.join("agents").join(&cmp_id);
        let cmp_str = cmp_wt.to_string_lossy().to_string();
        run_git(
            &self.repo,
            &[
                "worktree",
                "add",
                "-q",
                "-b",
                cmp_branch.as_str(),
                cmp_str.as_str(),
                branch.as_str(),
            ],
        );
        fs::create_dir_all(cmp_wt.join("summary")).unwrap();
        fs::write(
            cmp_wt.join("summary/001.md"),
            format!("conversation {conv_id}: pong\n"),
        )
        .unwrap();
        run_git(&cmp_wt, &["add", "summary/001.md"]);
        run_git(
            &cmp_wt,
            &[
                "commit",
                "-q",
                "-m",
                &format!("compaction: terminal summary [{conv_id}]"),
            ],
        );
        run_git(&wt, &["merge", "--no-ff", "-q", "--no-edit", &cmp_branch]);
        run_git(&self.repo, &["worktree", "remove", cmp_str.as_str()]);
        run_git(&self.repo, &["branch", "-q", "-D", &cmp_branch]);
        // Step record on disk, outside every worktree (ARCH §2.3).
        self.write_step_record(conv_id, user_message);
    }

    /// A **bare** agent branch: `agents/<conv_id>` pointed at the config tip,
    /// and nothing else — no worktree, no dispatch commit, no compaction merge
    /// and no step record. The ref namespace *is* the registry (ARCH §2.3), so
    /// the enumeration answers with this conversation exactly as it does with a
    /// built one; its row falls to the DESIGN §3.3 display ladder's last rung,
    /// the id, which is what a beat names it by.
    ///
    /// One `git branch` where [`build_agent`](Self::build_agent) is a dozen
    /// forks, two worktrees and a merge. That difference is what makes a
    /// fixture of *many* conversations affordable at all (yog bl-86a5): the
    /// column-budget beats need a list longer than any window, and they need
    /// rows rather than transcripts.
    pub(crate) fn build_bare_agent(&self, conv_id: &str) {
        run_git(
            &self.repo,
            &["branch", &format!("agents/{conv_id}"), "config/default"],
        );
    }

    /// Commit a lernie-0.0.4 `name` blob on an existing agent's branch — the
    /// name fact's one home (`git show agents/<id>:name`, DESIGN §3.3 as ruled
    /// by bl-50f3). Empty `name` mirrors lernie's unnamed write. Re-adds the
    /// branch's worktree when it is gone (a child's is torn down after
    /// [`build_child`](Self::build_child)).
    pub(crate) fn name_agent(&self, conv_id: &str, name: &str) {
        let wt = self.path.join("agents").join(conv_id);
        if !wt.exists() {
            let wt_str = wt.to_string_lossy().to_string();
            run_git(
                &self.repo,
                &[
                    "worktree",
                    "add",
                    "-q",
                    &wt_str,
                    &format!("agents/{conv_id}"),
                ],
            );
        }
        fs::write(wt.join("name"), format!("{name}\n")).unwrap();
        run_git(&wt, &["add", "name"]);
        run_git(&wt, &["commit", "-q", "--allow-empty", "-m", "settle name"]);
    }

    /// Point a `refs/lernie/<kind>/<agent-id>` mark ref at `config/default`
    /// (any commit is fine — the frontend reads existence, not content).
    /// Mirrors `transfer::decline` (§2.6) and `budget::mark_exhausted`
    /// (§6), which key the mark off the raw agent id.
    pub(crate) fn mark_ref(&self, refname: &str) {
        run_git(&self.repo, &["update-ref", refname, "config/default"]);
    }

    /// Point `refs/lernie/held/<agent_id>` at a **blob** carrying `value` —
    /// the one valued mark yog reads (lernie ARCH §3.3). Unlike [`mark_ref`],
    /// which only has to exist, this mark's content *is* what the operator
    /// reads, so the fixture writes the blob the seam would write.
    ///
    /// [`mark_ref`]: Fixture::mark_ref
    pub(crate) fn hold_ref(&self, agent_id: &str, value: &str) {
        let staged = self.path.join(".hold-mark");
        fs::write(&staged, value).unwrap();
        let oid = super::git::git_out(
            &self.repo,
            &["hash-object", "-w", "--", &staged.to_string_lossy()],
        );
        run_git(
            &self.repo,
            &["update-ref", &format!("refs/lernie/held/{agent_id}"), &oid],
        );
    }

    /// Build a child agent branch `agents/<child_id>` off `parent_id`'s
    /// branch tip — a hyphenated-descent fork (§2.3). Used to exercise the
    /// descent-tree render (§7.1). The child carries one dispatch commit.
    ///
    /// The commit's file is keyed by the child's own id, which is what makes
    /// **a child of a child** buildable: forked off its parent's tip it already
    /// carries the parent's dispatch file, so a fixed name and fixed content
    /// left git with nothing to commit and the fork failed one generation down.
    pub(crate) fn build_child(&self, parent_id: &str, child_id: &str) {
        let wt = self.path.join("agents").join(child_id);
        let wt_str = wt.to_string_lossy().to_string();
        let branch = format!("agents/{child_id}");
        run_git(
            &self.repo,
            &[
                "worktree",
                "add",
                "-q",
                "-b",
                branch.as_str(),
                wt_str.as_str(),
                &format!("agents/{parent_id}"),
            ],
        );
        let file = format!("{child_id}.md");
        fs::write(wt.join(&file), "child work\n").unwrap();
        run_git(&wt, &["add", file.as_str()]);
        run_git(
            &wt,
            &["commit", "-q", "-m", &format!("dispatch [{child_id}]")],
        );
        run_git(&self.repo, &["worktree", "remove", wt_str.as_str()]);
    }
}
