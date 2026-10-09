//! The crate's lock chokepoint (Bootstrap rule 7): the cross-thread
//! shared-mutable-state locks live in this one file, so the whole-crate
//! shared-state inventory is auditable in one place.
//! `rules/locks-outside-state.yml` enforces the confinement, and its only
//! carve-out is test scaffolding.
//!
//! **The residents, and they are the whole inter-thread interface** (§7.2).
//! This file is the complete inventory of what yog's threads share:
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
//! - [`PresenceCell`] — **the wire server → every answer**: which clients hold
//!   a live connection (REMOTE §5, [`registry::presence`](crate::registry::presence)).
//! - [`MailCell`] — **the invocation mailbox** (REMOTE §5): the queue per client
//!   and slot per invocation a routed tool call crosses
//!   ([`registry::mailbox`](crate::registry::mailbox)).
//! - [`hub_slots`] / [`hub_backend`] — **the process's one `notify` instance**
//!   and its fan-out registry (§7.1, [`fs_watcher`](crate::fs_watcher)): the
//!   backend's event thread delivers through the registry every watcher arms.
//! - `ProbeCacheCell` — the macOS 2 s liveness-probe TTL cache (§10), compiled
//!   only where it is used (macOS, and tests).

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex, MutexGuard, OnceLock, PoisonError};

use notify::{Event, RecommendedWatcher};

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

/// The live-connection map (REMOTE §5, bl-4e08): per identity, one stated
/// corpus edition per connection it holds. The wire server writes it, every
/// answer reads it; the map and every rule about it live with
/// [`Presence`](crate::registry::presence::Presence).
pub(crate) type PresenceCell = Arc<Mutex<BTreeMap<String, Vec<u32>>>>;

/// Lock the presence map, poison-immune — [`lock_cell`]'s one-line discipline.
pub(crate) fn lock_presence(cell: &PresenceCell) -> MutexGuard<'_, BTreeMap<String, Vec<u32>>> {
    cell.lock().unwrap_or_else(PoisonError::into_inner)
}

/// The invocation mailbox (REMOTE §5, bl-024b), shared by handle exactly as
/// [`PresenceCell`] is; its slots and rules live with
/// [`Mailbox`](crate::registry::mailbox::Mailbox).
pub(crate) type MailCell = Arc<Mutex<crate::registry::mailbox::slots::Slots>>;

/// Lock the mailbox, poison-immune — [`lock_cell`]'s one-line discipline.
pub(crate) fn lock_mail(cell: &MailCell) -> MutexGuard<'_, crate::registry::mailbox::slots::Slots> {
    cell.lock().unwrap_or_else(PoisonError::into_inner)
}

/// One watch-hub subscriber (§7.1, bl-908c): a canonical watched root and the
/// channel the watcher over it drains.
pub(crate) type HubSlot = (PathBuf, Sender<notify::Result<Event>>);

/// The hub's fan-out registry — a process singleton the backend's event thread
/// delivers through, so its callback captures nothing.
static HUB_SLOTS: OnceLock<Mutex<Vec<HubSlot>>> = OnceLock::new();

/// The process's one `notify` backend, or `None` if it could not be created.
static HUB_BACKEND: OnceLock<Option<Mutex<RecommendedWatcher>>> = OnceLock::new();

/// The hub's registry, locked poison-immune.
pub(crate) fn hub_slots() -> MutexGuard<'static, Vec<HubSlot>> {
    HUB_SLOTS
        .get_or_init(Mutex::default)
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
}

/// The hub's backend, built by `build` on first use and locked poison-immune;
/// `None` where it could not be built. The rule about never taking it while
/// [`hub_slots`] is held lives with the hub (`fs_watcher::hub`).
pub(crate) fn hub_backend(
    build: fn() -> Option<RecommendedWatcher>,
) -> Option<MutexGuard<'static, RecommendedWatcher>> {
    let lock = HUB_BACKEND
        .get_or_init(|| build().map(Mutex::new))
        .as_ref()?;
    Some(lock.lock().unwrap_or_else(PoisonError::into_inner))
}

/// The macOS liveness-probe TTL cache's map (§10): target path → when it was
/// observed and what was seen. Single-thread in practice (the probe traits
/// observe through `&self`); a resident here because every lock is.
#[cfg(any(test, target_os = "macos"))]
pub(crate) type ProbeCacheCell =
    Mutex<std::collections::HashMap<PathBuf, (std::time::Instant, crate::git_tree::Probe)>>;

/// Lock the probe cache, poison-immune — [`lock_cell`]'s one-line discipline.
#[cfg(any(test, target_os = "macos"))]
pub(crate) fn lock_probe_cache(
    cell: &ProbeCacheCell,
) -> MutexGuard<'_, std::collections::HashMap<PathBuf, (std::time::Instant, crate::git_tree::Probe)>>
{
    cell.lock().unwrap_or_else(PoisonError::into_inner)
}
