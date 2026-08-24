//! The shared `AppModel` test harness: a tempdir world of workspaces and
//! replays, the injected clock/runner seams, and the agent-row builder every
//! sibling test module drives.
//!
//! Split from `tests/mod.rs` at the cap — the fixture and the tests that use
//! it change for unrelated reasons, and every sibling already imports it
//! through `super::Harness`.

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
/// workspace (a real git fixture) symlinked under the lernie workspaces root so
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
    /// else** — no names root, no lernie workspaces root, no workspace. The
    /// shape the §8.1 start flow founds its first workspace into
    /// (`execute_ensure_workspace`'s `create_dir_all(parent)` + `lernie new`),
    /// and therefore the shape the §7.2 sweep first meets that workspace in.
    ///
    /// `ws` is the not-yet-existing path a workspace *would* take; a pristine
    /// harness's tests name what they mint instead.
    pub(crate) fn pristine() -> Self {
        let root = tempdir().unwrap();
        let roots = Roots {
            yog_data: root.path().join("yog"),
            lernie_data: root.path().join("lernie"),
            yog_state: root.path().join("state"),
            balls_clones: root.path().join("balls").join("clones"),
            home: root.path().join("home"),
            world: crate::test_support::no_world(),
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

    /// A read-only replay workspace symlinked under the lernie replays root
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

    /// Settle `agent`'s step 001 with `response` bytes — the §4.4 framing
    /// input (e.g. the live kind:auth failure shape).
    pub(crate) fn write_response(&self, agent: &str, response: &[u8]) {
        let step = self.fx.path.join("steps").join(agent).join("001");
        std::fs::create_dir_all(&step).unwrap();
        std::fs::write(step.join("response.json"), response).unwrap();
    }

    pub(crate) fn model(&self) -> (FakeClock, Rig) {
        self.model_focused(None)
    }

    pub(crate) fn model_focused(&self, initial: Option<PathBuf>) -> (FakeClock, Rig) {
        let clock = FakeClock::new();
        let (model, deriver) = AppModel::boot(
            self.roots.clone(),
            initial,
            clock.arc(),
            Box::new(no_balls()),
            Some("me".to_string()),
        );
        (clock, Rig { model, deriver })
    }
}

/// The frame and its worker, driven **by hand** (§7.2).
///
/// In production these are two threads: [`Worker`](crate::app::Worker) calls
/// `Deriver::step` forever and the frame calls [`AppModel::refresh`] per paint.
/// A test wants the same two calls in a known order, with no thread and no
/// sleeps, so the rig pairs them and [`tick`](Rig::tick) is one full round —
/// derive, then take. Every timing branch is reached by advancing the injected
/// clock, exactly as before the derivation moved off the frame.
///
/// It derefs to the model so a test asks the *frame* its questions the way the
/// shell does, and reaches `rig.deriver` only for facts the worker owns.
pub(crate) struct Rig {
    pub(crate) model: AppModel,
    pub(crate) deriver: crate::app::Deriver,
}

impl Rig {
    /// One derivation pass followed by one frame's take. Returns whether the
    /// frame's render source moved.
    pub(crate) fn tick(&mut self) -> bool {
        self.deriver.step();
        self.model.refresh()
    }

    /// Publish whatever the worker currently holds, without a pass. The seam
    /// for the Live/InFlight states, which need a held flock no fixture can
    /// take: a test writes the agent row into the worker's tree and this makes
    /// the frame see it, exactly as a real derivation would have.
    pub(crate) fn publish(&mut self) {
        self.deriver.publish();
        self.model.refresh();
    }

    /// The §11 all-collapsed list, asked through the boundary as every seat asks
    /// it (REMOTE §9.7, bl-44e9) — the model holds no conversation accessor any
    /// more, so the rig carries the seat's own door rather than each test
    /// spelling `Query::Conversations` for itself.
    pub(crate) fn conversations(&self, now_unix: i64) -> Vec<crate::nav::convs::ConvRow> {
        crate::test_support::convs::conversations(&self.model, now_unix)
    }

    /// The same answer, folded by a viewport holding `expanded`.
    pub(crate) fn visible_conversations(
        &self,
        now_unix: i64,
        expanded: &std::collections::HashSet<String>,
    ) -> Vec<crate::nav::convs::ConvRow> {
        crate::test_support::convs::visible(&self.model, now_unix, expanded)
    }

    /// The §6 attention-strip total, asked through the boundary (bl-296f) — the
    /// model holds no rollup accessor any more, the top bar folding
    /// `Query::Workspaces` for both the strip and the tabs.
    pub(crate) fn strip_total(&self) -> usize {
        crate::test_support::chrome::strip_total(&self.model)
    }

    /// The §11 workspace tab bar, folded off the same answer.
    pub(crate) fn tab_bar(&self) -> crate::nav::tabs::TabBar {
        crate::test_support::chrome::tab_bar(&self.model)
    }

    /// The §11 activity chip's counts, folded off the `Query::Ops` answer the
    /// expanded trail paints.
    pub(crate) fn activity(&self) -> crate::opslog::Activity {
        crate::test_support::chrome::activity(&self.model)
    }
}

impl std::ops::Deref for Rig {
    type Target = AppModel;
    fn deref(&self) -> &AppModel {
        &self.model
    }
}

impl std::ops::DerefMut for Rig {
    fn deref_mut(&mut self) -> &mut AppModel {
        &mut self.model
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
