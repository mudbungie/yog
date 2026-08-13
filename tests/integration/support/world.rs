//! Multi-agent workspace fixtures for the S4/S6/S7 story rows (STORIES "Test
//! harness"; the Z10–Z14 fake halves) — split out of [`super`] per the 300-line
//! cap.
//!
//! [`super::build_workspace`] lays exactly one bare agent, which is all the S1
//! rows need. The board, triage and forensic rungs need *several* conversations
//! in one workspace, each with its own goal stamp, its own `refs/lernie/*`
//! attention marks and (for S7) its own `messages/`, `steps/` and `inbox/`
//! payloads on disk. This module is that builder: plain data in, a real git
//! workspace out, so every assertion runs against
//! [`GitTree::from_repo`](yog::git_tree::GitTree::from_repo)'s own derivation
//! rather than a hand-built `Agent`.

#![allow(dead_code)]
#![allow(clippy::unwrap_used)]

use super::payload::{hash_object, set_mtime, write_step};
use std::fs;
use std::path::Path;

/// One agent branch to lay in a fixture workspace: its id (the §2.3 descent
/// grammar decides root-vs-child), its `goal.md` body — a `Ball <id>: <title>`
/// first line is the §3.3 stamp the conversation badge derives from — and the
/// `refs/lernie/<mark>/<id>` marks to point at its tip.
pub struct AgentFixture {
    pub id: String,
    pub goal: String,
    pub marks: Vec<String>,
    /// The dispatch commit's author/committer date, which with no `messages/`
    /// or `response.json` beside it *is* the agent's `last_action_unix` — the
    /// §11 list's one sort key (bl-cad5). `None` leaves it at "now".
    pub at: Option<i64>,
    /// The latest step's settled `response.json` framing (§4.4): `Some(true)`
    /// writes a **complete** tail (⇒ `Quiescent`), `Some(false)` a **failed**
    /// one (⇒ `Stopped`). `None` writes no step at all, which is also
    /// `Stopped` — "no step has run" and "the step failed" are one state.
    pub complete: Option<bool>,
    /// The `refs/lernie/held/<id>` **blob** body (§8.6): unlike every other
    /// mark, a hold carries a value — the parked `tool_use`, its tool and the
    /// control's reason — so the ref points at a blob, not at the branch.
    pub held: Option<String>,
}

impl AgentFixture {
    /// An agent with `goal` and no marks.
    pub fn new(id: &str, goal: &str) -> Self {
        Self {
            id: id.to_owned(),
            goal: goal.to_owned(),
            marks: Vec::new(),
            at: None,
            complete: None,
            held: None,
        }
    }

    /// An agent whose `goal.md` stamps ball `ball` (§3.3) — the fact the §11
    /// conversation badge reads.
    pub fn stamped(id: &str, ball: &str, title: &str) -> Self {
        Self::new(id, &format!("Ball {ball}: {title}\n"))
    }

    /// Point `refs/lernie/<mark>/<id>` at this agent's tip — `notify`,
    /// `budget-exhausted`, `conflicted` or `abandoned` (§6's watermark
    /// evidence).
    #[must_use]
    pub fn mark(mut self, mark: &str) -> Self {
        self.marks.push(mark.to_owned());
        self
    }

    /// Date the dispatch commit at `unix` — the agent's last action (§11).
    #[must_use]
    pub fn at(mut self, unix: i64) -> Self {
        self.at = Some(unix);
        self
    }

    /// Park a tool invocation at the capability boundary (§8.6): `tool` named,
    /// `reason` stated, keyed on `tool_use_id` — the three-field object lernie
    /// writes and `control::hold::parse` accepts.
    #[must_use]
    pub fn held(mut self, tool_use_id: &str, tool: &str, reason: &str) -> Self {
        self.held = Some(format!(
            r#"{{"tool_use_id":"{tool_use_id}","tool":"{tool}","reason":"{reason}"}}"#
        ));
        self
    }

    /// Give the agent a settled latest step: `true` a complete tail
    /// (`Quiescent`), `false` a failed one (`Stopped`).
    #[must_use]
    pub fn settled(mut self, complete: bool) -> Self {
        self.complete = Some(complete);
        self
    }
}

/// A settled `response.json` tail (§4.4): a `finish`-then-`end` segment reads
/// **complete**, an `error`-then-`end` one reads **failed**. Only the last
/// segment decides, so one segment is the whole fixture.
pub fn response_tail(complete: bool) -> String {
    let outcome = if complete {
        r#"{"type":"finish"}"#
    } else {
        r#"{"type":"error","message":"the model refused"}"#
    };
    format!("{outcome}\n{{\"type\":\"end\"}}\n")
}

