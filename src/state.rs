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
//! **Three residents, and they are the whole inter-thread interface** (§7.2).
//! Since bl-ee0a yog runs three threads — the frame, the derivation worker, and
//! the watch bridge — so this file is the complete inventory of what they share:
//!
//! - [`WatchSetHandle`] — the shared [`WatchSet`](crate::watch::WatchSet): the
//!   worker reconciles it, the [`Bridge`](crate::watch::Bridge) drains it.
//! - [`DirtySet`] — **announcements → worker**: a map of root → [`Mark`] (why it
//!   is dirty). The bridge fills it from the watchers; the frame fills it when a
//!   dispatched verb changed something the watch would only find later. The
//!   worker drains it.
//! - [`SnapshotCell`] — **worker → frame**: the latest *completed*
//!   [`Snapshot`]. The worker swaps a fresh `Arc` in; the frame clones it out
//!   once per frame. The lock is held for exactly one pointer move on either
//!   side, so "the frame never blocks on the worker" is true by construction —
//!   there is no derivation inside this critical section to wait for.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use crate::app::Snapshot;
use crate::search::Found;
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

/// The §8.5 search hand-off — **frame ⇄ searcher, one cell**, because a search
/// is one question at a time and two cells could disagree about which question
/// is current. The frame writes the ask and reads the answer; the
/// [`Searcher`](crate::search::Searcher) does the reverse.
///
/// The serial is the whole protocol. Every ask bumps `seq`; a run carries the
/// seq it started on and publishes only if that is still the current one, so a
/// superseded run's work is discarded rather than raced. It is also the
/// **cancellation** signal: the run asks whether `seq` still equals its own
/// between conversations and abandons when it does not. Nothing here is
/// durable — a query's answer lives only as long as the surface that asked
/// (§5.3 #26).
#[derive(Clone, Default)]
pub struct SearchCell {
    inner: Arc<Mutex<SearchSlot>>,
}

/// The cell's contents: the current ask, and the answer to whichever ask has
/// been answered. `seq == answered` means nothing is outstanding — the starting
/// state, with no bootstrap branch to write.
#[derive(Default)]
struct SearchSlot {
    seq: u64,
    text: String,
    answered: u64,
    found: Found,
}

impl SearchCell {
    /// The poison-immune guard (see [`lock_cell`] for the discipline).
    fn guard(&self) -> MutexGuard<'_, SearchSlot> {
        self.inner.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Ask (frame side): supersede whatever was pending. Cheap by construction
    /// — one lock, one string move — because the frame is what calls it.
    pub fn ask(&self, text: &str) {
        let mut slot = self.guard();
        slot.seq = slot.seq.wrapping_add(1);
        text.clone_into(&mut slot.text);
    }

    /// The outstanding ask as `(seq, text)`, or `None` when the published
    /// answer already answers the current question.
    pub(crate) fn pending(&self) -> Option<(u64, String)> {
        let slot = self.guard();
        (slot.seq != slot.answered).then(|| (slot.seq, slot.text.clone()))
    }

    /// The current ask's serial — the run's liveness test.
    pub(crate) fn seq(&self) -> u64 {
        self.guard().seq
    }

    /// Publish (searcher side), iff `seq` is still the question being asked.
    pub(crate) fn publish(&self, seq: u64, found: Found) {
        let mut slot = self.guard();
        if slot.seq == seq {
            slot.answered = seq;
            slot.found = found;
        }
    }

    /// The published answer (frame side) — a clone, so the frame renders a
    /// value nothing can mutate under it.
    pub fn found(&self) -> Found {
        self.guard().found.clone()
    }

    /// Whether an ask is still outstanding — the "searching…" fact, derived
    /// from the same two serials rather than stored beside them.
    pub fn searching(&self) -> bool {
        let slot = self.guard();
        slot.seq != slot.answered
    }
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
