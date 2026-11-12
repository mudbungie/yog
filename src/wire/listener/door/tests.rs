//! The registration door: the one request it reads, the two answers it gives,
//! and the three bounds on what a stranger can hold.

use super::*;
use crate::registry::Peer;
use crate::registry::presence::Presence;
use crate::test_support::wire::{certless, material, mint, registration};
use crate::wire::listener::Listener;
use crate::wire::material::{ANCHORS, Role};
use serde_json::json;
use std::net::TcpListener;
use tempfile::TempDir;

/// An answerer whose door opens for one code and echoes what it was handed.
struct Keeper;

impl Answerer for Keeper {
    fn answer(&self, _peer: &Peer, _request: Value) -> Box<dyn Iterator<Item = Value>> {
        Box::new(std::iter::once(json!({"ok": true, "kind": "wire"})))
    }

    fn register(&self, code: &str, body: &[u8]) -> Option<Value> {
        (code == "GOODCODE").then(|| json!({ "said": String::from_utf8_lossy(body) }))
    }
}

/// An answerer that holds no offers: the trait's own default.
struct Closed;

impl Answerer for Closed {
    fn answer(&self, _peer: &Peer, _request: Value) -> Box<dyn Iterator<Item = Value>> {
        Box::new(std::iter::empty())
    }
}

fn listening(answerer: Arc<dyn Answerer>) -> (TempDir, Listener) {
    let tmp = TempDir::new().expect("tmp");
    mint(tmp.path());
    let server = material(
        tmp.path(),
        Role::Server,
        crate::test_support::wire::EPHEMERAL,
    );
    let listener = Listener::bind(&server, answerer, Presence::default()).expect("bind");
    (tmp, listener)
}

/// **A peer with no certificate is answered its one request and closed**
/// (REMOTE §15.2): the claim's JSON object under a `200`, with the length
/// stated and the connection said to be closing.
#[test]
fn a_registration_is_answered_and_closed() {
    let (tmp, listener) = listening(Arc::new(Keeper));
    let said = certless(
        &tmp.path().join(ANCHORS),
        &listener.address(),
        &registration("GOODCODE", b"{\"public_key\":\"pem\"}"),
    );
    let (head, body) = said.split_once("\r\n\r\n").expect("a head and a body");
    assert!(head.starts_with("HTTP/1.1 200 OK\r\n"), "{head}");
    assert!(head.contains("Content-Type: application/json"), "{head}");
    assert!(
        head.contains(&format!("Content-Length: {}", body.len())),
        "{head}"
    );
    assert!(head.contains("Connection: close"), "{head}");
    let answer: Value = serde_json::from_str(body).expect("json");
    assert_eq!(answer["said"], "{\"public_key\":\"pem\"}");
}

/// **Everything else is the same bare `404`**: a code the answerer refuses, a
/// path that is not the door's, a method that is not POST, a request with no
/// stated length, bytes that are no request at all — and a wire frame, which a
/// certless peer may not say either. Indistinguishable, by construction.
#[test]
fn everything_else_is_one_bare_404() {
    let (tmp, listener) = listening(Arc::new(Keeper));
    let ask = |request: &[u8]| certless(&tmp.path().join(ANCHORS), &listener.address(), request);
    let unknown = ask(&registration("OTHERCODE", b"{}"));
    assert_eq!(
        unknown,
        "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
    );
    let mut wire_frame = Vec::new();
    crate::wire::frame::write_value(&mut wire_frame, &json!({"op": "workspaces"})).expect("frame");
    wire_frame.extend(b"\r\n\r\n");
    for request in [
        &b"POST /elsewhere/GOODCODE HTTP/1.1\r\nContent-Length: 2\r\n\r\n{}"[..],
        b"GET /yog/register/GOODCODE HTTP/1.1\r\nContent-Length: 2\r\n\r\n{}",
        b"POST /yog/register/GOODCODE HTTP/1.0\r\nContent-Length: 2\r\n\r\n{}",
        b"POST /yog/register/GOODCODE HTTP/1.1\r\nHost: engine.invalid\r\n\r\n{}",
        b"POST /yog/register/GOODCODE HTTP/1.1\r\nContent-Length: two\r\n\r\n{}",
        b"\xff\xfe\r\n\r\n",
        &wire_frame,
    ] {
        assert_eq!(
            ask(request),
            unknown,
            "{}",
            String::from_utf8_lossy(request)
        );
    }
    // And a door nobody keeps answers the same to the right-looking request.
    let (closed_tmp, closed) = listening(Arc::new(Closed));
    let said = certless(
        &closed_tmp.path().join(ANCHORS),
        &closed.address(),
        &registration("GOODCODE", b"{}"),
    );
    assert_eq!(said, unknown);
}

