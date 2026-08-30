//! Filesystem watcher for a watched root.
//!
//! Exposes the subset of paths admitted by the root's [`RootKind`] allowlist
//! (DESIGN §7.1) as a drainable stream of coalesced change notifications. The
//! module is pure Rust — no egui/eframe dependency — so a future
//! `litany-ui-web` crate can reuse it unchanged. The watcher is strictly
//! read-only: it never mutates the repo.
//!
//! Backing impl: `notify::RecommendedWatcher` (inotify on Linux, kqueue on
//! BSD/macOS, polling fallback elsewhere) — **one instance for the whole
//! process**, fanned out per root by [`hub`], because a backend instance is a
//! per-user kernel budget and one per root exhausted it (bl-908c; the hub's doc
//! carries the argument). Coalescing collapses multiple
//! events for the same path within one tick window — rapid sequential
//! writes and atomic-rename sequences both emerge as a single change per
//! destination path.
//!
//! **The backend announces its own losses, and they are not dropped.** inotify
//! reports a kernel queue overflow (`IN_Q_OVERFLOW`) as a rescan-flagged event
//! carrying no path, and a watch it could not arm mid-tree (descriptor
//! exhaustion — `fs.inotify.max_user_watches`) as an `Err` on the same channel.
//! Both mean *"events under this root were lost"*. They surface as one
//! [`ChangeKind::Desynced`] change on the **root itself**, so the ordinary
//! whole-root re-derivation dissolves the case with no second mechanism — and
//! the loss is named rather than left for the 15 s sweep to quietly repair
//! (DESIGN §7.2, §7.3).

mod fold;
mod hub;
mod roots;

pub use roots::RootKind;

use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::sync::mpsc::Receiver;

use notify::{Event, event::EventKind};

use fold::{coalesce, drain, lead_with_desync};

#[derive(Debug, thiserror::Error)]
#[error("filesystem watcher: {0}")]
pub struct WatchError(#[from] notify::Error);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeKind {
    Touched,
    Removed,
    /// The backend lost events under this root (queue overflow, or a watch it
    /// could not arm). Carried on the **root path**: re-read the whole root.
    Desynced,
}

/// A watched root's inode identity — `(dev, ino)` of the armed directory. The
/// backend watches an *inode*, not a name: a root deleted and re-created (a
/// re-primed balls clone, a workspace rebuilt) leaves the armed watch pointing
/// at a dead inode that will never fire again. Comparing the armed identity
/// against the path's current one is the only way to see that, and it is what
/// makes DESIGN §7.3's "the 2 s reconcile rebuilds the watcher" true.
type Identity = (u64, u64);

fn identity_of(path: &Path) -> Option<Identity> {
    let meta = std::fs::metadata(path).ok()?;
    Some((meta.dev(), meta.ino()))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Change {
    pub path: PathBuf,
    pub kind: ChangeKind,
}

pub struct Watcher {
    repo_root: PathBuf,
    kind: RootKind,
    armed: Option<Identity>,
    rx: Receiver<notify::Result<Event>>,
}

/// Retire this root's subscription from the shared instance ([`hub`]). Without
/// it a dropped watcher would leave its descriptors armed for the life of the
/// process — the leak that owning a private instance used to hide.
impl Drop for Watcher {
    fn drop(&mut self) {
        hub::disarm(&self.repo_root);
    }
}

impl Watcher {
    /// Watch a litany workspace root — the original behavior, unchanged
    /// ([`RootKind::Workspace`]).
    pub fn new(repo_root: &Path) -> Result<Self, WatchError> {
        Self::with_kind(repo_root, RootKind::Workspace)
    }

    /// Watch `repo_root`, admitting only the paths in `kind`'s allowlist
    /// (DESIGN §7.1).
    pub fn with_kind(repo_root: &Path, kind: RootKind) -> Result<Self, WatchError> {
        // Canonicalize the root so the prefix we strip from emitted event
        // paths (`is_watched`) and the `Change.path` values match the
        // backend's own path spelling. The macOS FSEvents backend reports
        // fully-resolved paths (`/private/…`); a raw `/tmp/…` root would
        // never prefix-match them. This also validates existence, so a
        // missing root surfaces as `WatchError` (via `io::Error`).
        let repo_root = repo_root.canonicalize().map_err(notify::Error::from)?;
        let rx = hub::arm(&repo_root)?;
        let armed = identity_of(&repo_root);
        Ok(Self {
            repo_root,
            kind,
            armed,
            rx,
        })
    }

    /// Whether the directory this watcher armed is gone or has been replaced by
    /// a different inode. A stale watcher is deaf: its inotify watch descriptor
    /// still exists but points at an unlinked inode nothing will ever write to.
    /// [`WatchSet::reconcile`](crate::watch::WatchSet::reconcile) rebuilds on
    /// this (§7.3).
    pub fn is_stale(&self) -> bool {
        identity_of(&self.repo_root) != self.armed
    }

    /// Drain pending notify events and return one coalesced `Change` per
    /// affected watched path. Paths outside this root's [`RootKind`] allowlist
    /// (DESIGN §7.1) are dropped; rename-source events are dropped in favor of
    /// the destination. A backend-announced loss leads the list as a
    /// [`ChangeKind::Desynced`] change on the root (see the module doc).
    pub fn tick(&self) -> Vec<Change> {
        let mut raw: Vec<(PathBuf, EventKind)> = Vec::new();
        let desynced = drain(&self.rx, &mut raw);
        let changes = coalesce(&self.repo_root, self.kind, raw);
        lead_with_desync(&self.repo_root, desynced, changes)
    }
}

#[cfg(test)]
mod tests;
