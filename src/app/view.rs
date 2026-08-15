//! The model's read surface — everything the shell may ask an [`AppModel`],
//! and nothing that changes one.
//!
//! Split from `app/mod.rs` at the cap. The parent boots the model and takes
//! the worker's published snapshots; this is what a frame *reads* off one.
//! Every query here is over [`AppModel::snap`] — a completed derivation the
//! frame cannot change and never waits for (§7.2).

use super::AppModel;
#[cfg(test)]
use super::Focus;
#[cfg(test)]
use crate::binding::Workspace;
use crate::binding::WorkspaceKind;
use crate::git_tree::GitTree;
use crate::nav;
use std::path::Path;

impl AppModel {
    /// The published-snapshot cell this model reads (§7.2) — what the gesture
    /// consumer answers queries from (§8.5), shared by `Arc`.
    pub fn snapshot_cell(&self) -> crate::state::SnapshotCell {
        std::sync::Arc::clone(&self.cell)
    }

    /// The durable `ui.json` path (§4.1) — the consumer's write-through copy
    /// opens the same file this instance adopts external changes from.
    pub fn ui_json_path(&self) -> std::path::PathBuf {
        self.roots.ui_json()
    }

    /// Wall-clock unix seconds off this model's **injected** clock (§7.2) —
    /// what a derivation dates its ages against. The shell mints its own at the
    /// process boundary for the seats that take one as a parameter; this is the
    /// same fact for the view-models that do not, and it is the injected seam,
    /// so a test advances it like anything else.
    pub(crate) fn now_unix(&self) -> i64 {
        self.clock.stamp().parse().unwrap_or(0)
    }

    /// The §11 ops-surface staleness line, or `None` while the rendered
    /// snapshot is current (§7.2). Honest by construction: it is the age of the
    /// derivation on screen, not a claim about how fresh it ought to be.
    pub fn staleness(&self) -> Option<String> {
        super::drift::stale_label(
            self.clock
                .now()
                .saturating_duration_since(self.snap.derived_at),
            self.snap.cadence.stale_after(),
        )
    }

    /// The clock's live periods off the rendered snapshot (bl-3381): what the
    /// frame derives its own rhythms from — the I4 poll floor
    /// (`cadence().cheap_sweep`) and the wound-banner grace
    /// ([`Cadence::wound_grace`](super::Cadence::wound_grace)) — so an operator
    /// who re-tunes the sweeps re-tunes everything spelled in terms of them,
    /// and no frame reads disk.
    pub fn cadence(&self) -> super::Cadence {
        self.snap.cadence
    }

    /// The classified workspace set — a test-only reader of the roster input
    /// the shell derives internally (§11).
    #[cfg(test)]
    pub(crate) fn workspaces(&self) -> &[Workspace] {
        &self.snap.workspaces
    }

    /// The frame→worker dirty hand-off (§7.2) — a test-only reader, so a test
    /// can mark a root with a specific [`Mark`](crate::watch::Mark) rather than
    /// the `Watch` every production caller means.
    #[cfg(test)]
    pub(crate) fn dirty_handle(&self) -> crate::state::DirtySet {
        self.dirty.clone()
    }

    /// What grew since the last derivation (§7.2), as the §11 ops accessory
    /// says it — `None` when nothing did. A dispatch storm is a fact about a
    /// *conversation*, and before bl-ee0a yog had no way to say it: 227 branches
    /// under one root read as yog being slow, so the operator's one signal
    /// pointed at the wrong layer.
    pub fn growth_note(&self) -> Option<String> {
        super::snapshot::growth_label(&self.snap.growth)
    }

    /// The current snapshot for `ws`, if derived.
    pub fn tree(&self, ws: &Path) -> Option<&GitTree> {
        self.snap.trees.get(ws)
    }

    /// **The one door from a §3.1 name to a path** (bl-7407): `name` resolved
    /// against the painted enumeration — which carries the §3.4 raise claim's
    /// wall ([`raise`](super::raise)), so a start's own workspace resolves from
    /// the frame the receipt lands rather than one derivation later.
    ///
    /// One resolver rather than a lookup per reader, so "what does this window
    /// mean by that workspace" has a single answer, and it is the same
    /// [`by_leaf`](crate::naming::by_leaf) rule the engine resolves a gesture's
    /// address by — a name nothing answers resolves to nothing rather than to a
    /// guess. Every seat that still needs a path (the §3.6 delete dialog, the
    /// pin key, the spawn cwd) reaches it through here or through
    /// [`focused_workspace`](Self::focused_workspace) above it.
    pub fn workspace_path(&self, name: &str) -> Option<std::path::PathBuf> {
        self.snap.ws_path(name).ok()
    }

    /// The focused workspace as a path — the focus's name through the one door.
    /// `None` for a name the enumeration does not answer, which is what an
    /// unfetched or deleted workspace already read as.
    pub fn focused_workspace(&self) -> Option<std::path::PathBuf> {
        self.workspace_path(self.focus.ws.as_deref()?)
    }

    /// The focused workspace's snapshot (the center panel renders this tree).
    pub fn focused_tree(&self) -> Option<&GitTree> {
        self.snap.trees.get(&self.focused_workspace()?)
    }

    /// Whether the focused workspace is a read-only replay (§3.1
    /// `<lernie-data>/replays/*`). "Replay is not a mode": the ordinary center
    /// view renders it through the same tree renderer — this query only gates
    /// the mutating composer off, so a replay offers no write surface.
    pub fn focused_is_replay(&self) -> bool {
        let Some(ws) = self.focused_workspace() else {
            return false;
        };
        self.snap
            .workspaces
            .iter()
            .any(|w| w.path == ws && w.kind == WorkspaceKind::Replay)
    }

