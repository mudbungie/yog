//! The shared `AppModel` test harness: a tempdir world of workspaces and
//! replays, the injected clock/runner seams, and the agent-row builder every
//! sibling test module drives.
//!
//! Split from `tests/mod.rs` at the cap — the fixture and the tests that use
//! it change for unrelated reasons, and every sibling already imports it
//! through `super::Harness`. Split again at the same cap on the seam this
//! file's own doc lists: the world a test is given is here, and the pair of
//! calls it drives that world with is [`rig`].

mod rig;

pub(crate) use rig::Rig;

use super::*;
use crate::cli_outbound::Cli;
use crate::git_tree::tests::fixture::Fixture;
use crate::git_tree::{Agent, AgentState};
use crate::projects::runner::BlStore;
use crate::test_support::FakeClock;
use crate::xdg::Env;
use std::path::PathBuf;
use tempfile::{TempDir, tempdir};

/// The workspace-focused tests set up no balls clones, so the injected `bl`
/// runner is never consulted. A production [`BlStore`] (empty world, never-spawned
/// binary) satisfies the seam without a bespoke fake whose stubs would go
/// uncovered — its reads are proven in `projects::runner`, the ball/join wiring in
/// `crate::app::balls`.
pub(crate) fn no_balls() -> BlStore {
    let xdg = Env::from_pairs([("HOME", "/nonexistent")]).balls_layout();
    BlStore::new(xdg, Cli::new("bl"))
}

/// A hermetic test world: XDG roots under one tempdir, plus one ad-hoc
/// workspace (a real git fixture) symlinked under the litany workspaces root so
/// [`binding::workspaces`](crate::binding::workspaces) enumerates it. The
/// fixture's single agent (`c-1`) has no driver and no response — it classifies
/// [`AgentState::Stopped`], so its unseen stop is attention (§6).
pub(crate) struct Harness {
    _root: TempDir,
    pub(crate) fx: Fixture,
    pub(crate) roots: Roots,
    pub(crate) ws: PathBuf,
    /// Every fixture added after construction, held so its tempdir outlives the
    /// model — and, through [`last_added`](Harness::last_added), stays drivable.
    added: Vec<Fixture>,
}

impl Harness {
    pub(crate) fn new() -> Self {
        let mut h = Self::pristine();
        std::fs::create_dir_all(h.roots.workspaces()).unwrap();
        h.fx.build_agent("c-1", "hello");
        h.ws = h.roots.workspaces().join("ws");
        std::os::unix::fs::symlink(&h.fx.path, &h.ws).unwrap();
        h
    }

    /// A world as the operator first meets it: the yog state root and **nothing
    /// else** — no names root, no litany workspaces root, no workspace. The
    /// shape the §8.1 start flow founds its first workspace into
    /// (`execute_ensure_workspace`'s `create_dir_all(parent)` + `litany new`),
    /// and therefore the shape the §7.2 sweep first meets that workspace in.
    ///
    /// `ws` is the not-yet-existing path a workspace *would* take; a pristine
    /// harness's tests name what they mint instead.
    pub(crate) fn pristine() -> Self {
        let root = tempdir().unwrap();
        let roots = Roots {
            yog_data: root.path().join("yog"),
            litany_data: root.path().join("litany"),
            yog_state: root.path().join("state"),
            balls_clones: root.path().join("balls").join("clones"),
            home: root.path().join("home"),
            // A world **under this tempdir**, so a §9 destination the
            // derivation reads (the global `models.yaml`, §9.2) is a real path
            // a test can write. Hermetic either way: nothing here is the
            // operator's, and an unexpected write lands in the tempdir.
            world: crate::test_support::world_under(root.path()),
        };
        std::fs::create_dir_all(&roots.yog_state).unwrap();
        let ws = roots.workspaces().join("ws");
        Self {
            _root: root,
            fx: Fixture::new(),
            roots,
            ws,
            added: Vec::new(),
        }
    }

    /// Mint a **named** workspace the way the start flow does (§3.1, §8.1):
    /// found the names root if it is absent, then land the workspace in it.
    pub(crate) fn mint_named(&mut self, name: &str, agent: &str) -> PathBuf {
        std::fs::create_dir_all(self.roots.names()).unwrap();
        self.add_at(self.roots.names().join(name), agent)
    }

    /// A second ad-hoc workspace symlinked in (a new git fixture), returning its
    /// enumerated path.
    pub(crate) fn add_workspace(&mut self, name: &str, agent: &str) -> PathBuf {
        self.add_at(self.roots.workspaces().join(name), agent)
    }

    /// A read-only replay workspace symlinked under the litany replays root
    /// (§3.1) — the same fixture shape as an ad-hoc, classified `Replay` by its
    /// root alone.
    pub(crate) fn add_replay(&mut self, name: &str, agent: &str) -> PathBuf {
        std::fs::create_dir_all(self.roots.replays()).unwrap();
        self.add_at(self.roots.replays().join(name), agent)
    }

    /// Land a fresh one-agent fixture at `ws` and retain it. The one place a
    /// post-construction workspace is made; the callers differ only in root.
    fn add_at(&mut self, ws: PathBuf, agent: &str) -> PathBuf {
        let fx = Fixture::new();
        fx.build_agent(agent, "hi");
        std::os::unix::fs::symlink(&fx.path, &ws).unwrap();
        self.added.push(fx);
        ws
    }

    /// The most recently added workspace's fixture — for a test that moves its
    /// disk again after the model has snapshotted it.
    pub(crate) fn last_added(&self) -> &Fixture {
        self.added.last().expect("a workspace was added")
    }

    /// Land another agent on the primary workspace's disk (a live mutation the
    /// re-derivation tests observe).
    pub(crate) fn build_more(&self, id: &str, msg: &str) {
        self.fx.build_agent(id, msg);
    }

    pub(crate) fn model(&self) -> (FakeClock, Rig) {
        let clock = FakeClock::new();
        let (model, deriver) = AppModel::boot(
            self.roots.clone(),
            clock.arc(),
            Box::new(no_balls()),
            Some("me".to_string()),
        );
        (clock, Rig { model, deriver })
    }
}

/// A synthetic agent row (the Live/InFlight states need a held lock and can't be
/// reached against a dead fixture, so they are injected into `trees`).
pub(crate) fn agent(id: &str, state: AgentState) -> Agent {
    Agent {
        branch_name: format!("agents/{id}"),
        agent_id: id.to_string(),
        tip_oid: "d".repeat(40),
        tip_short_oid: "dddddddd".into(),
        tip_timestamp_unix: 4,
        last_action_unix: 4,
        messages: 0,
        steps: vec![],
        preview: None,
        stream: crate::git_tree::Stream::default(),
        tool_calls: Vec::new(),
        state,
        state_uncertain: false,
        truncated: false,
        failure: None,
        pending: vec![],
        conflicted_oid: None,
        budget_oid: None,
        abandoned_oid: None,
        notify_oid: None,
        held: None,
        goal_ball: None,
        name: None,
        goal_name: None,
        call_start_unix: None,
    }
}
