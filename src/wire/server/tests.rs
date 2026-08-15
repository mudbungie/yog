//! The listener, end to end over loopback — and what an unauthenticated peer
//! gets instead of an answer.

use super::*;
use crate::test_support::wire::{material, mint};
use crate::wire::client::Seat;
use crate::wire::material::Role;
use serde_json::json;
use std::sync::atomic::AtomicUsize;
use tempfile::TempDir;

/// An answerer that counts what it was asked and echoes it back inside a
/// reply-shaped envelope — the boundary's own shape without the boundary.
struct Echo {
    asked: Arc<AtomicUsize>,
    /// How many frames one answer is. `1` is today's every answer; more than
    /// one is the follow-class shape, which the framing already carries.
    chunks: usize,
}

impl Answerer for Echo {
    fn answer(&self, request: Value) -> Vec<Value> {
        self.asked.fetch_add(1, Ordering::Relaxed);
        (0..self.chunks)
            .map(|n| json!({"ok": true, "kind": "echo", "seq": n, "asked": request}))
            .collect()
    }
}

/// A bound listener and a seat pointed at it, over freshly minted material.
fn wired(chunks: usize) -> (TempDir, Listener, Seat, Arc<AtomicUsize>) {
    let tmp = TempDir::new().expect("tmp");
    mint(tmp.path());
    let asked = Arc::new(AtomicUsize::new(0));
    let listener = Listener::bind(
        &material(
            tmp.path(),
            Role::Server,
            crate::test_support::wire::EPHEMERAL,
        ),
        Arc::new(Echo {
            asked: Arc::clone(&asked),
            chunks,
        }),
    )
    .expect("bind");
    let seat = Seat::open(&material(tmp.path(), Role::Client, &listener.address())).expect("seat");
    (tmp, listener, seat, asked)
}

/// The whole wire: a certificate-bearing seat gestures, the engine answers,
/// and the answer is the envelope the seat asked about.
#[test]
fn a_certificate_bearing_seat_is_answered() {
    let (_tmp, listener, seat, asked) = wired(1);
    assert!(listener.address().starts_with("127.0.0.1:"));
    let stream = seat.ask(&json!({"op": "workspaces"})).expect("answered");
    assert_eq!(stream.len(), 1);
    assert_eq!(stream[0]["asked"], json!({"op": "workspaces"}));
    assert_eq!(asked.load(Ordering::Relaxed), 1);
    // One connection per gesture, and a second one is answered like the first.
    seat.ask(&json!({"op": "board"})).expect("answered");
    assert_eq!(asked.load(Ordering::Relaxed), 2);
}

/// A follow-class answer is not a second form: N frames then the terminator,
/// which is the same reader the one-frame answer above went through.
#[test]
fn a_many_frame_answer_is_the_same_shape() {
    let (_tmp, _listener, seat, _asked) = wired(3);
    let stream = seat.ask(&json!({"op": "ops"})).expect("answered");
    assert_eq!(stream.len(), 3);
    assert_eq!(stream[2]["seq"], json!(2));
}

/// **An unauthenticated connection gets a TLS refusal, not a yog reply**
/// (REMOTE §4): a peer holding no client certificate never reaches the
/// answerer, so nothing was asked and nothing was said.
#[test]
fn an_uncertificated_peer_is_refused_by_tls() {
    let tmp = TempDir::new().expect("tmp");
    mint(tmp.path());
    let asked = Arc::new(AtomicUsize::new(0));
    let listener = Listener::bind(
        &material(
            tmp.path(),
            Role::Server,
            crate::test_support::wire::EPHEMERAL,
        ),
        Arc::new(Echo {
            asked: Arc::clone(&asked),
            chunks: 1,
        }),
    )
    .expect("bind");
    // A plain TCP peer speaking JSON at a TLS socket: the bytes are never a
    // handshake, so the connection dies inside rustls.
    let mut plain = std::net::TcpStream::connect(listener.address()).expect("connect");
    let _ = crate::wire::frame::write_value(&mut plain, &json!({"op": "workspaces"}));
    plain
        .set_read_timeout(Some(Duration::from_secs(5)))
        .expect("timeout");
    let mut buf = [0u8; 1];
    // Either a TLS alert or a closed socket — never a reply frame.
    let _ = std::io::Read::read(&mut plain, &mut buf);
    assert_eq!(asked.load(Ordering::Relaxed), 0);
}