/// Build a workspace at `ws` holding one agent branch per fixture (ARCH §2.2
/// layout): a bare `repo.git` rooted at a `config/default` commit, then an
/// `agents/<id>` branch and worktree per fixture carrying its `goal.md`, with
/// its marks pointed at that branch's tip.
pub fn build_agents(ws: &Path, agents: &[AgentFixture]) {
    let repo = ws.join("repo.git");
    fs::create_dir_all(&repo).unwrap();
    super::run_git(&repo, &["init", "-q", "--bare", "-b", "config/default"]);
    super::run_git(&repo, &["config", "user.email", "t@t.local"]);
    super::run_git(&repo, &["config", "user.name", "Tester"]);
    super::run_git(&repo, &["config", "commit.gpgsign", "false"]);
    author_config_root(ws, &repo);
    add_agents(ws, agents);
}

/// Lay more agent branches into a workspace [`build_agents`] already founded —
/// the config root is committed once, so a later wave adds branches only (the
/// S7 descent fixtures grow a conversation after the first derivation).
pub fn add_agents(ws: &Path, agents: &[AgentFixture]) {
    let repo = ws.join("repo.git");
    for agent in agents {
        lay_agent(ws, &repo, agent);
    }
}

/// The orphan `config/default` root commit (§2.2), authored in a throwaway
/// worktree — the lineage every agent branch forks off.
fn author_config_root(ws: &Path, repo: &Path) {
    let author = ws.join(".author");
    let author_str = author.to_string_lossy().to_string();
    super::run_git(
        repo,
        &[
            "worktree",
            "add",
            "-q",
            "--orphan",
            "-b",
            "config/default",
            &author_str,
        ],
    );
    fs::write(author.join("version"), "1\n").unwrap();
    super::run_git(&author, &["add", "version"]);
    super::run_git(&author, &["commit", "-q", "-m", "config: init"]);
    super::run_git(repo, &["worktree", "remove", &author_str]);
}

/// One `agents/<id>` branch + worktree carrying `goal.md`, then its marks.
fn lay_agent(ws: &Path, repo: &Path, agent: &AgentFixture) {
    let wt = ws.join("agents").join(&agent.id);
    let wt_str = wt.to_string_lossy().to_string();
    let branch = format!("agents/{}", agent.id);
    super::run_git(
        repo,
        &[
            "worktree",
            "add",
            "-q",
            "-b",
            &branch,
            &wt_str,
            "config/default",
        ],
    );
    fs::write(wt.join("goal.md"), &agent.goal).unwrap();
    super::run_git(&wt, &["add", "goal.md"]);
    commit_at(&wt, &format!("dispatch [{}]", agent.id), agent.at);
    for mark in &agent.marks {
        let refname = format!("refs/lernie/{mark}/{}", agent.id);
        super::run_git(repo, &["update-ref", &refname, &branch]);
    }
    if let Some(blob) = &agent.held {
        let oid = hash_object(repo, blob);
        let refname = format!("refs/lernie/held/{}", agent.id);
        super::run_git(repo, &["update-ref", &refname, &oid]);
    }
    if let Some(complete) = agent.complete {
        write_step(
            ws,
            &agent.id,
            "000",
            "response.json",
            &response_tail(complete),
        );
        // `last_action_unix` is the NEWEST of the tip timestamp, the newest
        // `messages/` mtime and this file's mtime (the live streaming tail), so
        // a step written "now" would drown every dated commit and collapse the
        // §11 order into an id tiebreak. Date the file with the commit.
        if let Some(unix) = agent.at {
            set_mtime(
                &ws.join("steps").join(&agent.id).join("000/response.json"),
                unix,
            );
        }
    }
}

/// Commit the staged tree, optionally dated at `when` (unix seconds) so the
/// agent's `last_action_unix` is the fixture's choice rather than the clock.
fn commit_at(wt: &Path, message: &str, when: Option<i64>) {
    // Scrubbed and config-free for the same reasons [`super::run_git`] is.
    let mut cmd = yog::git_env::git();
    cmd.env("GIT_CONFIG_GLOBAL", "/dev/null");
    cmd.env("GIT_CONFIG_SYSTEM", "/dev/null");
    if let Some(unix) = when {
        // git's raw format: `@<unix> <tz>` — a bare integer is refused.
        let stamp = format!("@{unix} +0000");
        cmd.env("GIT_AUTHOR_DATE", &stamp);
        cmd.env("GIT_COMMITTER_DATE", &stamp);
    }
    let status = cmd
        .arg("-C")
        .arg(wt)
        .args(["commit", "-q", "-m", message])
        .status()
        .unwrap();
    assert!(status.success(), "commit {message}: {status}");
}
