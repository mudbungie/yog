//! **Presence is the connection's own scope** (REMOTE §5, bl-4e08): a live
//! connection is in the map while it is being answered, and out of it once it
//! is gone — with no leave verb anywhere for anyone to forget.

use super::*;
use crate::registry::presence::Presence;
use std::time::Instant;

/// An answerer that reports the presence map **as it stands mid-answer** —
/// which is the only moment "connected right now" can be asserted from the
/// outside without racing the connection's own teardown.
struct Watcher(Presence);

impl Answerer for Watcher {
    fn answer(&self, _peer: &Peer, _request: Value) -> Box<dyn Iterator<Item = Value>> {
        Box::new(std::iter::once(
            json!({"live": self.0.live().into_iter().collect::<Vec<String>>()}),
        ))
    }
}

/// The whole claim, end to end over mTLS: while a seat is being answered its
/// certificate's name is live, and once the connection is gone it is not.
#[test]
fn a_live_connection_is_present_and_a_closed_one_is_not() {
    let tmp = TempDir::new().expect("tmp");
    mint(tmp.path());
    let presence = Presence::default();
    let listener = Listener::bind(
        &material(
            tmp.path(),
            Role::Server,
            crate::test_support::wire::EPHEMERAL,
        ),
        Arc::new(Watcher(presence.clone())),
        presence.clone(),
    )
    .expect("bind");
    let seat = Seat::open(&material(tmp.path(), Role::Client, &listener.address())).expect("seat");
    let stream = seat.ask(&json!({"op": "workspaces"})).expect("answered");
    // `make wire-certs` mints `yog-client`; the identity is the leaf's own name.
    assert_eq!(stream[0]["live"], json!(["yog-client"]));
    // The seat dials per ask (REMOTE §10), so the connection is already gone —
    // its thread just has to notice. Presence is released by the guard's drop,
    // and nothing else releases it.
    let deadline = Instant::now() + Duration::from_secs(5);
    while !presence.live().is_empty() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(presence.live().is_empty(), "the connection's guard dropped");
}
