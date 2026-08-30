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
        self.clock.unix()
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

    /// **The `Query::Workspaces` answer** (bl-296f) — a test-only reader over
    /// the **derivation** the chokepoint answers from, which is the one thing
    /// `boundary::answer::answer` does for that query. The window asks it over
    /// the wire; a test asks here, through `test_support::chrome`, and the two
    /// cannot be two derivations because this is that one call.
    #[cfg(test)]
    pub(crate) fn ws_listing(&self) -> crate::boundary::reply::Workspaces {
        crate::boundary::answer::workspaces(&self.derived, &self.ui, self.now_unix())
    }

    /// The frame→worker dirty hand-off (§7.2) — a test-only reader, so a test
    /// can mark a root with a specific [`Mark`](crate::watch::Mark) rather than
    /// the `Watch` every production caller means.
    #[cfg(test)]
    pub(crate) fn dirty_handle(&self) -> crate::state::DirtySet {
        self.dirty.clone()
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

    /// The focused workspace's name (§3.1): the **target** an Assign
    /// (`bl claim <id> --as <name>`) stamps (§8.2/§3.2). `None` when no
    /// workspace is focused (the affordance is then withheld).
    ///
    /// **The focus verbatim since bl-7407** — it holds the name — where it was
    /// a leaf derivation off a held path. The derivation did not move, it
    /// dissolved: one fact, one home. It sits here since bl-b4b5, beside the
    /// resolver and the path below it.
    pub fn focused_ws_name(&self) -> Option<String> {
        self.focus.ws.clone()
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
    /// `<litany-data>/replays/*`). "Replay is not a mode": the ordinary center
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

/// The derived projections a frame renders — what is left of them. Split out
/// of `app/focus.rs` at the cap: those are *reads*, and focus.rs is the focus
/// and seen-acknowledgement **state machine** (§6). Same surface, one home.
///
/// The §11 altitude-0 chrome that used to live here — the tab bar, the strip
/// total, the activity chip's counts, the live mark's seats and the in-flight
/// strip — is gone (bl-296f): every one of them is now a fold at the seat over
/// an answer the boundary landed (`Query::Workspaces`, `Query::Ops`,
/// `Query::Agent`), which is REMOTE §9.7's three-move discipline with the third
/// move being a subtraction rather than a new question. **The §7.2 staleness
/// and growth lines went the same way with bl-b4b5**, once the snapshot carried
/// its completion as a wall-clock stamp: they are two fields on the
/// `Query::Workspaces` answer the tab bar above them already stands on.
impl AppModel {
    /// Per-workspace roster facts (§6 rollup): attention-bearing agent count,
    /// total agent count, and whether any agent is running (Live/InFlight). An
    /// unfetched workspace contributes zeros.
    pub fn workspace_stats(&self, ws: &Path) -> (usize, usize, bool) {
        crate::boundary::answer::workspace_stats(&self.snap, &self.ui, ws)
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

    /// Whether a left-panel section carries a persisted collapse override
    /// (§4.1 `collapsed` — the balls section's key).
    pub fn is_collapsed(&self, key: &str) -> bool {
        self.ui.is_collapsed(key)
    }
}
