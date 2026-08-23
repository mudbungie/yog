//! The follow lane, end to end over loopback mTLS — the frame's hand-off, the
//! held read, and what happens when the far end is not there.
//!
//! The engine here is a stand-in [`Answerer`]: what is under test is the
//! *lane*, and `boundary::follow::tests` is where the frames themselves are
//! argued.
//!
//! The two beats about the **two halves together** — that a held read delays
//! nothing, and that one connection carries every growth of a real tail — are
//! [`engine`], split off at §12's per-file budget on that seam: everything here
//! stands the engine in, and everything there drives the real intake over a
//! real workspace, because neither of those claims can be made by one half.

mod engine;

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use serde_json::{Value, json};
use tempfile::TempDir;

use super::*;
use crate::registry::presence::Presence;
use crate::test_support::wire::{EPHEMERAL, NO_LISTENER, material, mint};
use crate::watch::NoRepaint;
use crate::wire::material::Role;
use crate::wire::server::{Answerer, Listener};

/// An engine that answers every request with a fixed stream of frames and then
/// terminates it — the shape a follow read has, with the waiting taken out.
struct Says(Vec<Value>);

impl Answerer for Says {
    fn answer(
        &self,
        _client: &crate::registry::Client,
        _request: Value,
    ) -> Box<dyn Iterator<Item = Value>> {
        Box::new(self.0.clone().into_iter())
    }
}

/// An engine whose answer is **paced by the test**: frame `i` is written only
/// once `gate` has passed `i`, and the stream ends only once it has passed them
/// all. That is what a held read is — the engine writes when the world moves,
/// not when the peer asks — and gating the terminator too is what lets a beat
/// observe a frame *before* the end rather than racing it.
struct Paced {
    gate: Arc<AtomicUsize>,
    frames: Vec<Value>,
}

impl Answerer for Paced {
    fn answer(
        &self,
        _client: &crate::registry::Client,
        _request: Value,
    ) -> Box<dyn Iterator<Item = Value>> {
        let gate = Arc::clone(&self.gate);
        let frames = self.frames.clone();
        Box::new((0..=frames.len()).filter_map(move |i| {
            while gate.load(Ordering::Relaxed) <= i {
                std::thread::yield_now();
            }
            frames.get(i).cloned()
        }))
    }
}

/// Settle until the seat's fold reads `text`, bounded — a wait on another
/// thread's publish, never an assertion about how long anything took. It waits
/// for the *value* rather than for any value, because the seat already holds
/// the previous frame and "something landed" would be true of that one.
pub(super) fn awaited(tail: &mut Tail, text: &str) -> bool {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
    loop {
        if declared(tail).and_then(|s| s.text).as_deref() == Some(text) {
            return true;
        }
        if std::time::Instant::now() > deadline {
            return false;
        }
        std::thread::yield_now();
    }
}

/// One follow frame's envelope, carrying `text` as the whole tail.
pub(super) fn frame(text: &str) -> Value {
    json!({"ok": true, "kind": "follow", "stream": {"text": text, "delta": "text"}})
}

/// The question a frame declares: follow `alba`'s `c-1`.
pub(super) fn subject() -> Value {
    crate::boundary::codec::encode(&crate::boundary::Gesture::Ask(
        crate::boundary::Query::Follow {
            workspace: "alba".to_owned(),
            agent: "c-1".to_owned(),
        },
    ))
}

/// A bound listener answering `says`, and a lane seated on it.
fn wired(tmp: &TempDir, says: Vec<Value>) -> (Listener, Lane, Tail) {
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
    let (tail, end) = pair();
    (listener, Lane::new(seat, end, Arc::new(NoRepaint)), tail)
}

/// One frame's dance at the seat: settle, then declare and read — the order
/// [`AppModel::refresh`](crate::AppModel::refresh) keeps.
pub(super) fn declared(tail: &mut Tail) -> Option<crate::git_tree::Stream> {
    tail.settle();
    tail.ask(&subject())
}

/// Bring the lane onto the subject. A declaration reaches it on the settle
/// **after** the frame that first made it — the `Link` discipline exactly — so
/// watching a conversation is two frames, not one.
pub(super) fn watching(tail: &mut Tail) -> Option<crate::git_tree::Stream> {
    declared(tail);
    declared(tail)
}

