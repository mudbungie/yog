//! **One accepted connection** (REMOTE §4, §15.2; bl-9ea6): the handshake
//! driven to its end, the one question asked of it — did the peer present a
//! certificate — and then the wire or the registration door. Split from
//! [`listener`](super) at §12's cap on the seam its own doc draws: everything
//! there is the accept loop and its handle, and this is what one thread does
//! with what was accepted.

use super::peer::{self, peer_client};
use super::{Answerer, door};
use crate::registry::presence::Presence;
use crate::wire::frame;
use rustls::{ServerConfig, ServerConnection, StreamOwned};
use std::net::TcpStream;
use std::sync::Arc;
use std::time::Duration;

/// **One accepted connection, which is one of two things** (REMOTE §15.2):
/// a peer that presented a certificate is the wire — request → reply stream →
/// terminator, until the peer goes away or a frame refuses; a peer that
/// presented none is offered the registration [`door`] and nothing else.
///
/// `quiet` is how long a read may find nothing before the peer counts as gone
/// — [`peer::IDLE_TIMEOUT`] at the accept loop's one call, and a test names a
/// short one rather than sleeping for real. A socket that refuses the timeout
/// is served without one.
pub(super) fn dialled(
    tcp: TcpStream,
    config: &Arc<ServerConfig>,
    answerer: &dyn Answerer,
    presence: &Presence,
    slots: &door::Slots,
    quiet: Duration,
) {
    let Some(mut tls) = shake(tcp, config, quiet) else {
        return;
    };
    if tls.conn.peer_certificates().is_some() {
        wire(&mut tls, answerer, presence);
    } else {
        door::serve(&mut tls, answerer, slots, door::DEADLINE);
    }
}

/// The TLS handshake, driven to its end so the caller can ask what the peer
/// presented before a byte of anything else is read. A handshake that fails
/// is hung up on — which is what delivers the alert saying so — and is `None`.
fn shake(tcp: TcpStream, config: &Arc<ServerConfig>, quiet: Duration) -> Option<Tls> {
    let _ = tcp.set_read_timeout(Some(quiet));
    let conn = ServerConnection::new(Arc::clone(config)).ok()?;
    let mut tls = StreamOwned::new(conn, tcp);
    while tls.conn.is_handshaking() {
        if tls.conn.complete_io(&mut tls.sock).is_err() {
            peer::hang_up(&mut tls);
            return None;
        }
    }
    Some(tls)
}

/// An authenticated connection: the §3 version preface, stated and checked
/// before any gesture (bl-a670), then its requests. What an admitted peer
/// hands back is its corpus EDITION (REMOTE §3.2, bl-1be7) — stated, never
/// adjudicated — which rides on the presence entry.
fn wire(tls: &mut Tls, answerer: &dyn Answerer, presence: &Presence) {
    let Some(edition) = crate::wire::hello::admit(tls) else {
        peer::hang_up(tls);
        return;
    };
    let _ = converse(tls, answerer, presence, edition);
}

type Tls = StreamOwned<ServerConnection, TcpStream>;

/// An admitted connection's requests, answered until the peer goes away.
/// It returns only then, so its one answer is `None` — and every way the peer
/// can be gone is a `?` onto it, rather than an exit apiece.
fn converse(
    tls: &mut Tls,
    answerer: &dyn Answerer,
    presence: &Presence,
    edition: u32,
) -> Option<std::convert::Infallible> {
    // **Presence is this scope** (REMOTE §5, bl-4e08): the guard is taken when
    // the connection first names its client and released when this function
    // leaves, however it leaves — a clean close, a refused frame, a peer that
    // vanished without a FIN (which is [`peer::IDLE_TIMEOUT`] expiring, and was the
    // one case this list claimed and did not hold — bl-1421). There is no
    // leave verb to forget, which is what makes
    // "connected right now" true rather than aspirational. It cannot be taken
    // any earlier: the handshake completes inside the first read, so before it
    // there is no certificate to read an identity off.
    let mut live = None;
    loop {
        // Every end is the peer's own — an EOF, a refused frame, a vanished
        // socket, or the quiet bound expiring on a read.
        let request = frame::read_value(tls).ok()??;
        // **The identity is derived per request, not held** (REMOTE §4,
        // bl-8bbc): the handshake completes inside the first read, so there is
        // no earlier moment to read a certificate at, and re-reading it is a
        // DER walk over bytes already in memory. A peer whose certificate
        // carries no name yog can use is dropped without a reply, on exactly
        // the terms an unauthenticated peer is — a connection that cannot be
        // authorized gets nothing said to it.
        let peer = peer_client(tls.conn.peer_certificates())?;
        let _ = live.get_or_insert_with(|| presence.enter(&peer.client, edition));
        for chunk in answerer.answer(&peer, request) {
            frame::write_value(tls, &chunk).ok()?;
        }
        frame::write_end(tls).ok()?;
    }
}
