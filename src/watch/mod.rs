//! Watch registry + repaint bridge (DESIGN §7.2, §15 Y6).
//!
//! Three pieces wire the built-but-unwired [`fs_watcher`](crate::fs_watcher)
//! to a live-re-rendering UI:
//!
//! - [`WatchSet`] owns one [`Watcher`] per `(root, RootKind)` and
//!   [`reconcile`](WatchSet::reconcile)s a *desired* root list against the live
//!   one — dropping watchers no longer wanted, creating missing ones, and
//!   leaving surviving watchers (and their armed inotify state) untouched. A
//!   construction failure is normal (a missing root is absent, not an error):
//!   it is skipped, never poisoning the set, and retried on the next reconcile
//!   (the live set is the single source of truth — "absent" is `desired`
//!   minus `live`, computed, never stored).
//! - [`DirtySet`](crate::state::DirtySet) is the announcement hand-off: a
//!   mutex-guarded map of dirty root → [`Mark`] the bridge fills and the §7.2
//!   derivation worker drains.
//! - [`Bridge`] is the ingest thread. DESIGN §7.2 describes it "blocking on the
//!   aggregated notify channels"; the existing [`Watcher`] is **pull-based**
//!   ([`Watcher::tick`] drains coalesced changes). Rather than grow the watcher
//!   a channel-exposing surface, the smallest faithful mechanism is a thread
//!   that *polls* every live watcher's `tick()` on a short interval and parks
//!   between polls ([`BRIDGE_POLL`]) — the pull API's equivalent of blocking on
//!   the channels.
//! - [`Repaint`] is the injected wake-the-window effect (the LockProbe
//!   template, so the thread that uses it is testable without a window).
//!
//! **Ingest stays its own thread, and that is deliberate** (bl-ee0a). The
//! derivation moved off the frame onto [`Worker`](crate::app::Worker), and it
//! would have been easy to let that one thread drain the watchers too. It must
//! not: an *announcement* and a *derivation* are the two halves §7.2's drift
//! instrumentation compares (a change with no announcement is a dropped event),
//! and folding them into one thread makes the comparison unobservable — nothing
//! could ever exercise "disk moved and nothing said so". Two threads, one
//! question each. The bridge no longer requests a repaint either: a dirty root
//! is not something to render, a published snapshot is (the worker's job).
//! Correctness never rides on this thread — the sweeps are the floor ("watches
//! are latency, polls are correctness", I4).

use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;
use std::time::Duration;

use crate::fs_watcher::{Change, ChangeKind, RootKind, Watcher};
use crate::state::{DirtySet, WatchSetHandle, lock_watchset};

/// Bridge poll cadence: how often the ingest thread drains the watchers. Short
/// enough that a disk change reaches the worker's next pass; the sweeps are the
/// correctness backstop (§7.2), so this is a latency knob only, deliberately not
/// clock-injected (it is a real thread sleep, not a time-gated decision under
/// test).
const BRIDGE_POLL: Duration = Duration::from_millis(50);

/// **Why** a root is dirty — the provenance a mark carries from whatever marked
/// it to the re-derivation that consumes it (DESIGN §7.2 instrumentation).
///
/// This is the whole instrumentation mechanism: a re-derivation that changes a
/// snapshot is *evidence* only in the light of what claimed the root had
/// changed. Under [`Watch`](Mark::Watch) it is the watcher working; under
/// [`Poll`](Mark::Poll) it is the liveness probe, for which no filesystem event
/// exists at all; under [`Sweep`](Mark::Sweep) **nobody announced it**, and that
/// is a dropped event, measured at the moment it costs something.
///
/// The variants are ordered weakest-explanation-first and merged with `max`, so
/// two marks on one root in one window keep the strongest explanation and a
/// blanket sweep mark can never mask a real announcement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Mark {
    /// The 15 s full sweep's blanket mark. A change found under it is a drop.
    Sweep,
    /// The 2 s cheap sweep's targeted liveness re-probe — process state, which
    /// emits no filesystem event by construction (a released flock is silent).
    Poll,
    /// The watcher announced an allowlisted change under this root (§7.1).
    Watch,
    /// The backend announced that it *lost* events under this root
    /// ([`ChangeKind::Desynced`]). The drop is real but **announced**: yog
    /// re-derives the root at once instead of waiting on the sweep, and records
    /// the desync.
    Desync,
}

/// The provenance a tick's worth of changes carries: [`Mark::Desync`] if the
/// backend announced a loss, else [`Mark::Watch`]. Pure over the changes so the
/// classification is provable without forcing a kernel into overflow.
fn mark_of(changes: &[Change]) -> Mark {
    if changes.iter().any(|c| c.kind == ChangeKind::Desynced) {
        Mark::Desync
    } else {
        Mark::Watch
    }
}

/// One [`Watcher`] per `(root, RootKind)` (DESIGN §7.1). The live map is the
/// only state; a desired root that fails to arm is simply not present and is
/// retried on the next [`reconcile`](Self::reconcile).
#[derive(Default)]
pub struct WatchSet {
    live: HashMap<(PathBuf, RootKind), Watcher>,
}

