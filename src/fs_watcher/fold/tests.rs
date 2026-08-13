//! The pure fold's own tables: the coalescer's rename/delete disambiguation,
//! the `Name(Both)` split, and the classifier — each driven by a synthesized
//! `(path, EventKind)` burst rather than a real backend, which is the whole
//! point of the fold living apart from the armed watch.

use super::*;
use crate::fs_watcher::tests::workspace;
use std::fs;
use tempfile::tempdir;

#[test]
fn coalesce_drops_rename_source_with_trailing_modify() {
    // macOS atomic-rename through the pure coalesce seam: FSEvents coalesces
    // CREATED|MODIFIED|RENAMED, so the `.tmp` source's last event is a trailing
    // Modify(Data) *after* the Name — gone, no Remove, must not surface (see the
    // `classify` doc). The destination exists and survives as a single Touched.
    use notify::event::{CreateKind, DataChange};
    let (_dir, root) = workspace();
    fs::create_dir_all(root.join("steps/abc/001")).unwrap();
    let tmp = root.join("steps/abc/001/request.json.tmp");
    let final_path = root.join("steps/abc/001/request.json");
    fs::write(&final_path, b"{}").unwrap(); // destination exists post-rename
    let name = EventKind::Modify(ModifyKind::Name(RenameMode::Any));
    let modify = EventKind::Modify(ModifyKind::Data(DataChange::Content));
    let raw = vec![
        (tmp.clone(), EventKind::Create(CreateKind::File)),
        (tmp.clone(), name),
        (tmp.clone(), modify),
        (final_path.clone(), EventKind::Create(CreateKind::File)),
    ];
    let changes = coalesce(&root, RootKind::Workspace, raw);
    assert_eq!(changes.len(), 1);
    assert_eq!(changes[0].path, final_path);
    assert_eq!(changes[0].kind, ChangeKind::Touched);
}

#[test]
fn coalesce_surfaces_genuine_deletion_with_non_remove_last_event() {
    // macOS genuine deletion through the same seam: CREATED|REMOVED|MODIFIED, so
    // the last event is a trailing Modify(Data) *after* the Remove. Gone, never
    // renamed ⇒ Removed (see the `classify` doc). bl-71ee's Remove-keyed classify
    // dropped this, regressing detects_removal_under_summary on macos-14.
    use notify::event::{CreateKind, DataChange, RemoveKind};
    let (_dir, root) = workspace();
    fs::create_dir_all(root.join("agents/aa-bb/summary")).unwrap();
    let gone = root.join("agents/aa-bb/summary/001.md");
    let modify = EventKind::Modify(ModifyKind::Data(DataChange::Content));
    let raw = vec![
        (gone.clone(), EventKind::Create(CreateKind::File)),
        (gone.clone(), EventKind::Remove(RemoveKind::File)),
        (gone.clone(), modify),
    ];
    let changes = coalesce(&root, RootKind::Workspace, raw);
    assert_eq!(changes.len(), 1);
    assert_eq!(changes[0].path, gone);
    assert_eq!(changes[0].kind, ChangeKind::Removed);
}

#[test]
fn ingest_splits_name_both_into_from_and_to() {
    let mut raw = Vec::new();
    ingest(
        Event {
            kind: EventKind::Modify(ModifyKind::Name(RenameMode::Both)),
            paths: vec![PathBuf::from("/a"), PathBuf::from("/b")],
            attrs: notify::event::EventAttributes::default(),
        },
        &mut raw,
    );
    assert_eq!(raw.len(), 2);
    assert!(matches!(
        raw[0].1,
        EventKind::Modify(ModifyKind::Name(RenameMode::From))
    ));
    assert!(matches!(
        raw[1].1,
        EventKind::Modify(ModifyKind::Name(RenameMode::To))
    ));
}

#[test]
fn coalesce_drops_prior_events_when_rename_from_arrives() {
    // A `Name` event on an absent path is a rename source: its prior create is
    // dropped, nothing surfaces.
    let repo = Path::new("/r");
    let p = PathBuf::from("/r/steps/abc/001/request.json");
    let raw = vec![
        (
            p.clone(),
            EventKind::Create(notify::event::CreateKind::File),
        ),
        (
            p.clone(),
            EventKind::Modify(ModifyKind::Name(RenameMode::From)),
        ),
    ];
    assert!(coalesce(repo, RootKind::Workspace, raw).is_empty());
}

#[test]
fn classify_over_existence_and_rename() {
    // Existence is ground truth: present ⇒ Touched, whatever the history. Gone
    // and renamed ⇒ dropped (a rename source, its destination survives); gone
    // and not renamed ⇒ Removed (a genuine deletion, whatever its last event).
    let root = tempdir().unwrap();
    let present = root.path().join("x");
    fs::write(&present, b"").unwrap();
    assert_eq!(classify(false, &present), Some(ChangeKind::Touched));
    assert_eq!(classify(true, &present), Some(ChangeKind::Touched));
    let absent = Path::new("/no/such/path");
    assert_eq!(classify(true, absent), None);
    assert_eq!(classify(false, absent), Some(ChangeKind::Removed));
}
