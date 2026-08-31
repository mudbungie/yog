//! **The hand-off half**: what a watcher's tick becomes once it has fired —
//! the [`DirtySet`] a background thread marks and the frame drains, the pure
//! [`pump`] step over it (both arms), the real background [`Bridge`] end to
//! end (a disk change surfaces as a dirty root). Split from [`super`] at §12's
//! budget on the seam between **arming** a watch and **spending** what it
//! produced.

use super::super::*;
use super::{DETECT, wait_armed, wait_until, workspace};
use std::path::PathBuf;
use std::sync::Mutex;

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
