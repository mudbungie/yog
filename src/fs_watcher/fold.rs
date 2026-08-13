//! The pure raw-event fold: notify's channel in, coalesced [`Change`]s out.
//!
//! Every step is a plain function over values — a `Receiver`, a `Vec` of
//! `(path, EventKind)`, a bool. That is deliberate (§7.3): a real kernel queue
//! overflow and a descriptor exhaustion cannot be provoked from a test, so the
//! loss arms are proved over a plain channel instead, and the assembly that
//! turns them into a root [`ChangeKind::Desynced`] is proved with them. [`super`]
//! holds the part that cannot be: the armed `notify` watch and its inode identity.

use super::{Change, ChangeKind, RootKind, roots::is_watched};
use notify::{
    Event,
    event::{EventKind, ModifyKind, RenameMode},
};
use std::collections::{HashMap, hash_map::Entry};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, TryRecvError};

/// Prepend the root's [`ChangeKind::Desynced`] marker when the backend announced
/// a loss, else hand the changes back untouched. Split from [`Watcher::tick`]
/// for the same reason as [`drain`]: a real kernel queue overflow cannot be
/// provoked from a test, and an untested assembly is exactly where a silent
/// drop would hide again.
pub(super) fn lead_with_desync(
    root: &Path,
    desynced: bool,
    mut changes: Vec<Change>,
) -> Vec<Change> {
    if desynced {
        changes.insert(
            0,
            Change {
                path: root.to_path_buf(),
                kind: ChangeKind::Desynced,
            },
        );
    }
    changes
}

/// Drain `rx` into `raw`, returning whether the backend announced a **loss** —
/// a rescan-flagged event (inotify `IN_Q_OVERFLOW`) or an error on the channel
/// (a watch it could not arm: descriptor exhaustion). Split from
/// [`Watcher::tick`] so both loss arms are provable over a plain channel,
/// without a kernel that has to be pushed into overflow first.
pub(super) fn drain(
    rx: &Receiver<notify::Result<Event>>,
    raw: &mut Vec<(PathBuf, EventKind)>,
) -> bool {
    let mut desynced = false;
    loop {
        match rx.try_recv() {
            Ok(Ok(event)) => desynced |= ingest(event, raw),
            Ok(Err(_)) => desynced = true,
            Err(TryRecvError::Empty | TryRecvError::Disconnected) => break,
        }
    }
    desynced
}

/// Fold one event into `raw`, returning whether it is the backend's own
/// **rescan** signal — an event that names no path and means "events were
/// lost here" (`notify::Event::need_rescan`, set on inotify `IN_Q_OVERFLOW`).
fn ingest(event: Event, raw: &mut Vec<(PathBuf, EventKind)>) -> bool {
    if event.need_rescan() {
        return true;
    }
    let paths = event.paths;
    let kind = event.kind;
    // A `Both` rename carries exactly the (from, to) pair — a slice pattern
    // binds both without indexing; any other shape falls through unchanged.
    if let EventKind::Modify(ModifyKind::Name(RenameMode::Both)) = kind
        && let [from, to] = paths.as_slice()
    {
        raw.push((
            from.clone(),
            EventKind::Modify(ModifyKind::Name(RenameMode::From)),
        ));
        raw.push((
            to.clone(),
            EventKind::Modify(ModifyKind::Name(RenameMode::To)),
        ));
        return false;
    }
    for path in paths {
        raw.push((path, kind));
    }
    false
}

pub(super) fn coalesce(
    repo_root: &Path,
    root_kind: RootKind,
    raw: Vec<(PathBuf, EventKind)>,
) -> Vec<Change> {
    let mut renamed: HashMap<PathBuf, bool> = HashMap::new();
    let mut order: Vec<PathBuf> = Vec::new();
    for (path, kind) in raw {
        if !is_watched(root_kind, repo_root, &path) {
            continue;
        }
        let is_rename = matches!(kind, EventKind::Modify(ModifyKind::Name(_)));
        match renamed.entry(path.clone()) {
            Entry::Occupied(mut e) => *e.get_mut() |= is_rename,
            Entry::Vacant(e) => {
                order.push(path);
                e.insert(is_rename);
            }
        }
    }
    order
        .into_iter()
        .filter_map(|p| {
            let change_kind = classify(renamed.get(&p).copied().unwrap_or(false), &p)?;
            Some(Change {
                path: p,
                kind: change_kind,
            })
        })
        .collect()
}

/// Classify a watched path into a surfaced [`ChangeKind`], or `None` to drop
/// it, from two established facts: whether the path exists now, and whether any
/// rename (`Modify(Name(_))`) event landed on it this tick (`renamed`,
/// OR-folded across the path's whole event burst in `coalesce`).
///
/// Existence is ground truth for the current state, so a path present now is
/// `Touched` whatever its history. A path that is gone is disambiguated by
/// *how* it left: an atomic-write rename source carried a `Name` event and its
/// destination survives, so it is dropped (`None`); a genuine deletion carried
/// no rename, so it surfaces as `Removed`. This depends only on the invariants
/// that a rename source always carries a `Name` event and a delete never does —
/// not on whether macOS FSEvents emits a trailing `Remove` for a deletion. It
/// does not: the coalesced `CREATED|REMOVED` burst's last event for the
/// vanished path is a non-`Remove` `Modify(Data)`, so keying on the *presence*
/// or *absence* of a `Remove` event misreads both a real deletion and a rename
/// source. The OR-fold is essential — that trailing `Modify(Data)` must not
/// clear the `Name` seen earlier in the same tick.
fn classify(renamed: bool, path: &Path) -> Option<ChangeKind> {
    if path.exists() {
        Some(ChangeKind::Touched)
    } else if renamed {
        None
    } else {
        Some(ChangeKind::Removed)
    }
}

#[cfg(test)]
mod tests;
