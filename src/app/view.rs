//! The model's read surface — everything a caller may ask an [`AppModel`], and
//! nothing that changes one.
//!
//! Split from `app/mod.rs` at the cap. The parent boots the model and takes the
//! worker's published snapshots; this is what is read off one. Every query here
//! is over [`AppModel::snap`] — a completed derivation the reader cannot change
//! and never waits for (§7.2).
//!
//! **What left with the window** (bl-7942): the focus and every accessor over
//! it. Which workspace and which conversation a seat is looking at is per-seat
//! state (REMOTE §7), held by the seat and named in the gesture it sends; a
//! server that kept a focus would be keeping one seat's view of itself.

use super::AppModel;
#[cfg(test)]
use crate::binding::Workspace;
use crate::git_tree::GitTree;
use std::path::Path;

impl AppModel {
    /// **Take the newest completed derivation**, if the worker published one
    /// since the last take. Returns whether the read source moved.
    ///
    /// It was `refresh` — one frame's whole model duty — and everything else it
    /// did was the frame's (bl-7942): the optimistic echo's retirement, the
    /// window's wire settle, the follow lane's, the act path's receipts, the §6
    /// ack held while a conversation was on screen. Every one of those is a
    /// seat's now, and what is left is the pointer swap.
    pub fn take(&mut self) -> bool {
        let latest = crate::state::latest_snapshot(&self.cell);
        let landed = !std::sync::Arc::ptr_eq(&self.snap, &latest);
        if landed {
            self.snap = latest;
            // Adopt an externally-changed `ui.json` the worker read for us
            // (§4.1, I5): unless it is this instance's own echo (content-hash
            // match), wholesale-adopt it — the converging seen/pins path every
            // instance shares.
            if let Some(bytes) = self.snap.ui_bytes.clone()
                && !self.ui.is_echo(&bytes)
            {
                self.ui.adopt(&bytes);
            }
        }
        landed
    }

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
    /// what a derivation dates its ages against, and the injected seam, so a
    /// test advances it like anything else.
    pub fn now_unix(&self) -> i64 {
        self.clock.unix()
    }

    /// The clock's live periods off the published snapshot (bl-3381) — the
    /// §7.2 rhythms everything else is spelled in terms of, so an operator who
    /// re-tunes the sweeps re-tunes them all and no reader touches disk.
    pub fn cadence(&self) -> super::Cadence {
        self.snap.cadence
    }

    /// The classified workspace set — a test-only reader of the roster input
    /// the boundary derives its listing from.
    #[cfg(test)]
    pub(crate) fn workspaces(&self) -> &[Workspace] {
        &self.snap.workspaces
    }

    /// **The `Query::Workspaces` answer** (bl-296f) — a test-only reader over
    /// the derivation the chokepoint answers from, which is the one thing
    /// `boundary::answer::answer` does for that query. A seat asks it over the
    /// wire; a test asks here, and the two cannot be two derivations because
    /// this is that one call.
    #[cfg(test)]
    pub(crate) fn ws_listing(&self) -> crate::boundary::reply::Workspaces {
        crate::boundary::answer::workspaces(&self.snap, &self.ui, self.now_unix())
    }

    /// The dirty hand-off to the worker (§7.2) — a test-only reader, so a test
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
    /// against the enumeration, by the same [`by_leaf`](crate::naming::by_leaf)
    /// rule the engine resolves a gesture's address by — a name nothing answers
    /// resolves to nothing rather than to a guess.
    pub fn workspace_path(&self, name: &str) -> Option<std::path::PathBuf> {
        self.snap.ws_path(name).ok()
    }

    /// Per-workspace roster facts (§6 rollup): attention-bearing agent count,
    /// total agent count, and whether any agent is running (Live/InFlight). An
    /// unfetched workspace contributes zeros.
    pub fn workspace_stats(&self, ws: &Path) -> (usize, usize, bool) {
        crate::boundary::answer::workspace_stats(&self.snap, &self.ui, ws)
    }

    /// The in-process query chokepoint (§8.5): the same
    /// [`answer`](crate::boundary::answer::answer) the deposit consumer runs,
    /// over `deps` — one derivation, two serializations (VISION §4.8). `deps`
    /// is the same [`boundary_deps`](Self::boundary_deps) every dispatch takes:
    /// the §9 config family's three reads (bl-0164) ask the world through it
    /// exactly as their writes do.
    pub fn answer(
        &self,
        deps: &crate::boundary::dispatch::Deps,
        query: &crate::boundary::Query,
        now_unix: i64,
    ) -> Result<crate::boundary::reply::Reply, String> {
        crate::boundary::answer::answer(query, deps, &self.ui, now_unix)
    }
}
