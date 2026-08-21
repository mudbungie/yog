//! The window as a wire client of loopback: it seats itself, it asks, it lands
//! decoded replies — and it never hands the frame anything it had to wait for.

use super::*;
use crate::binding::{Workspace, WorkspaceKind};
use crate::boundary::reply::Reply;
use crate::registry::presence::Presence;
use crate::state::new_snapshot_cell;
use crate::test_support::wire::{EPHEMERAL, NO_LISTENER, material, mint};
use crate::watch::NoRepaint;
use crate::wire::link::{Link, LinkEnd, pair};
use crate::wire::material::Role;
use crate::wire::server::{Answerer, Listener};
use serde_json::{Value, json};
use std::path::Path;
use tempfile::TempDir;

/// An engine that answers with a fixed stream, so the asker's decode and its
/// stream-shape rules are the subject rather than the boundary's.
struct Says(Vec<serde_json::Value>);

impl Answerer for Says {
    fn answer(&self, _client: &crate::registry::Client, _request: serde_json::Value) -> Vec<Value> {
        self.0.clone()
    }
}

/// One frame, in [`AppModel::refresh`](crate::AppModel::refresh)'s order:
/// settle, then ask. A question therefore reaches the asker on the frame after
/// the one that first painted it.
fn frame(link: &mut Link, question: &serde_json::Value) -> Option<crate::wire::link::Landed> {
    link.settle();
    link.ask(question)
}

/// A snapshot enumerating `names` as yog-named workspaces under `root`.
fn snapshot_of(root: &Path, names: &[&str]) -> SnapshotCell {
    let mut snap = crate::app::Snapshot::empty(0);
    snap.workspaces = names
        .iter()
        .map(|name| Workspace {
            path: root.join(name),
            kind: WorkspaceKind::Named {
                name: (*name).to_owned(),
            },
        })
        .collect();
    new_snapshot_cell(Arc::new(snap))
}

/// A bound listener answering `says`, and an asker seated on it as the window.
fn wired(tmp: &TempDir, says: Vec<serde_json::Value>, names: &[&str]) -> (Listener, Asker, Link) {
    mint(tmp.path());
    let listener = Listener::bind(
        &material(tmp.path(), Role::Server, EPHEMERAL),
        Arc::new(Says(says)),
        Presence::default(),
    )
    .expect("bind");
    let seat = crate::wire::client::Seat::open(&material(
        tmp.path(),
        Role::Window,
        &crate::wire::loopback(&listener.address()),
    ))
    .expect("seat");
    let (link, end) = pair();
    let asker = Asker::new(
        seat,
        end,
        snapshot_of(tmp.path(), names),
        tmp.path().to_path_buf(),
        Arc::new(NoRepaint),
    );
    (listener, asker, link)
}

/// The whole read path in one test: the window seats its own leaf, asks over
/// loopback mTLS presenting that leaf, and the frame reads a decoded `Reply`
/// it never waited for.
#[test]
fn the_window_seats_itself_and_paints_a_decoded_reply() {
    let tmp = TempDir::new().expect("tmp");
    let (_listener, mut asker, mut link) = wired(
        &tmp,
        vec![json!({"ok": true, "kind": "clients", "rows": [
            {"client": "laptop", "present": true, "tools": []}]})],
        &["home", "away"],
    );
    // Nothing declared: a pass still seats the window, and asks nothing.
    assert_eq!(asker.pass(), 0);
    let window = crate::registry::window();
    assert_eq!(
        crate::registry::registered(tmp.path(), &window),
        ["away".to_owned(), "home".to_owned()].into_iter().collect(),
        "the engine seats its own window in every workspace it enumerates"
    );

    let question = json!({"op": "clients", "workspace": "home"});
    assert!(frame(&mut link, &question).is_none(), "nothing landed yet");
    frame(&mut link, &question);
    assert_eq!(asker.pass(), 1);
    let Some(Ok(Reply::Clients(rows))) = frame(&mut link, &question) else {
        panic!("the reply decodes to the roster");
    };
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].client, "laptop");
    assert!(rows[0].present);
}

/// A registration already written is one directory read, not a second write —
/// which is what makes the seating free on every pass.
#[test]
fn a_seating_that_is_already_there_writes_nothing() {
    let tmp = TempDir::new().expect("tmp");
    let (_listener, mut asker, _link) = wired(&tmp, Vec::new(), &["home"]);
    asker.pass();
    let seat = crate::registry::registrations(tmp.path(), &crate::registry::window()).join("home");
    let before = std::fs::metadata(&seat).expect("seated").modified().ok();
    asker.pass();
    assert_eq!(
        std::fs::metadata(&seat).expect("seated").modified().ok(),
        before
    );
}

