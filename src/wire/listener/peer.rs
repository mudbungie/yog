//! The peer of one connection, seen from the server's side (bl-4263): who it
//! is, how long its silence is waited on, and how it is hung up on. Split from
//! [`listener`](super) at §12's cap.

use crate::registry::{Client, Peer};
use rustls::pki_types::CertificateDer;
use rustls::{ServerConnection, StreamOwned};
use std::io::{Read, Write};
use std::net::{Shutdown, TcpStream};
use std::time::Duration;

/// How long an accepted connection may say nothing before the server treats it
/// as gone (REMOTE §5.1, bl-1421). **A bound on the transport, not on the
/// wait** — thrall's channel states the same two minutes from the other end,
/// and for the same reason: the server's longest legitimate quiet is a
/// follow-class hold, which is thirty seconds
/// ([`slots`](crate::registry::mailbox), [`follow`](crate::boundary::follow),
/// and [`attend`](crate::boundary::attend), which holds on the follow lane's
/// own two constants rather than a third pair)
/// and then an answer, so a client parked for hours is a sequence of answered
/// reads and never one read held for hours. No client idles past it: a foot
/// re-asks immediately, a seat dials per gesture.
///
/// **A timeout is "the connection is gone", never a retry.** It fires mid-record
/// as readily as between them, and rustls has no clean resume from a half-read
/// frame — so the read loop ends on it exactly as it ends on an EOF, which is
/// what releases the presence guard and returns the thread.
pub(crate) const IDLE_TIMEOUT: Duration = Duration::from_mins(2);

/// **A refusal is only a refusal if the peer can read it** (bl-e4c8).
///
/// A refused peer is the one connection the server hangs up on *first*:
/// everywhere else the peer closes and the server reads the EOF. Closing a
/// socket that still holds unread bytes — or that the peer writes to just after
/// — makes the kernel answer RST, and an RST **discards what the peer had
/// already received**. So the seat whose refusal was sitting in its receive
/// buffer reads a transport error instead, having been told nothing: exactly
/// the outcome [`hello`](super::super::hello) refused ALPN to avoid, arriving
/// by a different door. It is a race and it reads as silence, which is the
/// worst pair of properties a refusal can have.
///
/// The server therefore half-closes and reads to EOF. `close_notify` and a FIN
/// say there is nothing more coming; the read side stays open until the peer
/// has said its own last word, so nothing it wrote is ever unread at the close.
/// The wait is the connection's own quiet bound ([`IDLE_TIMEOUT`]) and not a new one — a
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