    /// The per-instance focus (RAM, §13.1) — a test-only reader.
    #[cfg(test)]
    pub(crate) fn focus(&self) -> &Focus {
        &self.focus
    }

    /// The focused agent's snapshot row — the inspector's per-agent target
    /// (§11 Altitude-2): the [`Agent`](crate::git_tree::Agent) in the focused
    /// workspace's tree whose id is the focused agent id. `None` when no agent
    /// is selected or its row is absent (an unfetched or moved tree).
    pub fn focused_agent(&self) -> Option<&crate::git_tree::Agent> {
        let agent_id = self.focus.agent.as_deref()?;
        let tree = self.focused_tree()?;
        tree.agents.iter().find(|a| a.agent_id == agent_id)
    }
}

/// The derived projections a frame renders — the roster rollups, the tab
/// bar, the conversation rows and the activity chip. Split out of
/// `app/focus.rs` at the cap: those are *reads*, and focus.rs is the focus
/// and seen-acknowledgement **state machine** (§6). Same surface, one home.
impl AppModel {
    /// Per-workspace roster facts (§6 rollup): attention-bearing agent count,
    /// total agent count, and whether any agent is running (Live/InFlight). An
    /// unfetched workspace contributes zeros.
    pub fn workspace_stats(&self, ws: &Path) -> (usize, usize, bool) {
        crate::boundary::answer::workspace_stats(&self.snap, &self.ui, ws)
    }

    /// The §6 attention-strip total: attention-bearing agents across every
    /// workspace.
    pub fn strip_total(&self) -> usize {
        self.snap
            .workspaces
            .iter()
            .map(|w| self.workspace_stats(&w.path).0)
            .sum()
    }

    /// The §11 workspace tab bar (pinned hoists + named tabs; foreign/replay in
    /// the overflow), built from the classification + attention rollups + the
    /// `ui.json` pin order.
    pub fn tab_bar(&self) -> nav::tabs::TabBar {
        let items: Vec<nav::tabs::Item> = self
            .snap
            .workspaces
            .iter()
            .map(|w| nav::tabs::Item {
                ws: w.clone(),
                attention: self.workspace_stats(&w.path).0,
            })
            .collect();
        nav::tabs::build(&items, &self.ui.pinned(), self.focus.ws.as_deref())
    }

    /// The frame-side query chokepoint (§8.5): the same [`answer`]
    /// (crate::boundary::answer::answer) the deposit consumer runs, over
    /// `deps` — one derivation, two serializations (VISION §4.8). `deps` is
    /// the same [`boundary_deps`](Self::boundary_deps) every dispatch takes:
    /// the §9 config family's three reads (bl-0164) ask the world through it
    /// exactly as their writes do.
    ///
    /// **Search is the one query a frame hands over instead of running** (§8.5):
    /// it walks the world's bytes, and a frame that walked them would freeze.
    /// The ask goes to this instance's [`Searcher`](crate::search::Searcher) and
    /// the reply is whatever has landed — the same contract the frame already
    /// has with the derivation worker, and the same engine at the far end, so
    /// the seat is asynchronous without the derivation being duplicated.
    pub fn answer(
        &self,
        deps: &crate::boundary::dispatch::Deps,
        query: &crate::boundary::Query,
        now_unix: i64,
    ) -> Result<crate::boundary::reply::Reply, String> {
        if let crate::boundary::Query::Search { text } = query {
            self.search(text);
            return Ok(crate::boundary::reply::Reply::Search(self.found()));
        }
        crate::boundary::answer::answer(query, deps, &self.ui, now_unix)
    }

    /// The §11 **bottom in-flight strip** for the open conversation (bl-905f):
    /// the third seat of the one §5.1 #28 derivation, carrying the live
    /// characteristics of what is running. `None` — no strip painted, no
    /// repaint scheduled (§7.2) — whenever nothing is selected or nothing in
    /// the selection's conversation is in flight.
    /// `now_unix` is the caller's wall clock (the shell boundary mints it, as it
    /// does for the list's ages), so the elapsed segment ticks per frame off the
    /// snapshot's structural start with nothing stored (§5.1 #28).
    pub fn flight_strip(&self, now_unix: i64) -> Option<nav::convs::FlightStrip> {
        let agent = self.focus.agent.as_deref()?;
        let tree = self.focused_tree()?;
        let root = nav::convs::root_of(&tree.agents, agent)?;
        nav::convs::strip(&tree.agents, &root, now_unix)
    }

    /// The §11 **live mark's** seats for the open conversation (§5.1 #28b): the
    /// eye — the agent the operator is talking to — then its subagents in §2.3
    /// descent order, each with what it is doing right now.
    ///
    /// Empty whenever nothing is selected, which is not a case the mark
    /// branches on: no seats is every circle at rest, which is the logo.
    pub fn mark_seats(&self) -> Vec<nav::convs::Seat> {
        let Some(agent) = self.focus.agent.as_deref() else {
            return Vec::new();
        };
        let Some(tree) = self.focused_tree() else {
            return Vec::new();
        };
        let Some(root) = nav::convs::root_of(&tree.agents, agent) else {
            return Vec::new();
        };
        nav::convs::seats(&tree.agents, &root)
    }

    /// The §11 activity-accessory summary over the cached ops tail — the
    /// collapsed chip's counts, its ⚠ being the **live** failures only (§6's
    /// retirement rule); the expansion renders the trail over the wire (`Query::Ops`).
    pub fn activity(&self) -> crate::opslog::Activity {
        crate::opslog::activity(&self.snap.ops)
    }

    /// Whether a left-panel section carries a persisted collapse override
    /// (§4.1 `collapsed` — the balls section's key).
    pub fn is_collapsed(&self, key: &str) -> bool {
        self.ui.is_collapsed(key)
    }
}