/// An engine that ends the stream without saying anything is a refusal with a
/// sentence, not an empty roster — "not answered" and "answered nothing" are
/// different facts and a surface must not read them alike.
#[test]
fn a_stream_with_no_frames_is_a_refusal() {
    let tmp = TempDir::new().expect("tmp");
    let (_listener, mut asker, mut link) = wired(&tmp, Vec::new(), &[]);
    let question = json!({"op": "clients", "workspace": "home"});
    frame(&mut link, &question);
    frame(&mut link, &question);
    assert_eq!(asker.pass(), 1);
    let Some(Err(said)) = frame(&mut link, &question) else {
        panic!("a refusal");
    };
    assert!(said.contains("without answering"), "{said}");
}

/// An engine's own refusal reaches the frame verbatim, and bytes the codec
/// cannot read reach it as the codec's sentence — one `Err`, either way.
#[test]
fn a_refusal_and_an_unreadable_answer_both_land_as_one_err() {
    let tmp = TempDir::new().expect("tmp");
    let (_listener, mut asker, mut link) = wired(
        &tmp,
        vec![json!({"ok": false, "error": "unknown workspace \"home\""})],
        &[],
    );
    let question = json!({"op": "clients", "workspace": "home"});
    frame(&mut link, &question);
    frame(&mut link, &question);
    asker.pass();
    assert_eq!(
        frame(&mut link, &question),
        Some(Err("unknown workspace \"home\"".to_owned()))
    );

    let other = TempDir::new().expect("tmp");
    let (_l2, mut a2, mut k2) = wired(&other, vec![json!({"kind": "no-such-reply"})], &[]);
    frame(&mut k2, &question);
    frame(&mut k2, &question);
    a2.pass();
    let Some(Err(said)) = frame(&mut k2, &question) else {
        panic!("a refusal");
    };
    assert!(said.contains("no-such-reply"), "{said}");
}

/// An engine that is not there is a sentence on the surface, never a frame
/// that waited: the transport's failure is the same `Err` a refusal is.
#[test]
fn a_dead_engine_lands_a_sentence_rather_than_blocking() {
    let tmp = TempDir::new().expect("tmp");
    mint(tmp.path());
    // Aimed where nothing can be listening, rather than at a listener this test
    // just dropped — see `NO_LISTENER`.
    let seat = crate::wire::client::Seat::open(&material(tmp.path(), Role::Window, NO_LISTENER))
        .expect("seat");
    let (mut link, end) = pair();
    let mut asker = Asker::new(
        seat,
        end,
        snapshot_of(tmp.path(), &[]),
        tmp.path().to_path_buf(),
        Arc::new(NoRepaint),
    );
    let question = json!({"op": "clients", "workspace": "home"});
    frame(&mut link, &question);
    frame(&mut link, &question);
    assert_eq!(asker.pass(), 1);
    let Some(Err(said)) = frame(&mut link, &question) else {
        panic!("a refusal");
    };
    assert!(said.starts_with("connect "), "{said}");
}

/// A window that has gone away stops the pass where it stands — there is
/// nothing left to answer.
#[test]
fn a_dropped_window_ends_the_pass() {
    let tmp = TempDir::new().expect("tmp");
    let (_listener, mut asker, mut link) = wired(&tmp, Vec::new(), &[]);
    let question = json!({"op": "clients", "workspace": "home"});
    frame(&mut link, &question);
    frame(&mut link, &question);
    drop(link);
    assert_eq!(asker.pass(), 0, "nowhere to publish");
}

/// The thread shape: it runs passes until dropped, and dropping it stops and
/// joins — the searcher's shutdown exactly.
#[test]
fn the_thread_runs_until_dropped() {
    let tmp = TempDir::new().expect("tmp");
    let (_listener, asker, mut link) = wired(&tmp, Vec::new(), &["home"]);
    let question = json!({"op": "clients", "workspace": "home"});
    frame(&mut link, &question);
    frame(&mut link, &question);
    let thread = asker.start();
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    loop {
        if frame(&mut link, &question).is_some() {
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "the asker never asked"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
    drop(thread);
    assert!(
        crate::registry::registrations(tmp.path(), &crate::registry::window())
            .join("home")
            .is_file()
    );
}

/// A link end can only be taken once, which is what makes one asker per engine
/// a fact rather than a convention.
#[test]
fn a_link_end_is_the_asker_it_was_minted_for() {
    let (_link, mut end): (Link, LinkEnd) = pair();
    assert!(end.standing().is_empty());
}
