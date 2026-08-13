//! Loss announcements: the ways the backend itself says "events were lost" —
//! a kernel queue overflow, a watch-descriptor ceiling, a root inode replaced
//! under a live watch — and the desync lead that turns any of them into an
//! ordinary whole-root re-derivation (§7.2/§7.3).

use super::*;
use notify::ErrorKind;
use std::fs;
use std::sync::mpsc;
use tempfile::tempdir;

#[test]
fn scenario_queue_overflow_is_announced_not_swallowed() {
    // inotify's kernel event queue is bounded (`fs.inotify.max_queued_events`).
    // On overflow the kernel sets IN_Q_OVERFLOW and notify forwards it as a
    // rescan-flagged event carrying NO path — so the old `for path in paths`
    // ingest iterated zero times and the loss vanished without trace, leaving
    // the 15 s sweep to repair it silently. It must surface as a Desynced change
    // on the root: "re-read all of this".
    let (tx, rx) = mpsc::channel();
    tx.send(Ok(
        Event::new(EventKind::Other).set_flag(notify::event::Flag::Rescan)
    ))
    .unwrap();
    drop(tx);
    let mut raw = Vec::new();
    assert!(
        drain(&rx, &mut raw),
        "the rescan flag is a loss announcement"
    );
    assert!(raw.is_empty(), "a rescan event carries no path to ingest");
}

#[test]
fn scenario_watch_descriptor_exhaustion_is_announced_not_swallowed() {
    // `fs.inotify.max_user_watches` is a per-uid ceiling and yog watches one
    // root per workspace, recursively. When notify tries to arm a watch on a
    // directory that appears mid-tree and the ceiling is hit, inotify returns
    // ENOSPC, which notify maps to `ErrorKind::MaxFilesWatch` and sends as an
    // `Err` on the *event* channel. yog used to match `Ok(Err(_)) => {}` — it
    // threw away the only notification that a subtree had gone blind.
    let (tx, rx) = mpsc::channel();
    tx.send(Err(notify::Error::new(ErrorKind::MaxFilesWatch)))
        .unwrap();
    drop(tx);
    let mut raw = Vec::new();
    assert!(
        drain(&rx, &mut raw),
        "a backend error means events were lost"
    );
    assert!(raw.is_empty());
}

#[test]
fn a_quiet_channel_announces_no_loss() {
    let (tx, rx) = mpsc::channel::<notify::Result<Event>>();
    tx.send(Ok(Event::new(EventKind::Create(
        notify::event::CreateKind::File,
    ))
    .add_path(PathBuf::from("/r/steps/a/001/request.json"))))
        .unwrap();
    drop(tx);
    let mut raw = Vec::new();
    assert!(!drain(&rx, &mut raw), "an ordinary event is not a loss");
    assert_eq!(raw.len(), 1);
}

#[test]
fn a_desync_leads_the_tick_as_a_change_on_the_root_itself() {
    // The dissolve: a loss is not a new mechanism, it is the ordinary whole-root
    // re-derivation with the root as the changed path. That is what lets
    // `WatchSet::drain_dirty` mark the root and the frame re-derive it with no
    // second pathway at all.
    let root = Path::new("/w");
    let ordinary = vec![Change {
        path: PathBuf::from("/w/inbox/a/user-001.md"),
        kind: ChangeKind::Touched,
    }];
    let led = lead_with_desync(root, true, ordinary.clone());
    assert_eq!(led.len(), 2);
    assert_eq!(led[0].path, root);
    assert_eq!(led[0].kind, ChangeKind::Desynced);
    assert_eq!(led[1], ordinary[0], "the real changes ride along behind it");
    // A loss with nothing else to report still names the root.
    let alone = lead_with_desync(root, true, Vec::new());
    assert_eq!(alone.len(), 1);
    assert_eq!(alone[0].kind, ChangeKind::Desynced);
    // And a healthy tick is handed back untouched.
    assert_eq!(lead_with_desync(root, false, ordinary.clone()), ordinary);
}

#[test]
fn a_healthy_watcher_never_claims_a_desync() {
    let dir = tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    let watcher = Watcher::new(&root).unwrap();
    assert!(
        !watcher
            .tick()
            .iter()
            .any(|c| c.kind == ChangeKind::Desynced)
    );
}

#[test]
fn scenario_a_replaced_root_directory_leaves_a_deaf_watcher() {
    // inotify watches an INODE. A root removed and re-created under the same
    // path (a re-primed balls clone, a rebuilt workspace) leaves the armed watch
    // pointing at an unlinked inode: it will never fire again, and the path is
    // still "watched" as far as any name-keyed bookkeeping can tell. §7.3
    // promised the 2 s reconcile rebuilds it; nothing did, because the desired
    // set still contained the key. `is_stale` is the missing observation.
    let dir = tempdir().unwrap();
    let root = dir.path().join("clone");
    fs::create_dir_all(&root).unwrap();
    let watcher = Watcher::new(&root).unwrap();
    assert!(!watcher.is_stale(), "freshly armed on the live inode");
    fs::remove_dir_all(&root).unwrap();
    assert!(watcher.is_stale(), "a deleted root is a deaf watcher");
    fs::create_dir_all(&root).unwrap();
    assert!(
        watcher.is_stale(),
        "and re-creating the path does not re-arm the old inode"
    );
}
