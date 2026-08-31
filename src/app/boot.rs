//! **Founding the model, and the one signal it sends the worker** (DESIGN §7.2).
//!
//! Split off [`super`] at §12's budget on the seam that root's own doc draws:
//! the root declares what a frame owns, and this is how one is brought into
//! being and how it reaches the derivation thread. Both members are the same
//! subject read from its two ends — [`AppModel::boot`] builds the [`Deriver`]
//! and hands the caller the pair, and [`AppModel::mark_dirty`] is the frame's
//! *only* outbound signal to it — so the root stays declaration-light and
//! carries no coverable `impl` header (the llvm-cov phantom this module tree is
//! arranged around).

use super::{AppModel, Deriver, Roots, Snapshot};
use crate::projects::runner::BlRunner;
use crate::state::{DirtySet, latest_snapshot, new_snapshot_cell};
use crate::ui_state::{Clock, UiState};
use crate::watch::Mark;
use std::path::PathBuf;
use std::sync::Arc;

impl AppModel {
    /// Build the model **and** the worker's [`Deriver`], taking the first
    /// derivation synchronously so the engine answers real content from its
    /// first gesture rather than an empty snapshot.
    ///
    /// Returned as a pair rather than spawned here: the thread is the caller's
    /// ([`Engine::boot`](crate::engine::Engine::boot) hands it a wake hook; a
    /// test drives `step()` by hand), and a model that spawned its own thread
    /// could not be exercised deterministically. This is the same shape the
    /// watch bridge always had.
    pub fn boot(
        roots: Roots,
        clock: Arc<dyn Clock>,
        balls: Box<dyn BlRunner>,
        user: Option<String>,
    ) -> (Self, Deriver) {
        let ui = UiState::open(roots.ui_json());
        let dirty = DirtySet::default();
        let cell = new_snapshot_cell(Arc::new(Snapshot::empty(clock.unix())));
        let mut deriver = Deriver::new(
            roots.clone(),
            Arc::clone(&clock),
            balls,
            dirty.clone(),
            Arc::clone(&cell),
        );
        deriver.boot();
        let snap = latest_snapshot(&cell);
        let model = Self {
            snap,
            roots,
            ui,
            cell,
            clock,
            dirty,
            identity_user: user,
        };
        (model, deriver)
    }

    /// Tell the worker a root changed (§7.2). The model's *only* outbound
    /// signal: a dispatched verb names the root it touched and the worker's
    /// ordinary routing does the rest, so there is no second path in.
    pub(crate) fn mark_dirty<I: IntoIterator<Item = PathBuf>>(&self, roots: I) {
        self.dirty
            .mark_all(roots.into_iter().map(|r| (r, Mark::Watch)));
    }
}
