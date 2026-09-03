//! The engine's listener (REMOTE §8, §9.5; bl-b6fa): a synchronous mTLS accept
//! loop over `std::net`, one thread per connection.
//!
//! **It is a second intake to the same chokepoints, never a second surface.**
//! The gestures inbox (§8.5) and this listener are two doors into one
//! [`Answerer`] — the deposit consumer's own [`ConsumerCtx`](crate::boundary::consumer::ConsumerCtx),
//! which decodes with the one codec and runs the one `dispatch`/`answer`. The
//! wire therefore adds no verb (REMOTE §3): a capability a seat lacks is added
//! to the boundary, where every face gains it.
//!
//! **No async, and no tokio.** yog is a synchronous process (AGENTS.md rule 8
//! is installed and vacuous, and stays that way): the accept loop polls a
//! non-blocking [`TcpListener`] so a [`Drop`] can stop it, and each connection
//! is a blocking thread. That is the [`Worker`](crate::app::Worker) shape the
//! rest of the engine already uses — a stop flag, a loop, a `Drop` that joins
//! — and it costs one thread per live seat, which is the number of seats an
//! operator has.
//!
//! **An unauthenticated connection gets a TLS refusal, not a yog reply**
//! (REMOTE §4). The handshake happens inside the first `read`, so a peer with
//! no certificate never reaches [`frame`](super::frame) and the connection is
//! dropped with nothing said.

use super::frame;
use super::material::Material;
use crate::registry::presence::Presence;
use crate::registry::{Client, Peer};
use rustls::pki_types::CertificateDer;
use rustls::{ServerConfig, ServerConnection, StreamOwned};
use serde_json::Value;
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;
use std::time::Duration;

/// How often the accept loop looks for a connection. A latency knob on
/// *shutdown*, not on connections: the socket backlog holds an arriving seat
/// until the next look.
const ACCEPT_POLL: Duration = Duration::from_millis(20);

/// How long an accepted connection may say nothing before the engine treats it
/// as gone (REMOTE §5.1, bl-1421). **A bound on the transport, not on the
/// wait** — thrall's channel states the same two minutes from the other end,
/// and for the same reason: the engine's longest legitimate quiet is a
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
const IDLE_TIMEOUT: Duration = Duration::from_mins(2);

/// What answers a request frame: the reply stream it becomes, one [`Value`] per
/// frame. Most answers are one element long; a follow-class read is the same
/// signature with more of them (see [`frame`](super::frame)).
///
/// **It is an iterator and not a `Vec` since bl-73e7**, which is the whole of
/// what minting the follow lane cost the wire. A materialized list has to be
/// finished before the first frame can be written, so a read that answers *as
/// the world changes* could not be one; pulled lazily, the connection thread
/// writes each frame as it is produced and parks inside `next` between them —
/// and dropping the iterator is how a peer that went away stops the work,
/// with nothing to cancel and no second channel to say so.
///
/// **`peer` is the connection's authorization** (REMOTE §4, §4.2; bl-8bbc,
/// bl-7ff3): the identity read off the certificate the peer presented — which
/// the engine resolves to the workspaces that client is registered in — beside
/// the grade the same subject carries, which decides what it may say at all. It
/// is a parameter rather than connection state because the answer is the only
/// thing that ever needs it, and a field would be a second copy of what the
/// certificate says.
pub trait Answerer: Send + Sync {
    fn answer(&self, peer: &Peer, request: Value) -> Box<dyn Iterator<Item = Value>>;
}

