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
///
/// Twenty seconds, not five, and the number is a **platform** measurement
/// rather than a guess (bl-1015): on a three-core `macos-14` runner carrying
/// ~2000 tests in parallel, FSEvents delivery past five seconds was reproducible
/// — two different beats, two different runs. Nothing here polls for twenty
/// seconds in the healthy case; the budget is only ever spent by a beat that is
/// about to fail, so raising it costs a red run time and a green run nothing.
const DETECT: Duration = Duration::from_secs(20);

/// Wait until the set's watchers are **provably** armed, then absorb what
/// arming itself made.
///
/// The same discipline as `fs_watcher::tests::wait_quiet`, and for the same
/// reason: a watch is not live when `reconcile` returns. inotify arms inside
/// the syscall, so on Linux a bare drain was indistinguishable from
/// correctness; macOS FSEvents starts its stream on another thread, and a write
/// that lands before it is running emits **no event at all** — which no
/// detection budget downstream can recover, because there is nothing left to
/// detect. So arming is observed rather than slept on: rewrite a probe file
/// until the set reports something, then drain until the set goes quiet.
fn wait_armed(set: &mut WatchSet, root: &Path) {
    let probe = root.join("steps/.arming-probe");
    std::fs::create_dir_all(probe.parent().unwrap()).unwrap();
    let armed = wait_until(DETECT, || {
        std::fs::write(&probe, b"x").ok()?;
        (!set.drain_dirty().is_empty()).then_some(())
    });
    assert!(armed.is_some(), "the watch set never armed");
    // The probe file stays: removing it is one more event for the next beat to
    // trip over, and a file under `steps/` nothing reads is invisible to every
    // assertion here.
    while wait_until(QUIET, || (!set.drain_dirty().is_empty()).then_some(())).is_some() {}
}

/// How long the set must say nothing for arming's own events to be counted
/// spent. Bounded and re-entered, never a single sleep: one settle that
/// happened to be short leaves the next beat reading this beat's noise.
const QUIET: Duration = Duration::from_millis(150);

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
    // The root is re-primed: same path, new inode — guaranteed by construction
    // rather than hoped for, or a filesystem that recycles the freed inode
    // leaves this test reproducing nothing at all (bl-e492).
    crate::test_support::replace_directory(&root);
    let replaced = std::fs::metadata(&root).map(|m| std::os::unix::fs::MetadataExt::ino(&m));
    assert_ne!(armed.ok(), replaced.ok(), "the inode really changed");
    set.reconcile(&desired);
    assert!(
        set.watches(&root, RootKind::Workspace),
        "re-armed, not kept"
    );
    // And the re-armed watcher actually hears the new inode.
    wait_armed(&mut set, &root);
    let target = root.join("steps/abc/001/request.json");
    std::fs::create_dir_all(target.parent().unwrap()).unwrap();
    let dirty = wait_until(DETECT, || {
        std::fs::write(&target, b"{}").ok()?;
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
    wait_armed(&mut set, &root);
    let target = root.join("steps/abc/001/request.json");
    std::fs::create_dir_all(target.parent().unwrap()).unwrap();
    // Written on every sample, not once: the claim is that a change under the
    // root surfaces, and re-making the change costs nothing while making the
    // beat immune to a single delivery the backend drops under load (bl-1015).
    let dirty = wait_until(DETECT, || {
        std::fs::write(&target, b"{}").ok()?;
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
    wait_armed(&mut set, &root);
    let watchset = Arc::new(Mutex::new(set));
    let dirty = DirtySet::default();
    // Armed and quiet, so a pump with nothing under it is false.
    assert!(!pump(&watchset, &dirty));
    assert!(dirty.is_empty());
    // A change makes the next pump true and marks the root.
    let deposit = root.join("inbox/a/user-001.md");
    std::fs::create_dir_all(deposit.parent().unwrap()).unwrap();
    let marked = wait_until(DETECT, || {
        std::fs::write(&deposit, b"hi").ok()?;
        pump(&watchset, &dirty).then_some(())
    });
    assert!(marked.is_some());
    assert!(dirty.drain().contains_key(&root));
}

#[test]
fn the_bridge_thread_marks_a_real_disk_change_dirty() {
    let (_dir, root) = workspace();
    let mut set = WatchSet::new();
    set.reconcile(&[(root.clone(), RootKind::Workspace)]);
    wait_armed(&mut set, &root);
    let watchset = Arc::new(Mutex::new(set));
    let dirty = DirtySet::default();
    let bridge = Bridge::spawn(Arc::clone(&watchset), dirty.clone());
    let target = root.join("steps/abc/001/request.json");
    std::fs::create_dir_all(target.parent().unwrap()).unwrap();
    let seen = wait_until(DETECT, || {
        std::fs::write(&target, b"{}").ok()?;
        (!dirty.is_empty()).then_some(())
    });
    assert!(seen.is_some(), "the bridge marked the root dirty");
    drop(bridge); // clean stop + join
}

#[test]
fn egui_repaint_requests_without_panicking() {
    EguiRepaint(egui::Context::default()).request();
}

#[test]
fn the_windowless_repaint_does_nothing_by_contract() {
    // §8.5: `yog serve` has no event loop to wake — the whole impl.
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