/// **The whole lane in one beat**: a frame declares the conversation it is
/// watching, the lane holds the line, and **each** frame the engine writes
/// reaches the seat as it is written — then the stream ends and the seat is
/// told, so it falls back to the pull fold rather than standing on a tail that
/// stopped growing.
#[test]
fn every_frame_the_engine_writes_reaches_the_seat_and_the_end_is_told() {
    let tmp = TempDir::new().expect("tmp");
    mint(tmp.path());
    let gate = Arc::new(AtomicUsize::new(0));
    let listener = Listener::bind(
        &material(tmp.path(), Role::Server, EPHEMERAL),
        Arc::new(Paced {
            gate: Arc::clone(&gate),
            frames: vec![frame("the first "), frame("the first half.")],
        }),
        Presence::default(),
    )
    .expect("bind");
    let seat = crate::wire::client::Seat::open(&material(
        tmp.path(),
        Role::Window,
        &crate::wire::loopback(&listener.address()),
    ))
    .expect("seat");
    let (mut tail, end) = pair();
    let mut lane = Lane::new(seat, end, Arc::new(NoRepaint));

    assert_eq!(watching(&mut tail), None, "nothing has crossed yet");
    let held = std::thread::spawn(move || lane.turn());

    gate.store(1, Ordering::Relaxed);
    assert!(
        awaited(&mut tail, "the first "),
        "the first frame lands while the read is still open"
    );
    gate.store(2, Ordering::Relaxed);
    assert!(
        awaited(&mut tail, "the first half."),
        "and so does the next — a frame carries the whole fold, so the newest wins"
    );

    gate.store(3, Ordering::Relaxed);
    assert!(held.join().expect("the turn ends"), "frames landed");
    assert_eq!(
        declared(&mut tail),
        None,
        "the stream is over and the seat is back on the pull fold"
    );
}

/// **A lane that cannot dial is the pull path, not a broken chat.** The turn
/// answers that nothing landed — which is what paces the re-ask — and the seat
/// holds no tail, so the committed transcript's own fold is what paints.
#[test]
fn a_lane_with_no_engine_lands_nothing_and_says_so() {
    let tmp = TempDir::new().expect("tmp");
    mint(tmp.path());
    let seat = crate::wire::client::Seat::open(&material(tmp.path(), Role::Window, NO_LISTENER))
        .expect("seat");
    let (mut tail, end) = pair();
    let mut lane = Lane::new(seat, end, Arc::new(NoRepaint));
    watching(&mut tail);
    assert!(!lane.turn(), "no engine, no frames");
    assert_eq!(declared(&mut tail), None);
}

/// A lane nobody has aimed asks nothing at all — the resting state of a window
/// with no conversation open, and the same `false` a failed dial answers, so
/// the caller paces both the same way.
#[test]
fn a_lane_with_no_subject_asks_nothing() {
    let tmp = TempDir::new().expect("tmp");
    let (_listener, mut lane, _tail) = wired(&tmp, vec![frame("unread")]);
    assert!(!lane.turn(), "nothing declared, nothing asked");
}

/// **An answer of another kind ends the read.** A refusal, or a reply the codec
/// carried faithfully but this lane did not ask for, is a defect rather than a
/// state — so the lane hangs up and the seat falls back rather than painting
/// something it cannot read.
#[test]
fn an_answer_of_another_kind_ends_the_read() {
    let tmp = TempDir::new().expect("tmp");
    let (_listener, mut lane, mut tail) = wired(
        &tmp,
        vec![json!({"ok": false, "error": "no such conversation"})],
    );
    watching(&mut tail);
    assert!(!lane.turn(), "nothing landed");
    assert_eq!(declared(&mut tail), None);
}

/// **The question is its own key.** A frame answering the conversation the seat
/// was watching a moment ago cannot land on the one it is watching now — the
/// subject change drops what landed, with nothing to say "stop".
#[test]
fn a_frame_for_the_conversation_just_left_lands_nowhere() {
    let (mut tail, end) = pair();
    let other = crate::boundary::codec::encode(&crate::boundary::Gesture::Ask(
        crate::boundary::Query::Follow {
            workspace: "alba".to_owned(),
            agent: "c-2".to_owned(),
        },
    ));
    // Standing on c-1, and a fold lands for it.
    watching(&mut tail);
    end.publish(
        &subject().to_string(),
        Some(crate::git_tree::Stream {
            text: Some("c-1 is talking".to_owned()),
            ..crate::git_tree::Stream::default()
        }),
    );
    assert!(
        declared(&mut tail).is_some(),
        "the fold is for this subject"
    );

    // The operator moves to c-2. What c-1 says next reaches nobody.
    tail.settle();
    tail.ask(&other);
    end.publish(
        &subject().to_string(),
        Some(crate::git_tree::Stream {
            text: Some("c-1 is still talking".to_owned()),
            ..crate::git_tree::Stream::default()
        }),
    );
    tail.settle();
    assert_eq!(tail.ask(&other), None, "and c-2 has said nothing yet");
}

/// **A lane whose window is gone stops asking.** The lane thread is not joined
/// on drop, so this is what ends it: the subject channel disconnecting is read
/// as no subject, which is the same resting state as a window that has nothing
/// open.
#[test]
fn a_lane_whose_frame_end_is_gone_follows_nothing() {
    let (mut tail, mut end) = pair();
    watching(&mut tail);
    assert!(end.standing().is_some(), "a live frame end has a subject");
    drop(tail);
    assert!(end.standing().is_none(), "and a dead one has none, forever");
}

/// The lane's own thread, started and stopped — the `Drop` that signals and
/// unparks without joining, which is the one thread in yog that does not.
#[test]
fn the_lane_thread_starts_and_stops() {
    let tmp = TempDir::new().expect("tmp");
    let (_listener, lane, _tail) = wired(&tmp, vec![frame("nobody asked")]);
    drop(lane.start());
}
