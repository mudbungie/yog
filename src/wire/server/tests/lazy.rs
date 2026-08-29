//! The listener writes a frame **as it is produced**, not when the answer ends
//! (bl-73e7) — the whole of what minting the follow lane cost this file.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use serde_json::{Value, json};
use tempfile::TempDir;

use super::super::{Answerer, Listener, Peer};
use crate::registry::presence::Presence;
use crate::test_support::wire::{material, mint};
use crate::wire::client::Seat;
use crate::wire::material::Role;

/// **A frame is written when it is produced, not when the answer is finished**
/// (bl-73e7). That is the whole of what minting the follow lane cost this
/// listener, and it is the property a held read is impossible without: an
/// answer that had to be materialized first could never be one that answers *as
/// the world changes*.
///
/// The witness is a peer reading the first frame of an answer whose **second**
/// has not been produced yet — which nothing can do if the loop drains the
/// answer before it writes.
#[test]
fn a_frame_is_written_as_it_is_produced_rather_than_when_the_answer_ends() {
    /// An answerer whose second frame waits for the test to release it.
    struct Paced(Arc<AtomicUsize>);

    impl Answerer for Paced {
        fn answer(&self, _peer: &Peer, _request: Value) -> Box<dyn Iterator<Item = Value>> {
            let gate = Arc::clone(&self.0);
            Box::new((0..2).map(move |n| {
                while gate.load(Ordering::Relaxed) <= n {
                    std::thread::yield_now();
                }
                json!({"ok": true, "kind": "echo", "seq": n})
            }))
        }
    }

    let tmp = TempDir::new().expect("tmp");
    mint(tmp.path());
    let gate = Arc::new(AtomicUsize::new(1));
    let listener = Listener::bind(
        &material(
            tmp.path(),
            Role::Server,
            crate::test_support::wire::EPHEMERAL,
        ),
        Arc::new(Paced(Arc::clone(&gate))),
        Presence::default(),
    )
    .expect("bind");
    let seat = Seat::open(&material(
        tmp.path(),
        Role::Window,
        &crate::wire::loopback(&listener.address()),
    ))
    .expect("seat");

    let mut seen = 0usize;
    seat.followed(&json!({"op": "ops"}), &mut |landed| {
        // The first frame reached this peer while the second was still gated:
        // an eager answerer would have been parked in its own iterator and
        // written nothing at all.
        assert!(landed.is_err(), "the fixture's echo is nobody's Reply");
        seen += 1;
        gate.store(2, Ordering::Relaxed);
        true
    })
    .expect("the stream ends cleanly");
    assert_eq!(seen, 2, "and both frames arrive, in order");
}