impl WatchSet {
    pub fn new() -> Self {
        Self::default()
    }

    /// Diff `desired` against the live watchers: drop every watcher no longer
    /// desired **or gone stale**, then create each desired watcher not already
    /// live. A surviving watcher is left in place, keeping its armed state.
    /// Construction failure (a missing root — normal) is skipped, never
    /// poisoning the set; the key stays absent and is retried on the next
    /// reconcile.
    ///
    /// Staleness is what makes §7.3's "the 2 s reconcile rebuilds the watcher
    /// from the enumerated root list" true rather than aspirational: a root
    /// deleted and re-created keeps its `(path, kind)` key, so a
    /// desired-set diff alone leaves the dead-inode watcher in place forever
    /// ([`Watcher::is_stale`]).
    pub fn reconcile(&mut self, desired: &[(PathBuf, RootKind)]) {
        self.live
            .retain(|key, watcher| desired.contains(key) && !watcher.is_stale());
        for key in desired {
            if !self.live.contains_key(key)
                && let Ok(watcher) = Watcher::with_kind(&key.0, key.1)
            {
                self.live.insert(key.clone(), watcher);
            }
        }
    }

    /// Drain every live watcher; return each root that saw at least one
    /// allowlisted change since the last drain, with its provenance (root
    /// granularity — the frame re-derives a whole root, §7.2). Two watchers of
    /// different kinds over one path merge to the stronger [`Mark`].
    pub fn drain_dirty(&self) -> BTreeMap<PathBuf, Mark> {
        let mut dirty = BTreeMap::new();
        for ((root, _kind), watcher) in &self.live {
            let changes = watcher.tick();
            if changes.is_empty() {
                continue;
            }
            let mark = mark_of(&changes);
            let slot = dirty.entry(root.clone()).or_insert(mark);
            *slot = (*slot).max(mark);
        }
        dirty
    }

    /// Number of live watchers.
    pub fn len(&self) -> usize {
        self.live.len()
    }

    pub fn is_empty(&self) -> bool {
        self.live.is_empty()
    }

    /// Whether a live watcher guards `(root, kind)`.
    pub fn watches(&self, root: &Path, kind: RootKind) -> bool {
        self.live.contains_key(&(root.to_path_buf(), kind))
    }
}

/// The ingest thread (DESIGN §7.2). Owns its join handle and a stop flag;
/// [`Drop`] signals stop, unparks, and joins for a clean shutdown.
pub struct Bridge {
    stop: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl Bridge {
    /// Spawn the poll-and-mark loop over the shared `watchset` and `dirty`
    /// hand-off. See the module doc for why this polls rather than blocks, and
    /// why it is not the derivation thread.
    pub fn spawn(watchset: WatchSetHandle, dirty: DirtySet) -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&stop);
        let handle = std::thread::spawn(move || {
            while !flag.load(Ordering::Relaxed) {
                pump(&watchset, &dirty);
                std::thread::park_timeout(BRIDGE_POLL);
            }
        });
        Self {
            stop,
            handle: Some(handle),
        }
    }
}

impl Drop for Bridge {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            handle.thread().unpark();
            let _ = handle.join();
        }
    }
}

/// One bridge iteration: drain the watchset into the dirty hand-off. Returns
/// whether anything became dirty, so both arms are unit-tested without the
/// thread.
fn pump(watchset: &WatchSetHandle, dirty: &DirtySet) -> bool {
    let roots = lock_watchset(watchset).drain_dirty();
    if roots.is_empty() {
        return false;
    }
    dirty.mark_all(roots);
    true
}

/// Effect: request an egui repaint. Injected on the LockProbe template
/// (DESIGN §12) so the derivation worker is exercised headlessly with a
/// counting double; [`EguiRepaint`] is the production impl.
pub trait Repaint: Send + Sync {
    fn request(&self);
}

/// A shared hook is a hook. What makes this load-bearing rather than plumbing:
/// [`Engine`](crate::engine::Engine) is the *one* assembly both faces boot, so
/// the face's difference — an event loop to wake, or none — has to travel as a
/// value rather than as a second call site (§8.5, VISION §5 V5.4). It is
/// **shared** rather than owned because more than one engine thread wakes the
/// face: the derivation worker when a snapshot lands, and the §7.2 live-tail
/// follower when characters do.
impl Repaint for Arc<dyn Repaint> {
    fn request(&self) {
        (**self).request();
    }
}

/// Production [`Repaint`]: wakes the egui event loop.
pub struct EguiRepaint(pub egui::Context);

impl Repaint for EguiRepaint {
    fn request(&self) {
        self.0.request_repaint();
    }
}

/// The windowless [`Repaint`] (§8.5): `yog headless` has no event loop to
/// wake — a published snapshot is simply the next thing the gesture consumer
/// reads. Doing nothing is the whole contract.
pub struct NoRepaint;

impl Repaint for NoRepaint {
    fn request(&self) {}
}

#[cfg(test)]
mod tests;
