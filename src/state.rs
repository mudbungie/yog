//! The crate's lock chokepoint (Bootstrap rule 7): the cross-thread
//! shared-mutable-state locks live in this one file, so the whole-crate
//! shared-state inventory is auditable in one place.
//! `rules/locks-outside-state.yml` enforces the confinement; the only carve-outs
//! are test scaffolding and one documented exception,
//! [`git_tree::probe_cache`](crate::git_tree) — the macOS TTL cache's `Mutex` is
//! single-thread interior mutability local to the probe stack, not cross-thread
//! shared state, and a generic decorator folded in here would break llvm-cov's
//! per-line coverage (see that module's doc).
//!
//! **Four residents, and they are the whole inter-thread interface** (§7.2).
//! Since bl-ee0a yog runs three threads — the frame, the derivation worker, and
//! the watch bridge — so this file is the complete inventory of what they share:
//!
//! - [`WatchSetHandle`] — the shared [`WatchSet`](crate::watch::WatchSet): the
//!   worker reconciles it, the [`Bridge`](crate::watch::Bridge) drains it.
//! - [`DirtySet`] — **announcements → worker**: a map of root → [`Mark`] (why it
//!   is dirty). The bridge fills it from the watchers; the frame fills it when a
//!   dispatched verb changed something the watch would only find later. The
//!   worker drains it.
//! - [`LoginCell`] — **the §8.3 sign-in runs** (REMOTE §8.3): the act seats a
//!   `bz --login` child, its own reader thread drains it, and any number of
//!   held lanes read the buffer.
//! - [`SnapshotCell`] — **worker → frame**: the latest *completed*
//!   [`Snapshot`]. The worker swaps a fresh `Arc` in; the frame clones it out
//!   once per frame. The lock is held for exactly one pointer move on either
//!   side, so "the frame never blocks on the worker" is true by construction —
//!   there is no derivation inside this critical section to wait for.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use crate::app::Snapshot;
use crate::watch::{Mark, WatchSet};

/// The shared [`WatchSet`](crate::watch::WatchSet): the §7.2 worker reconciles
/// it, the [`Bridge`](crate::watch::Bridge) drains it. A transparent alias so
/// `.lock()` stays ergonomic at the use sites while the `Mutex` token itself is
/// confined here.
pub type WatchSetHandle = Arc<Mutex<WatchSet>>;

/// Build a fresh, empty [`WatchSet`](crate::watch::WatchSet) behind its shared
/// handle — the one place `Mutex::new` is applied to the watch set.
pub(crate) fn new_watchset() -> WatchSetHandle {
    Arc::new(Mutex::new(WatchSet::new()))
}

/// Lock the shared watch set, poison-immune (see [`lock_cell`] for the same
/// one-line recovery discipline).
pub(crate) fn lock_watchset(handle: &WatchSetHandle) -> MutexGuard<'_, WatchSet> {
    handle.lock().unwrap_or_else(PoisonError::into_inner)
}

/// The published derivation (§7.2): the worker writes, the frame reads. A
/// transparent alias so the `Mutex` token itself stays confined here.
pub type SnapshotCell = Arc<Mutex<Arc<Snapshot>>>;

/// Build the cell around the model's starting (empty) snapshot — the one place
/// `Mutex::new` is applied to it.
pub(crate) fn new_snapshot_cell(initial: Arc<Snapshot>) -> SnapshotCell {
    Arc::new(Mutex::new(initial))
}

/// Lock the cell, poison-immune: a panic while the guard was held leaves the
/// `Arc` intact, so we recover it rather than propagate ([`PoisonError::into_inner`]).
/// Keeping the `.lock()` and the recovery on one line is deliberate — a split
/// isolates the never-taken recovery on its own line, which reads as uncovered
/// under `ignore-panics`.
fn lock_cell(cell: &SnapshotCell) -> MutexGuard<'_, Arc<Snapshot>> {
    cell.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Publish a completed derivation (worker side).
pub(crate) fn publish_snapshot(cell: &SnapshotCell, snapshot: Arc<Snapshot>) {
    *lock_cell(cell) = snapshot;
}

/// The latest completed derivation (frame side) — an `Arc` clone, so the frame
/// renders from a value nothing can mutate under it.
pub(crate) fn latest_snapshot(cell: &SnapshotCell) -> Arc<Snapshot> {
    Arc::clone(&lock_cell(cell))
}

/// The engine's live sign-in runs (REMOTE §8.3, bl-c285): one `bz --login`
/// child per workspace × provider, written by the act and by each run's own
/// reader thread, read by every lane held on one. A transparent alias, so the
/// `Mutex` token stays confined here while the map and every rule about it live
/// with the runs ([`login::runs`](crate::login::runs)).
pub(crate) type LoginCell = Arc<Mutex<crate::login::runs::Board>>;

/// Lock the sign-in runs, poison-immune — [`lock_cell`]'s one-line discipline.
pub(crate) fn lock_logins(cell: &LoginCell) -> MutexGuard<'_, crate::login::runs::Board> {
    cell.lock().unwrap_or_else(PoisonError::into_inner)
}

/// The dirty-root hand-off: root paths, each with the [`Mark`] naming **why**
/// it is dirty (§7.2 instrumentation). Cloning shares the inner map (the frame
/// holds one clone, the worker another).
#[derive(Clone, Default)]
pub struct DirtySet {
    inner: Arc<Mutex<BTreeMap<PathBuf, Mark>>>,
}

impl DirtySet {
    /// The poison-immune guard — the one `.lock()` site for the dirty set (see
    /// [`lock_cell`] for the same discipline on the snapshot cell).
    fn guard(&self) -> MutexGuard<'_, BTreeMap<PathBuf, Mark>> {
        self.inner.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Mark every root in `roots` dirty, keeping the strongest explanation when
    /// a root is marked twice before the frame drains it ([`Mark`] is ordered
    /// weakest-first, so `max` is the merge).
    pub(crate) fn mark_all<I: IntoIterator<Item = (PathBuf, Mark)>>(&self, roots: I) {
        let mut guard = self.guard();
        for (root, mark) in roots {
            let slot = guard.entry(root).or_insert(mark);
            *slot = (*slot).max(mark);
        }
    }

    /// Take and clear the dirty set (the frame consumes it each tick).
    pub fn drain(&self) -> BTreeMap<PathBuf, Mark> {
        std::mem::take(&mut self.guard())
    }

    pub fn is_empty(&self) -> bool {
        self.guard().is_empty()
    }
}
