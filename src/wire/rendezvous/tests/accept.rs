//! **The punch port's one acceptor** (bl-5276): what lands on the port with
//! no window toward it is served and said, a stream that fails mTLS is
//! dropped and said, the port outlives a window, and a stream from a live
//! window's peer is that window's.

use super::bench::{Bench, bench, loopback, until};
use super::{Punch, WAIT};
use crate::dht::tests::fake::Mood;
use crate::test_support::wire::{material, mint};
use crate::wire::material::Role;
use crate::wire::{hello, tls};
use rustls::pki_types::ServerName;
use rustls::{ClientConnection, StreamOwned};
use std::net::{Ipv4Addr, TcpStream};
use std::sync::atomic::Ordering::Relaxed;
use std::time::Duration;

const P: &str = "yog: rendezvous:";

fn accepted() -> String {
    format!("{P} stream accepted on the punch port (v4) — served")
}

fn dial(b: &Bench) -> TcpStream {
    TcpStream::connect(loopback(b.port)).expect("connect")
}

/// A call toward a port nobody listens on, opened and its window told to the
/// acceptor: the SYNs go nowhere, so only what the test dials lands.
fn open_call_to_nobody(b: &Bench, nonce: u64) {
    let stats = b.engine.stats();
    assert!(until(|| stats.polls.load(Relaxed) >= 1, WAIT));
    let closed = Punch::bind(0).expect("a port").port();
    assert!(b.write_inbox(1, b.call(nonce, closed)) >= 1);
    b.clock.advance(Duration::from_secs(15));
    let opened = format!("{P} call nonce {nonce} opened");
    assert!(until(|| b.said(&opened) == 1, WAIT), "{:?}", b.heard());
    std::thread::sleep(Duration::from_millis(100));
}

#[test]
fn a_stream_between_calls_is_served_and_said_once_per_burst() {
    let b = bench(Mood::Answer, None);
    let (first, second) = (dial(&b), dial(&b));
    drop(b.ask_over(first));
    drop(b.ask_over(second));
    let stats = b.engine.stats();
    assert_eq!(b.said(&accepted()), 1, "{:?}", b.heard());
    let standing = stats.standing();
    assert_eq!((standing.accepted, standing.served), (2, 2));
    assert_eq!(standing.calls, 0, "nobody called");
}

#[test]
fn a_stream_that_fails_mtls_is_dropped_and_said_once() {
    let b = bench(Mood::Answer, None);
    let stranger = tempfile::TempDir::new().expect("tmp");
    mint(stranger.path());
    let config = tls::client_config(&material(stranger.path(), Role::Client, "127.0.0.1:1"))
        .expect("client tls");
    let name = ServerName::IpAddress(Ipv4Addr::LOCALHOST.into());
    let conn = ClientConnection::new(config, name).expect("conn");
    let tcp = dial(&b);
    tcp.set_read_timeout(Some(Duration::from_secs(3)))
        .expect("timeout");
    let mut tls = StreamOwned::new(conn, tcp);
    let _ = hello::state(&mut tls);
    let _ = hello::confirm(&mut tls);
    drop(tls);
    let refused = format!("{P} stream on the punch port (v4) refused at the handshake — dropped");
    assert!(until(|| b.said(&refused) == 1, WAIT), "{:?}", b.heard());
    assert_eq!(b.said(&accepted()), 1);
    assert_eq!(b.engine.stats().standing().accepted, 1);
}

#[test]
fn the_listener_survives_a_punch_window_ending() {
    let b = bench(Mood::Answer, None);
    open_call_to_nobody(&b, 8);
    let expired = format!("{P} punch for nonce 8 expired after 3s with no stream");
    assert!(until(|| b.said(&expired) == 1, WAIT), "{:?}", b.heard());
    drop(b.ask_over(dial(&b)));
    let standing = b.engine.stats().standing();
    assert_eq!((standing.accepted, standing.punched), (1, 0));
}

#[test]
fn a_stream_from_a_live_windows_peer_is_the_windows() {
    let b = bench(Mood::Answer, None);
    open_call_to_nobody(&b, 6);
    drop(b.ask_over(dial(&b)));
    let landed = format!("{P} punch for nonce 6 landed 1 stream(s) (1 v4)");
    assert_eq!(b.said(&landed), 1, "{:?}", b.heard());
    let stats = b.engine.stats();
    assert_eq!(
        (stats.standing().punched, stats.standing().accepted),
        (1, 0)
    );
    // The window has stopped taking but is not yet past its hint: the
    // acceptor is handed the stream back and serves it itself.
    drop(b.ask_over(dial(&b)));
    assert_eq!(b.said(&accepted()), 1, "{:?}", b.heard());
    assert_eq!(stats.standing().accepted, 1);
}
