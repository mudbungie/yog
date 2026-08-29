//! **Version skew, at the door** (REMOTE §3, bl-a670) — the listener's own
//! contract argued whole, beside [`identity`](super::identity) and for the same
//! reason. Identity asks *who* reached the answerer; this asks *whether
//! anything did*, because a peer of another protocol is refused before a
//! gesture is decoded and therefore before a certificate has bought it
//! anything.
//!
//! The client here is hand-built rather than a [`Seat`], because a `Seat` can
//! only ever state its own build's version — which is exactly the property
//! that made the wire versionless until four components could be installed
//! separately.

use super::*;
use rustls::ClientConnection;
use rustls::pki_types::ServerName;
use std::net::{Ipv4Addr, TcpStream};
use std::path::Path;

/// Dial `address` as a client of `dir`'s CA, state `version`, ask for the
/// workspaces, and read every frame the engine writes back.
fn stating(dir: &Path, address: &str, version: u32) -> Vec<Value> {
    let config = crate::wire::tls::client_config(&material(dir, Role::Client, address))
        .expect("client config");
    let conn = ClientConnection::new(config, ServerName::IpAddress(Ipv4Addr::LOCALHOST.into()))
        .expect("tls");
    let tcp = TcpStream::connect(address).expect("connect");
    tcp.set_read_timeout(Some(Duration::from_secs(10)))
        .expect("timeout");
    let mut tls = StreamOwned::new(conn, tcp);
    crate::wire::frame::write_value(&mut tls, &json!({ "protocol": version })).expect("preface");
    // A refused peer may already have been hung up on, and that is not this
    // caller's error to make: the refusal is what the read below collects.
    let _ = crate::wire::frame::write_value(&mut tls, &json!({"op": "workspaces"}));
    let mut frames = Vec::new();
    while let Ok(Some(chunk)) = crate::wire::frame::read_value(&mut tls) {
        frames.push(chunk);
    }
    frames
}

/// **The engine states its own version to every peer**, and a peer that speaks
/// it is answered exactly as before — the preface is a frame beside the
/// gesture, never a change to it.
#[test]
fn an_engine_states_its_version_and_answers_a_peer_that_shares_it() {
    let (tmp, listener, _seat, asked) = wired(1);
    let frames = stating(
        tmp.path(),
        &listener.address(),
        crate::wire::hello::PROTOCOL,
    );
    assert_eq!(frames.len(), 2, "the preface, then the answer");
    assert_eq!(
        frames[0],
        json!({ "protocol": crate::wire::hello::PROTOCOL })
    );
    assert_eq!(frames[1]["asked"], json!({"op": "workspaces"}));
    assert_eq!(asked.load(Ordering::Relaxed), 1);
}

/// **A skewed peer is refused, and nothing it asked for was adjudicated.** The
/// gesture it sent in the same breath as its preface reaches no answerer: the
/// refusal happens above the codec, so a version this build does not speak
/// cannot mean anything here by accident.
#[test]
fn a_skewed_peer_is_refused_and_never_reaches_the_answerer() {
    let (tmp, listener, _seat, asked) = wired(1);
    let frames = stating(
        tmp.path(),
        &listener.address(),
        crate::wire::hello::PROTOCOL + 1,
    );
    assert_eq!(frames.len(), 2, "the preface, then the refusal");
    assert_eq!(frames[1]["ok"], json!(false));
    let said = frames[1]["error"].as_str().expect("a sentence");
    assert!(said.contains("wire protocol mismatch"), "{said}");
    assert!(said.contains("upgrade the older component"), "{said}");
    assert_eq!(asked.load(Ordering::Relaxed), 0);
}

/// **The seat's half, end to end.** A `Seat` states this build's version, so
/// the engine admits it and the answer arrives — which is the whole of what
/// every other wire test now exercises on its way past the preface.
#[test]
fn a_seat_of_this_build_is_admitted() {
    let (_tmp, _listener, seat, asked) = wired(1);
    let stream = seat.ask(&json!({"op": "workspaces"})).expect("answered");
    assert_eq!(stream.len(), 1, "the preface is consumed by the dial");
    assert_eq!(stream[0]["asked"], json!({"op": "workspaces"}));
    assert_eq!(asked.load(Ordering::Relaxed), 1);
}
