//! `watch` tests: WatchSet reconcile (create/drop/absent-retry), the DirtySet
//! hand-off, the pure `pump` step (both arms), the real background [`Bridge`]
//! (a disk change surfaces as a dirty root), and [`EguiRepaint`].

use super::*;
use std::cell::Cell;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Instant;
use tempfile::{TempDir, tempdir};

/// A canonicalized workspace dir (mirrors `fs_watcher`'s test helper so paths
/// match the backend's spelling on macOS). The guard must be kept alive.
fn workspace() -> (TempDir, PathBuf) {
    let dir = tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    (dir, root)
}

/// Poll `probe` until it yields `Some` or `timeout` elapses (a budget generous
/// for FSEvents on the detection paths; tiny on the timeout-path test).
fn wait_until<T>(timeout: Duration, mut probe: impl FnMut() -> Option<T>) -> Option<T> {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if let Some(v) = probe() {
            return Some(v);
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    probe()
}

/// Detection budget for the real-watcher paths.
const DETECT: Duration = Duration::from_secs(5);

#[test]
fn wait_until_times_out_to_none() {
    assert!(wait_until(Duration::from_millis(20), || None::<()>).is_none());
}

/// A probe that misses once and then hits: the sleep-and-retry branch, pinned.
/// Every other caller hands `wait_until` a real watcher, so whether it ever
/// *retries* is up to how fast the backend happens to be — on a machine where
/// the first probe always hits, that line went uncovered and the 100% gate
/// flapped (bl-90bf, three runs in four). Timing decides speed here, never
/// which lines run.
#[test]
fn wait_until_retries_a_probe_that_missed() {
    let probes = Cell::new(0);
    let got = wait_until(DETECT, || {
        probes.set(probes.get() + 1);
        (probes.get() > 1).then_some(probes.get())
    });
    assert_eq!(got, Some(2), "the retry answered");
}

#[test]
fn reconcile_creates_then_drops_watchers() {
    let (_dir, root) = workspace();
    let mut set = WatchSet::new();
    assert!(set.is_empty());
    set.reconcile(&[(root.clone(), RootKind::Workspace)]);
    assert_eq!(set.len(), 1);
    assert!(set.watches(&root, RootKind::Workspace));
    // A surviving watcher stays; an unwanted one is dropped.
    set.reconcile(&[]);
    assert!(set.is_empty());
    assert!(!set.watches(&root, RootKind::Workspace));
}

#[test]
fn reconcile_keeps_surviving_watcher_instance() {
    let (_dir, a) = workspace();
    let (_dir_b, b) = workspace();
    let mut set = WatchSet::new();
    set.reconcile(&[(a.clone(), RootKind::Workspace)]);
    // Re-reconcile with `a` still desired plus `b`: `a` survives, `b` is added.
    set.reconcile(&[
        (a.clone(), RootKind::Workspace),
        (b.clone(), RootKind::Workspace),
    ]);
    assert_eq!(set.len(), 2);
    assert!(set.watches(&a, RootKind::Workspace));
    assert!(set.watches(&b, RootKind::Workspace));
}

#[test]
fn reconcile_skips_absent_root_and_retries_later() {
    let dir = tempdir().unwrap();
    let missing = dir.path().join("not-yet");
    let mut set = WatchSet::new();
    // Absent root: construction fails, skipped, set not poisoned.
    set.reconcile(&[(missing.clone(), RootKind::Workspace)]);
    assert!(set.is_empty());
    // It appears; the next reconcile arms it (retry, no stored absent state).
    std::fs::create_dir_all(&missing).unwrap();
    set.reconcile(&[(missing.clone(), RootKind::Workspace)]);
    assert_eq!(set.len(), 1);
}

#[test]
fn reconcile_rebuilds_a_replaced_root_instead_of_keeping_a_deaf_watcher() {
    // §7.3's "a watcher whose directory was deleted/recreated is rebuilt": the
    // desired set still holds the key, so a name-keyed diff alone leaves the
    // dead-inode watcher in place forever and the root goes silently unwatched
    // until the 15 s sweep. Staleness is what makes the promise true.
    let dir = tempdir().unwrap();
    let root = dir.path().join("clone");
    std::fs::create_dir_all(&root).unwrap();
    let desired = [(root.clone(), RootKind::Workspace)];
    let mut set = WatchSet::new();
    set.reconcile(&desired);
    let armed = std::fs::metadata(&root).map(|m| std::os::unix::fs::MetadataExt::ino(&m));
    // The root is re-primed: same path, new inode.
    std::fs::remove_dir_all(&root).unwrap();
    std::fs::create_dir_all(&root).unwrap();
    let replaced = std::fs::metadata(&root).map(|m| std::os::unix::fs::MetadataExt::ino(&m));
    assert_ne!(armed.ok(), replaced.ok(), "the inode really changed");
    set.reconcile(&desired);
    assert!(
        set.watches(&root, RootKind::Workspace),
        "re-armed, not kept"
    );
    // And the re-armed watcher actually hears the new inode.
    std::fs::create_dir_all(root.join("steps/abc/001")).unwrap();
    std::fs::write(root.join("steps/abc/001/request.json"), b"{}").unwrap();
    let dirty = wait_until(DETECT, || {
        let d = set.drain_dirty();
        (!d.is_empty()).then_some(d)
    })
    .expect("the rebuilt watcher fires");
    assert_eq!(dirty.get(&root), Some(&Mark::Watch));
}

#[test]
fn a_ticks_provenance_is_desync_only_when_the_backend_announced_a_loss() {
    let ordinary = [Change {
        path: PathBuf::from("/w/steps/a/001/request.json"),
        kind: ChangeKind::Touched,
    }];
    assert_eq!(mark_of(&ordinary), Mark::Watch);
    let lost = [
        Change {
            path: PathBuf::from("/w"),
            kind: ChangeKind::Desynced,
        },
        Change {
            path: PathBuf::from("/w/inbox/a/user-001.md"),
            kind: ChangeKind::Touched,
        },
    ];
    assert_eq!(
        mark_of(&lost),
        Mark::Desync,
        "a loss in the batch dominates the ordinary changes beside it"
    );
    // The merge rule the whole instrumentation rests on.
    assert!(Mark::Sweep < Mark::Poll && Mark::Poll < Mark::Watch && Mark::Watch < Mark::Desync);
}

#[test]
fn drain_dirty_reports_the_changed_root() {
    let (_dir, root) = workspace();
    let mut set = WatchSet::new();
    set.reconcile(&[(root.clone(), RootKind::Workspace)]);
    std::fs::create_dir_all(root.join("steps/abc/001")).unwrap();
    let _ = set.drain_dirty(); // absorb the arming/creation events
    std::fs::write(root.join("steps/abc/001/request.json"), b"{}").unwrap();
    let dirty = wait_until(DETECT, || {
        let d = set.drain_dirty();
        (!d.is_empty()).then_some(d)
    })
    .expect("a change surfaces");
    assert!(dirty.contains_key(&root));
}

#[test]
fn dirty_set_marks_and_drains() {
    let dirty = DirtySet::default();
    assert!(dirty.is_empty());
    dirty.mark_all([
        (PathBuf::from("/w/a"), Mark::Watch),
        (PathBuf::from("/w/b"), Mark::Watch),
    ]);
    assert!(!dirty.is_empty());
    let drained = dirty.drain();
    assert_eq!(drained.len(), 2);
    assert!(dirty.is_empty(), "drain clears the set");
}

#[test]
fn pump_is_false_without_a_change_and_marks_on_change() {
    let (_dir, root) = workspace();
    let mut set = WatchSet::new();
    set.reconcile(&[(root.clone(), RootKind::Workspace)]);
    let watchset = Arc::new(Mutex::new(set));
    let dirty = DirtySet::default();
    // Absorb arming events, then a quiet pump is false.
    let _ = watchset.lock().unwrap().drain_dirty();
    assert!(!pump(&watchset, &dirty));
    assert!(dirty.is_empty());
    // A change makes the next pump true and marks the root.
    std::fs::create_dir_all(root.join("inbox/a")).unwrap();
    std::fs::write(root.join("inbox/a/user-001.md"), b"hi").unwrap();
    let marked = wait_until(DETECT, || pump(&watchset, &dirty).then_some(()));
    assert!(marked.is_some());
    assert!(dirty.drain().contains_key(&root));
}

#[test]
fn the_bridge_thread_marks_a_real_disk_change_dirty() {
    let (_dir, root) = workspace();
    let mut set = WatchSet::new();
    set.reconcile(&[(root.clone(), RootKind::Workspace)]);
    let watchset = Arc::new(Mutex::new(set));
    let dirty = DirtySet::default();
    let bridge = Bridge::spawn(Arc::clone(&watchset), dirty.clone());
    std::fs::create_dir_all(root.join("steps/abc/001")).unwrap();
    std::fs::write(root.join("steps/abc/001/request.json"), b"{}").unwrap();
    let seen = wait_until(DETECT, || (!dirty.is_empty()).then_some(()));
    assert!(seen.is_some(), "the bridge marked the root dirty");
    drop(bridge); // clean stop + join
}

#[test]
fn egui_repaint_requests_without_panicking() {
    EguiRepaint(egui::Context::default()).request();
}

#[test]
fn the_windowless_repaint_does_nothing_by_contract() {
    // §8.5: `yog headless` has no event loop to wake — the whole impl.
    NoRepaint.request();
}

#[test]
fn a_shared_hook_forwards_to_the_hook_it_holds() {
    // The seam that lets one `Engine::boot` serve both faces: the difference
    // between them travels as a value (VISION §5 V5.4). Shared rather than
    // owned because two engine threads wake the face — the derivation worker
    // when a snapshot lands, the §7.2 follower when characters do.
    let shared: std::sync::Arc<dyn Repaint> =
        std::sync::Arc::new(EguiRepaint(egui::Context::default()));
    shared.request();
    std::sync::Arc::clone(&shared).request();
}