/// A certificate the operator's CA did not issue is the same refusal: identity
/// is the certificate, and a foreign CA's leaf is no identity here.
#[test]
fn a_foreign_certificate_is_refused() {
    let tmp = TempDir::new().expect("tmp");
    mint(tmp.path());
    let stranger = TempDir::new().expect("tmp");
    mint(stranger.path());
    let asked = Arc::new(AtomicUsize::new(0));
    let listener = Listener::bind(
        &material(
            tmp.path(),
            Role::Server,
            crate::test_support::wire::EPHEMERAL,
        ),
        Arc::new(Echo {
            asked: Arc::clone(&asked),
            chunks: 1,
        }),
    )
    .expect("bind");
    // The stranger's own CA and leaf: a complete, valid, wrong identity. Its
    // anchors are swapped for ours so it will accept our server and we can see
    // the *client* half refuse.
    std::fs::copy(tmp.path().join("ca.pem"), stranger.path().join("ca.pem")).expect("copy");
    let seat = Seat::open(&material(
        stranger.path(),
        Role::Client,
        &listener.address(),
    ))
    .expect("seat");
    seat.ask(&json!({"op": "workspaces"})).expect_err("refused");
    assert_eq!(asked.load(Ordering::Relaxed), 0);
}

/// A connection that says nothing costs nothing: the serve loop ends on the
/// peer's EOF rather than waiting on a frame that is not coming.
#[test]
fn a_silent_connection_ends_at_eof() {
    let (_tmp, listener, _seat, asked) = wired(1);
    drop(std::net::TcpStream::connect(listener.address()).expect("connect"));
    assert_eq!(asked.load(Ordering::Relaxed), 0);
}

/// A configuration rustls cannot make a connection from is a dropped
/// connection, never a panic — the one shape prod is not allowed to have.
#[test]
fn an_unusable_config_drops_the_connection() {
    let tmp = TempDir::new().expect("tmp");
    mint(tmp.path());
    let mut config = Arc::into_inner(
        crate::wire::tls::server_config(&material(tmp.path(), Role::Server, "127.0.0.1:0"))
            .expect("config"),
    )
    .expect("sole owner");
    // Out of range by rustls' own rule, which is the one way `new` refuses.
    config.max_fragment_size = Some(1);
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let address = listener.local_addr().expect("addr").to_string();
    let peer = std::thread::spawn(move || {
        let (stream, _) = listener.accept().expect("accept");
        serve(
            stream,
            &Arc::new(config),
            &Echo {
                asked: Arc::new(AtomicUsize::new(0)),
                chunks: 1,
            },
        );
    });
    drop(TcpStream::connect(address).expect("connect"));
    peer.join().expect("served");
}

/// Dropping the listener stops it: the port stops answering.
#[test]
fn dropping_the_listener_stops_it() {
    let (_tmp, listener, seat, _asked) = wired(1);
    seat.ask(&json!({"op": "workspaces"})).expect("answered");
    drop(listener);
    // The socket is closed with the accept loop, so the next dial refuses.
    seat.ask(&json!({"op": "workspaces"})).expect_err("stopped");
}

/// An address nothing can bind refuses, naming it.
#[test]
fn an_unbindable_address_refuses() {
    let tmp = TempDir::new().expect("tmp");
    mint(tmp.path());
    let refusal = Listener::bind(
        &material(tmp.path(), Role::Server, "256.256.256.256:1"),
        Arc::new(Echo {
            asked: Arc::new(AtomicUsize::new(0)),
            chunks: 1,
        }),
    )
    .err()
    .expect("refused");
    assert!(refusal.contains("256.256.256.256:1"), "{refusal}");
}

/// Material that will not build a configuration refuses before anything binds.
#[test]
fn unusable_material_refuses_before_binding() {
    let tmp = TempDir::new().expect("tmp");
    mint(tmp.path());
    std::fs::write(tmp.path().join("ca.pem"), "").expect("write");
    assert!(
        Listener::bind(
            &material(
                tmp.path(),
                Role::Server,
                crate::test_support::wire::EPHEMERAL
            ),
            Arc::new(Echo {
                asked: Arc::new(AtomicUsize::new(0)),
                chunks: 1,
            }),
        )
        .is_err()
    );
}
