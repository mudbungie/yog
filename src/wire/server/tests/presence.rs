//! **Presence is the connection's own scope** (REMOTE §5, bl-4e08): a live
//! connection is in the map while it is being answered, and out of it once it
//! is gone — with no leave verb anywhere for anyone to forget.

use super::*;
use crate::registry::presence::Presence;
use crate::wire::frame;
use rustls::ClientConnection;
use rustls::pki_types::ServerName;
use std::net::{Ipv4Addr, TcpStream};
use std::path::Path;
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

/// **A peer that vanished without a FIN is reaped** (bl-1421): the read that
/// hears nothing for the idle bound *is* the connection ending, so the guard
/// drops and the thread returns while the socket is still ESTABLISHED and this
/// scope still holds the client end of it. Without the bound the read blocks
/// forever, the roster says `present` for a box that is gone, and the thread
/// is never returned.
///
/// The idle bound is named short here rather than slept for real — the
/// production two minutes is `IDLE_TIMEOUT`, wired at the accept loop's one
/// call, and the shape is `Mailbox::holding`'s.
#[test]
fn a_peer_that_goes_silent_is_reaped() {
    let tmp = TempDir::new().expect("tmp");
    mint(tmp.path());
    let presence = Presence::default();
    let config = crate::wire::tls::server_config(&material(
        tmp.path(),
        Role::Server,
        crate::test_support::wire::EPHEMERAL,
    ))
    .expect("config");
    let tcp = TcpListener::bind("127.0.0.1:0").expect("bind");
    let address = tcp.local_addr().expect("addr").to_string();
    let watcher = Watcher(presence.clone());
    let held = presence.clone();
    let served = std::thread::spawn(move || {
        let (stream, _) = tcp.accept().expect("accept");
        serve(stream, &config, &watcher, &held, Duration::from_millis(50));
    });
    let mut tls = client(tmp.path(), &address);
    frame::write_value(&mut tls, &json!({"protocol": crate::wire::hello::PROTOCOL}))
        .expect("preface");
    frame::write_value(&mut tls, &json!({"op": "workspaces"})).expect("ask");
    frame::read_value(&mut tls).expect("read").expect("preface");
    let answer = frame::read_value(&mut tls).expect("read").expect("answer");
    // The guard was taken: the answerer saw its own connection in the map.
    assert_eq!(answer["live"], json!(["yog-client"]));
    // Now say nothing, and never hang up. `tls` lives past both assertions.
    let deadline = Instant::now() + Duration::from_secs(5);
    while !presence.live().is_empty() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(presence.live().is_empty(), "the silent peer was reaped");
    drop(tls);
    served.join().expect("the serve thread returned");
}

/// A hand-built client of `dir`'s CA pointed at `address`, kept rather than
/// spent: a [`Seat`](crate::test_support::seat::Seat) closes its connection
/// with the ask, and this test is about a connection nobody closes.
fn client(dir: &Path, address: &str) -> StreamOwned<ClientConnection, TcpStream> {
    let config =
        crate::wire::tls::client_config(&material(dir, Role::Client, address)).expect("config");
    let conn = ClientConnection::new(config, ServerName::IpAddress(Ipv4Addr::LOCALHOST.into()))
        .expect("tls");
    let tcp = TcpStream::connect(address).expect("connect");
    tcp.set_read_timeout(Some(Duration::from_secs(10)))
        .expect("timeout");
    StreamOwned::new(conn, tcp)
}