/// The listener thread. Owns its join handle and a stop flag; [`Drop`] signals
/// stop and joins, the engine's own shutdown shape (§7.2).
pub struct Listener {
    address: String,
    stop: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl Listener {
    /// Bind `m`'s address and serve `answerer` over mTLS until dropped.
    pub fn bind(
        m: &Material,
        answerer: Arc<dyn Answerer>,
        presence: Presence,
    ) -> Result<Self, String> {
        let config = super::tls::server_config(m)?;
        let tcp = TcpListener::bind(&m.address).map_err(|e| format!("bind {}: {e}", m.address))?;
        // The bound address, not the requested one: a `:0` in the file is a
        // request for whatever port is free, and the answer is what a seat
        // needs to be told.
        let address = tcp
            .local_addr()
            .map_err(|e| format!("bind {}: {e}", m.address))?
            .to_string();
        tcp.set_nonblocking(true)
            .map_err(|e| format!("bind {address}: {e}"))?;
        let stop = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&stop);
        let handle =
            std::thread::spawn(move || accept_loop(&tcp, &config, &answerer, &presence, &flag));
        Ok(Self {
            address,
            stop,
            handle: Some(handle),
        })
    }

    /// The address actually bound.
    pub fn address(&self) -> String {
        self.address.clone()
    }
}

impl Drop for Listener {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

/// Accept until stopped. Every accept error is transient by treatment — a
/// descriptor exhaustion or a peer that vanished mid-handshake is a reason to
/// look again, never a reason for the engine to stop having a wire — so there
/// is one arm and it is the same as having nothing to accept.
fn accept_loop(
    tcp: &TcpListener,
    config: &Arc<ServerConfig>,
    answerer: &Arc<dyn Answerer>,
    presence: &Presence,
    stop: &Arc<AtomicBool>,
) {
    while !stop.load(Ordering::Relaxed) {
        match tcp.accept() {
            Ok((stream, _)) => {
                let config = Arc::clone(config);
                let answerer = Arc::clone(answerer);
                let presence = presence.clone();
                std::thread::spawn(move || {
                    let _ = stream.set_nonblocking(false);
                    serve(stream, &config, answerer.as_ref(), &presence, IDLE_TIMEOUT);
                });
            }
            Err(_) => std::thread::sleep(ACCEPT_POLL),
        }
    }
}

/// One connection: handshake (inside the first read), then request → reply
/// stream → terminator, until the peer goes away or a frame refuses.
///
/// `idle` is how long a read may find nothing before the peer counts as gone —
/// [`IDLE_TIMEOUT`] is the production bound, and a test names a short one
/// rather than sleeping for real ([`Mailbox::holding`](crate::registry::mailbox::Mailbox::holding)'s
/// own shape). A socket that refuses the timeout is served without one: the
/// engine having no bound is the behaviour it had before, never a reason to
/// hang up on a peer that has done nothing wrong.
pub(crate) fn serve(
    tcp: TcpStream,
    config: &Arc<ServerConfig>,
    answerer: &dyn Answerer,
    presence: &Presence,
    idle: Duration,
) {
    let _ = tcp.set_read_timeout(Some(idle));
    let Ok(conn) = ServerConnection::new(Arc::clone(config)) else {
        return;
    };
    let mut tls = StreamOwned::new(conn, tcp);
    // The §3 version preface, stated and checked before any gesture (bl-a670).
    if !super::hello::admit(&mut tls) {
        return;
    }
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
    while let Ok(Some(request)) = frame::read_value(&mut tls) {
        // **The identity is derived per request, not held** (REMOTE §4,
        // bl-8bbc): the handshake completes inside the first read, so there is
        // no earlier moment to read a certificate at, and re-reading it is a
        // DER walk over bytes already in memory. A peer whose certificate
        // carries no name yog can use is dropped without a reply, on exactly
        // the terms an unauthenticated peer is — a connection that cannot be
        // authorized gets nothing said to it.
        let Some(peer) = peer_client(tls.conn.peer_certificates()) else {
            return;
        };
        let _ = live.get_or_insert_with(|| presence.enter(&peer.client));
        for chunk in answerer.answer(&peer, request) {
            if frame::write_value(&mut tls, &chunk).is_err() {
                return;
            }
        }
        if frame::write_end(&mut tls).is_err() {
            return;
        }
    }
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
