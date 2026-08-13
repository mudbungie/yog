//! The shared-instance fan-out (bl-908c): routing is pure over a message and a
//! root, and the two live-instance properties — a nested root survives its
//! parent's retirement, and a root outlives a *second* watcher's drop — are
//! proved against the real backend, because both are exactly what one instance
//! per root used to make impossible to get wrong.

use super::super::hub::addressed;
use super::super::{ChangeKind, RootKind, Watcher};
use super::{poll_until, workspace};
use notify::event::{CreateKind, EventKind};
use notify::{Event, Result as NotifyResult};
use std::path::{Path, PathBuf};
use std::time::Duration;

const WAIT: Duration = Duration::from_secs(5);
const STEP: Duration = Duration::from_millis(50);

fn created(paths: &[&str]) -> Event {
    Event {
        kind: EventKind::Create(CreateKind::File),
        paths: paths.iter().map(PathBuf::from).collect(),
        attrs: notify::event::EventAttributes::new(),
    }
}

fn paths_of(res: &NotifyResult<Event>) -> Vec<PathBuf> {
    res.as_ref().map(|e| e.paths.clone()).unwrap_or_default()
}

#[test]
fn an_event_reaches_only_the_roots_that_contain_it() {
    let msg: NotifyResult<Event> = Ok(created(&["/a/one/f", "/b/two/f"]));
    assert_eq!(
        paths_of(&addressed(Path::new("/a"), &msg).unwrap()),
        vec![PathBuf::from("/a/one/f")],
        "narrowed to the paths under the root"
    );
    assert!(
        addressed(Path::new("/c"), &msg).is_none(),
        "a root the event never touched hears nothing"
    );
}

#[test]
fn a_rename_pair_inside_one_root_stays_a_pair() {
    let msg: NotifyResult<Event> = Ok(created(&["/a/from", "/a/to"]));
    assert_eq!(
        paths_of(&addressed(Path::new("/a"), &msg).unwrap()).len(),
        2,
        "both ends survive, so the (from, to) fold still sees a pair"
    );
}

#[test]
fn an_instance_wide_loss_reaches_every_root() {
    let mut event = Event::new(EventKind::Any);
    event.attrs.set_flag(notify::event::Flag::Rescan);
    let rescan = Ok(event);
    assert!(
        addressed(Path::new("/nowhere"), &rescan).is_some(),
        "a rescan flag is the instance's loss, not one root's"
    );
    let err: NotifyResult<Event> = Err(notify::Error::generic("boom"));
    assert!(
        addressed(Path::new("/nowhere"), &err).unwrap().is_err(),
        "and so is a backend error"
    );
}

#[test]
fn retiring_a_parent_root_leaves_a_nested_root_still_hearing() {
    // `notify`'s inotify backend unwatches a whole subtree, so dropping the
    // enumeration-root watcher takes the workspace watcher's descriptors with
    // it. The hub must put them back — otherwise the nested watcher is deaf and
    // says nothing about it (§7.3).
    let (_guard, parent) = workspace();
    let nested = parent.join("nested");
    std::fs::create_dir_all(&nested).unwrap();
    let outer = Watcher::with_kind(&parent, RootKind::WorkspacesRoot).unwrap();
    let inner = Watcher::with_kind(&nested, RootKind::Workspace).unwrap();
    drop(outer);
    std::fs::create_dir_all(nested.join("steps").join("c-1")).unwrap();
    std::fs::write(nested.join("steps").join("c-1").join("request.json"), b"{}").unwrap();
    let found = poll_until(
        || {
            let changes = inner.tick();
            changes
                .iter()
                .any(|c| c.kind == ChangeKind::Touched)
                .then_some(changes)
        },
        WAIT,
        STEP,
    );
    assert!(found.is_some(), "the nested watch was re-armed");
}

#[test]
fn a_root_two_watchers_share_survives_the_first_drop() {
    let (_guard, root) = workspace();
    let first = Watcher::with_kind(&root, RootKind::Workspace).unwrap();
    let second = Watcher::with_kind(&root, RootKind::WorkspacesRoot).unwrap();
    drop(first);
    std::fs::create_dir_all(root.join("steps").join("c-1")).unwrap();
    std::fs::write(root.join("steps").join("c-1").join("request.json"), b"{}").unwrap();
    let found = poll_until(
        || {
            let changes = second.tick();
            (!changes.is_empty()).then_some(changes)
        },
        WAIT,
        STEP,
    );
    assert!(found.is_some(), "the surviving subscriber keeps its arm");
}
