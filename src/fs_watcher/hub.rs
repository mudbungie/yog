//! The process's ONE `notify` instance, fanned out per watched root (§7.1).
//!
//! **An inotify instance is not a private resource.**
//! `fs.inotify.max_user_instances` (128 by default) is a *per-user* kernel
//! budget shared with every other process the operator runs. yog used to open
//! one instance per watched root — five enumeration roots plus one per
//! workspace — and a failed arm is silent by design:
//! [`WatchSet::reconcile`](crate::watch::WatchSet::reconcile) skips the key and
//! retries. So the budget running out did not surface as an error, it surfaced
//! as watches that were simply never armed, and the §7.2 drift instrumentation
//! reading `false` from
//! [`WatchSet::watches`](crate::watch::WatchSet::watches). Four concurrent test
//! binaries, or a yog with a hundred workspaces, is ordinary load, not an
//! extreme (bl-908c).
//!
//! One instance can hold `fs.inotify.max_user_watches` (65536) descriptors, so
//! sharing it moves the constraint from the budget that binds at ~20 roots to
//! the one that binds at tens of thousands of directories. The hub owns that
//! instance for the whole process and delivers each raw event to every
//! registered root that contains it; the per-root [`RootKind`](super::RootKind)
//! filtering is untouched and still happens in [`Watcher::tick`](super::Watcher).
//!
//! **Two consequences of sharing, both deliberate:**
//!
//! - A backend *error* (a watch it could not arm mid-tree) has no reliable root
//!   attribution once the instances are one, so it is delivered to every live
//!   root. That is a superset of the truth — extra re-derivation, never a
//!   missed one — and it is what [`ChangeKind::Desynced`](super::ChangeKind)
//!   already means: "re-read this root".
//! - `notify`'s inotify backend removes every descriptor *underneath* the path
//!   it unwatches, so dropping a watcher on an enumeration root would deafen a
//!   workspace watcher nested inside it. [`disarm`] re-arms every overlapping
//!   live root instead of leaving a deaf watcher (§7.3).
//!
//! **Why the locks live here and not in `state.rs`** (Bootstrap rule 7's second
//! sanctioned carve-out, declared in `rules/locks-outside-state.yml`): both are
//! `OnceLock` process singletons that are never dropped and never handed out, so
//! they are not the cross-thread handoff `state.rs` inventories — and folding
//! them in costs the chokepoint its 100 % floor for the same llvm-cov reason
//! `git_tree::probe_cache` records: adding types there shifts the file's byte
//! offsets and llvm-cov mis-attributes phantom uncovered regions onto its `impl`
//! headers and type aliases (measured: 3 lines, on a file that is otherwise 100 %).

use super::WatchError;
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher as NotifyWatcher};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Mutex, MutexGuard, OnceLock, PoisonError};

/// One fan-out subscriber: a canonical watched root and the channel the
/// [`Watcher`](super::Watcher) over it drains.
type Slot = (PathBuf, Sender<notify::Result<Event>>);

/// The registry the backend's event thread delivers through — named directly by
/// the callback, which therefore captures nothing.
static SLOTS: OnceLock<Mutex<Vec<Slot>>> = OnceLock::new();

/// The process's one backend, or `None` if it could not be created at all (the
/// budget was gone before the first arm) — every caller then degrades exactly as
/// it did on a per-root arm failure.
static BACKEND: OnceLock<Option<Mutex<RecommendedWatcher>>> = OnceLock::new();

/// The registry, locked poison-immune (the one-line recovery discipline
/// `state::lock_watchset` records: a split reads as uncovered under
/// `ignore-panics`).
fn slots() -> MutexGuard<'static, Vec<Slot>> {
    SLOTS
        .get_or_init(|| Mutex::new(Vec::new()))
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
}

/// The backend, locked. Never taken while [`slots`] is held: the backend's own
/// event loop runs the fan-out callback, so a thread holding the registry across
/// a `watch()` call would deadlock against its own notification.
fn backend() -> Option<MutexGuard<'static, RecommendedWatcher>> {
    let lock = BACKEND.get_or_init(build).as_ref()?;
    Some(lock.lock().unwrap_or_else(PoisonError::into_inner))
}

fn build() -> Option<Mutex<RecommendedWatcher>> {
    notify::recommended_watcher(|res| deliver(&res))
        .map(Mutex::new)
        .ok()
}

/// Fan one raw backend message out to every root it concerns.
fn deliver(res: &notify::Result<Event>) {
    for (root, tx) in slots().iter() {
        if let Some(addressed) = addressed(root, res) {
            let _ = tx.send(addressed);
        }
    }
}

/// The message `root` should see, or `None` when this one is not its business.
///
/// A rescan flag (inotify `IN_Q_OVERFLOW`) and an error are the *instance's*
/// losses, not one root's, so both reach every root. An ordinary event is
/// narrowed to the paths under `root` — which keeps a rename's `(from, to)`
/// pair intact whenever both ends are inside it.
pub(super) fn addressed(root: &Path, res: &notify::Result<Event>) -> Option<notify::Result<Event>> {
    let Ok(event) = res else {
        return Some(Err(notify::Error::generic("watch backend loss")));
    };
    if event.need_rescan() {
        return Some(Ok(event.clone()));
    }
    let paths: Vec<PathBuf> = event
        .paths
        .iter()
        .filter(|p| p.starts_with(root))
        .cloned()
        .collect();
    (!paths.is_empty()).then(|| {
        Ok(Event {
            paths,
            ..event.clone()
        })
    })
}

/// Arm `root` (already canonicalized) on the shared instance and return the
/// channel its events arrive on. The watch is taken before the slot is
/// registered, so a failure leaves no subscriber behind.
pub(super) fn arm(root: &Path) -> Result<Receiver<notify::Result<Event>>, WatchError> {
    let mut armer = backend().ok_or_else(|| notify::Error::generic("no watch backend"))?;
    let (tx, rx) = mpsc::channel();
    armer.watch(root, RecursiveMode::Recursive)?;
    drop(armer);
    slots().push((root.to_path_buf(), tx));
    Ok(rx)
}

/// Retire one subscriber on `root`, unwatching it once nothing else wants it —
/// and re-arming every live root the unwatch took down as collateral (see the
/// module doc).
pub(super) fn disarm(root: &Path) {
    let live: Vec<PathBuf> = {
        let mut registry = slots();
        if let Some(at) = registry.iter().position(|(r, _)| r == root) {
            registry.swap_remove(at);
        }
        registry.iter().map(|(r, _)| r.clone()).collect()
    };
    if live.iter().any(|r| r == root) {
        return;
    }
    let Some(mut armer) = backend() else { return };
    let _ = armer.unwatch(root);
    for overlapping in live
        .iter()
        .filter(|r| r.starts_with(root) || root.starts_with(r))
    {
        let _ = armer.watch(overlapping, RecursiveMode::Recursive);
    }
}