/// **A request may be a few kilobytes and no more**: a head that never ends
/// and a body stated past the cap are each refused without being read whole.
#[test]
fn a_request_past_the_size_cap_is_refused() {
    let (tmp, listener) = listening(Arc::new(Keeper));
    let ask = |request: &[u8]| certless(&tmp.path().join(ANCHORS), &listener.address(), request);
    let endless = [
        b"POST /yog/register/GOODCODE HTTP/1.1\r\nX: ".to_vec(),
        vec![b'a'; MAX_REQUEST],
    ]
    .concat();
    assert!(ask(&endless).starts_with("HTTP/1.1 404 "));
    let stated = format!(
        "POST /yog/register/GOODCODE HTTP/1.1\r\nContent-Length: {MAX_REQUEST}\r\n\r\n{{}}"
    );
    assert!(ask(stated.as_bytes()).starts_with("HTTP/1.1 404 "));
    // The largest request that fits is still served.
    let head = "POST /yog/register/GOODCODE HTTP/1.1\r\nContent-Length: ";
    let room = MAX_REQUEST - head.len() - "0000\r\n\r\n".len();
    let fits = [
        format!("{head}{room:04}\r\n\r\n").into_bytes(),
        vec![b' '; room],
    ]
    .concat();
    assert_eq!(fits.len(), MAX_REQUEST);
    assert!(ask(&fits).starts_with("HTTP/1.1 200 "));
}

/// One certless connection served directly, with a stated patience — the
/// production deadline is seconds, and a test names a short one rather than
/// sleeping for it.
fn served(slots: &Slots, patience: Duration, request: &'static [u8]) -> (String, Duration) {
    let tmp = TempDir::new().expect("tmp");
    mint(tmp.path());
    let server = material(
        tmp.path(),
        Role::Server,
        crate::test_support::wire::EPHEMERAL,
    );
    let config = crate::wire::tls::server_config(&server).expect("config");
    let tcp = TcpListener::bind("127.0.0.1:0").expect("bind");
    let address = tcp.local_addr().expect("bound").to_string();
    let slots = slots.clone();
    let server = std::thread::spawn(move || {
        let (stream, _) = tcp.accept().expect("accept");
        let conn = ServerConnection::new(config).expect("conn");
        let mut tls = StreamOwned::new(conn, stream);
        let began = Instant::now();
        serve(&mut tls, &Keeper, &slots, patience);
        began.elapsed()
    });
    let said = certless(&tmp.path().join(ANCHORS), &address, request);
    (said, server.join().expect("served"))
}

/// **A peer has a few seconds, in all**: a request that stops short of the
/// body it stated is a `404` once the deadline passes, and the connection is
/// not held a moment longer for the peer's sake.
#[test]
fn a_peer_that_does_not_finish_is_refused_at_the_deadline() {
    let patience = Duration::from_millis(300);
    let (said, took) = served(
        &Slots::default(),
        patience,
        b"POST /yog/register/GOODCODE HTTP/1.1\r\nContent-Length: 64\r\n\r\n{",
    );
    assert!(said.starts_with("HTTP/1.1 404 "), "{said}");
    assert!(took >= patience, "it waited the deadline out: {took:?}");
    assert!(took < Duration::from_secs(10), "and no longer: {took:?}");
}

/// **Only so many certless connections are served at once**: with every slot
/// held, the next is closed with nothing said — and a slot given back is a
/// slot the next peer takes.
#[test]
fn certless_connections_past_the_cap_are_closed_unanswered() {
    let slots = Slots::default();
    let held: Vec<Slot> = (0..SLOTS).map(|_| slots.enter().expect("a slot")).collect();
    assert!(slots.enter().is_none(), "the cap");
    let request = b"POST /yog/register/GOODCODE HTTP/1.1\r\nContent-Length: 2\r\n\r\n{}";
    let (said, _) = served(&slots, Duration::from_secs(5), request);
    assert_eq!(said, "", "nothing is said to the peer past the cap");
    drop(held);
    let (said, _) = served(&slots, Duration::from_secs(5), request);
    assert!(said.starts_with("HTTP/1.1 200 "), "{said}");
}

/// The head's grammar, off the socket: the header name is read in any case,
/// and the code is whatever follows the path.
#[test]
fn the_head_yields_the_code_and_the_stated_length() {
    let read = |text: &str| head(text);
    assert_eq!(
        read("POST /yog/register/ABCDEFGH HTTP/1.1\r\nCONTENT-LENGTH:  12 \r\n\r\n"),
        Some(("ABCDEFGH".to_owned(), 12))
    );
    assert_eq!(read(""), None);
    assert_eq!(read("POST /yog/register/ABCDEFGH HTTP/1.1\r\n\r\n"), None);
}
