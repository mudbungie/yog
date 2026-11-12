//! **One accepted connection** (REMOTE §4, §15.2; bl-9ea6): the handshake
//! driven to its end, the one question asked of it — did the peer present a
//! certificate — and then the wire or the registration door. Split from
//! [`listener`](super) at §12's cap on the seam its own doc draws: everything
//! there is the accept loop and its handle, and this is what one thread does
//! with what was accepted.

use super::peer::{self, peer_client};
use super::{Answerer, Quiet, door};
use crate::registry::presence::Presence;
use crate::wire::frame;
use rustls::{ServerConfig, ServerConnection, StreamOwned};
use std::net::TcpStream;
use std::sync::Arc;
use std::time::Duration;

/// **One dialled connection, which is one of two things** (REMOTE §15.2):
/// a peer that presented a certificate is the wire, byte for byte as before;
/// a peer that presented none is offered the registration [`door`] and
/// nothing else.
pub(super) fn dialled(
    tcp: TcpStream,
    config: &Arc<ServerConfig>,
    answerer: &dyn Answerer,
    presence: &Presence,
    slots: &door::Slots,
) {
    let Some(mut tls) = shake(tcp, config, Quiet::dialled()) else {
        return;
    };
    if tls.conn.peer_certificates().is_some() {
        wire(&mut tls, answerer, presence, Quiet::dialled());
    } else {
        door::serve(&mut tls, answerer, slots, door::DEADLINE);
    }
}

/// One **wire** connection: handshake, then request → reply stream →
/// terminator, until the peer goes away or a frame refuses. The punch port's
/// acceptor enters here, and so a peer with no certificate is hung up on with
/// nothing said — the door is the dialled listener's alone.
///
/// `quiet` is how long a read may find nothing before the peer counts as gone
/// — [`Quiet::dialled`] is the production bound, and a test names a short one
/// rather than sleeping for real — and, on a held connection, how often the
/// silence is pinged ([`peer`]). A socket that refuses the timeout is served
/// without one.
///
/// Answers whether the peer was **admitted** — `false` for a handshake or a
/// preface that refused it, which the punch port's acceptor says (bl-5276).
pub(crate) fn serve(
    tcp: TcpStream,
    config: &Arc<ServerConfig>,
    answerer: &dyn Answerer,
    presence: &Presence,
    quiet: Quiet,
) -> bool {
    let Some(mut tls) = shake(tcp, config, quiet) else {
        return false;
    };
    if tls.conn.peer_certificates().is_none() {
        peer::hang_up(&mut tls);
        return false;
    }
    wire(&mut tls, answerer, presence, quiet)
}

/// The TLS handshake, driven to its end so the caller can ask what the peer
/// presented before a byte of anything else is read. A handshake that fails
/// is hung up on — which is what delivers the alert saying so — and is `None`.
fn shake(tcp: TcpStream, config: &Arc<ServerConfig>, quiet: Quiet) -> Option<Tls> {
    let _ = tcp.set_read_timeout(Some(quiet.read_timeout()));
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
fn wire(tls: &mut Tls, answerer: &dyn Answerer, presence: &Presence, quiet: Quiet) -> bool {
    let Some(edition) = crate::wire::hello::admit(tls) else {
        peer::hang_up(tls);
        return false;
    };
    let _ = converse(tls, answerer, presence, quiet, edition);
    true
}

type Tls = StreamOwned<ServerConnection, TcpStream>;

/// An admitted connection's requests, answered until the peer goes away.
/// It returns only then, so its one answer is `None` — and every way the peer
/// can be gone is a `?` onto it, rather than an exit apiece.
fn converse(
    tls: &mut Tls,
    answerer: &dyn Answerer,
    presence: &Presence,
    quiet: Quiet,
    edition: u32,
) -> Option<std::convert::Infallible> {
    // **Presence is this scope** (REMOTE §5, bl-4e08): the guard is taken when
    // the connection first names its client and released when this function
    // leaves, however it leaves — a clean close, a refused frame, a peer that
    // vanished without a FIN (which is [`IDLE_TIMEOUT`] expiring, and was the
    // one case this list claimed and did not hold — bl-1421). There is no
    // leave verb to forget, which is what makes
    // "connected right now" true rather than aspirational. It cannot be taken
    // any earlier: the handshake completes inside the first read, so before it
    // there is no certificate to read an identity off.
    let mut live = None;
    let mut quiet_for = Duration::ZERO;
    loop {
        let request = match frame::read_value(tls) {
            Ok(Some(request)) => request,
            // A held connection's silence is pinged, one timeout at a time,
            // until the bound says the peer is gone; every other end is the
            // peer's own — an EOF, a refused frame, a vanished socket.
            Err(e) if quiet.pings_at(quiet_for, &e) && peer::ping(tls) => {
                quiet_for += quiet.read_timeout();
                continue;
            }
            _ => return None,
        };
        quiet_for = Duration::ZERO;
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
