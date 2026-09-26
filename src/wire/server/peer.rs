//! The peer of one connection, seen from the engine's side (bl-4263): who it
//! is, how its silence is read, and how it is hung up on. Split from
//! [`server`](super) at §12's cap when the punched wire gave a connection a
//! second way to be quiet.
//!
//! **A held connection is quiet on purpose** (REMOTE §13.4). A dialled seat
//! asks and hangs up, so two minutes of nothing is a peer that vanished; a
//! punched stream is *held* between asks, and the two NATs it crosses forget
//! a mapping that carries nothing. So the engine writes a ping frame into the
//! silence at a 25-second cadence — a frame, because std exposes no keepalive
//! and a frame is what a test can read — and the two-minute bound still says
//! when the peer is gone. The frame is `{"ping":true}` and never the start of
//! a reply stream: a client reading a held connection discards it.

use super::frame;
use crate::registry::{Client, Peer};
use rustls::pki_types::CertificateDer;
use rustls::{ServerConnection, StreamOwned};
use serde_json::{Value, json};
use std::io::{self, Read, Write};
use std::net::{Shutdown, TcpStream};
use std::time::Duration;

/// How a connection's silence is read.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Quiet {
    /// How long a read may find nothing, in all, before the peer counts as
    /// gone.
    pub(crate) gone: Duration,
    /// On a held connection, how often the engine pings into the silence;
    /// `None` is a dialled connection, which is never pinged.
    pub(crate) ping: Option<Duration>,
}

impl Quiet {
    /// The socket's read timeout: one ping's worth of silence, or the whole
    /// bound where nothing is pinged.
    pub(crate) fn read_timeout(self) -> Duration {
        self.ping.unwrap_or(self.gone)
    }

    /// Whether a read that found nothing for one more timeout is a moment to
    /// ping rather than the end: only on a held connection, and only while
    /// the silence so far — `quiet_for`, which the caller advances on a
    /// `true` — is still inside the bound.
    pub(crate) fn pings_at(self, quiet_for: Duration, e: &io::Error) -> bool {
        let timed_out = matches!(
            e.kind(),
            io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
        );
        timed_out && self.ping.is_some_and(|ping| quiet_for + ping < self.gone)
    }
}

/// The frame a held connection carries through its silence.
pub(crate) fn ping_frame() -> Value {
    json!({"ping": true})
}

/// Write one ping; a peer the write cannot reach is gone.
pub(crate) fn ping(tls: &mut StreamOwned<ServerConnection, TcpStream>) -> bool {
    frame::write_value(tls, &ping_frame()).is_ok()
}

/// **A refusal is only a refusal if the peer can read it** (bl-e4c8).
///
/// A refused peer is the one connection the engine hangs up on *first*:
/// everywhere else the peer closes and the engine reads the EOF. Closing a
/// socket that still holds unread bytes — or that the peer writes to just after
/// — makes the kernel answer RST, and an RST **discards what the peer had
/// already received**. So the seat whose refusal was sitting in its receive
/// buffer reads a transport error instead, having been told nothing: exactly
/// the outcome [`hello`](super::super::hello) refused ALPN to avoid, arriving
/// by a different door. It is a race and it reads as silence, which is the
/// worst pair of properties a refusal can have.
///
/// The engine therefore half-closes and reads to EOF. `close_notify` and a FIN
/// say there is nothing more coming; the read side stays open until the peer
/// has said its own last word, so nothing it wrote is ever unread at the close.
/// The wait is [`serve`](super::serve)'s quiet bound and not a new one — a
/// peer that will not hang up is a peer saying nothing, which is the case that
/// clock is for.
pub(crate) fn hang_up(tls: &mut StreamOwned<ServerConnection, TcpStream>) {
    tls.conn.send_close_notify();
    let _ = tls.flush();
    let _ = tls.sock.shutdown(Shutdown::Write);
    let mut spent = [0u8; 1024];
    while matches!(tls.sock.read(&mut spent), Ok(1..)) {}
}

/// The peer a presented chain names (REMOTE §2, §4.2): the **leaf's** subject
/// common name, and the grade the same subject carries. The leaf is the first
/// certificate — TLS sends the end entity first and the chain toward the anchor
/// after it, so the issuer's own common name is never mistaken for the peer's,
/// and an `OU` on the CA is never mistaken for a grade on the client.
pub(crate) fn peer_client(chain: Option<&[CertificateDer<'_>]>) -> Option<Peer> {
    let leaf = chain?.first()?;
    let client = Client::parse(&crate::registry::leaf::common_name(leaf)?).ok()?;
    Some(Peer {
        client,
        grade: crate::registry::leaf::grade(leaf),
    })
}

#[cfg(test)]
mod tests;
