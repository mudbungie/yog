//! The act path end to end: a gesture the frame posted crosses loopback mTLS,
//! and its decoded receipt lands under the ticket the click was given.

use super::*;
use crate::boundary::reply::Reply;
use crate::registry::presence::Presence;
use crate::test_support::wire::{EPHEMERAL, NO_LISTENER, material, mint};
use crate::watch::NoRepaint;
use crate::wire::material::Role;
use crate::wire::post::{Post, pair};
use crate::wire::server::{Answerer, Listener};
use serde_json::{Value, json};
use tempfile::TempDir;

/// An engine that answers every request with a fixed stream, so the poster's
/// send and decode are the subject rather than the boundary's.
struct Says(Vec<Value>);

impl Answerer for Says {
    fn answer(
        &self,
        _peer: &crate::registry::Peer,
        _request: Value,
    ) -> Box<dyn Iterator<Item = Value>> {
        Box::new(self.0.clone().into_iter())
    }
}

/// A bound listener answering `says`, and a window's poster seated on it.
fn wired(tmp: &TempDir, says: Vec<Value>) -> (Listener, Poster, Post) {
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
    let (post, outbox) = pair();
    (
        listener,
        Poster::new(
            crate::wire::dial::Dial::of(seat),
            outbox,
            Arc::new(NoRepaint),
        ),
        post,
    )
}

/// The whole act path in one test: a click posts a gesture, the poster sends it
/// over the real socket presenting the window leaf, and the receipt the frame
/// reads is a decoded `Reply` it never waited for.
#[test]
fn a_posted_act_crosses_the_wire_and_its_receipt_lands_under_its_ticket() {
    let tmp = TempDir::new().expect("tmp");
    let (_listener, mut poster, mut post) = wired(&tmp, vec![json!({"ok": true, "kind": "acked"})]);
    let ticket = post.send(&json!({"op": "seen", "workspace": "home", "agent": "c-1"}));
    assert!(post.settle().is_empty(), "nothing has crossed yet");
    assert!(poster.pass(), "one act, sent and answered");
    assert_eq!(post.settle(), vec![ticket]);
    assert_eq!(post.receipt(ticket), Some(Ok(Reply::Acked)));
}

/// An engine that is not there is a sentence under the ticket, never a frame
/// that waited — the transport's failure being the same one `Err` a refusal is.
#[test]
fn a_dead_engine_lands_a_sentence_rather_than_blocking() {
    let tmp = TempDir::new().expect("tmp");
    mint(tmp.path());
    // Aimed where nothing can be listening, rather than at a listener this test
    // just dropped — see `NO_LISTENER`.
    let seat = crate::wire::client::Seat::open(&material(tmp.path(), Role::Window, NO_LISTENER))
        .expect("seat");
    let (mut post, outbox) = pair();
    let mut poster = Poster::new(
        crate::wire::dial::Dial::of(seat),
        outbox,
        Arc::new(NoRepaint),
    );
    let ticket = post.send(&json!({"op": "scan", "workspace": "home"}));
    assert!(poster.pass());
    post.settle();
    let Some(Err(said)) = post.receipt(ticket) else {
        panic!("a refusal");
    };
    assert!(said.starts_with("connect "), "{said}");
}

/// **The channel is the thread.** No stop flag, no unpark and no join: the loop
/// ends when the window's end of the outbox drops, which is what makes the
/// handle safe to hold and safe to forget.
#[test]
fn the_thread_runs_until_the_window_drops() {
    let tmp = TempDir::new().expect("tmp");
    let (_listener, poster, mut post) = wired(&tmp, vec![json!({"ok": true, "kind": "acked"})]);
    let ticket = post.send(&json!({"op": "seen", "workspace": "home", "agent": "c-1"}));
    let thread = poster.start();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    loop {
        post.settle();
        if post.receipt(ticket).is_some() {
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "the act never crossed"
        );
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    drop(post);
    thread.join().expect("the poster ends when nobody can post");
}
