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
use crate::registry::Peer;
use crate::registry::presence::Presence;
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

/// How often a **held** connection is pinged through its silence (REMOTE
/// REMOTE §13.4, bl-4263) — under the shortest NAT mapping lifetime worth planning
/// for, and a stated default to be revisited on evidence (REMOTE §13.7 ruling 3).
pub(crate) const PING: Duration = Duration::from_secs(25);

/// Who the peer is, how its silence is read, and how it is hung up on.
pub(crate) mod peer;
pub(crate) use peer::{Quiet, peer_client};

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
                    serve(
                        stream,
                        &config,
                        answerer.as_ref(),
                        &presence,
                        Quiet::dialled(),
                    );
                });
            }
            Err(_) => std::thread::sleep(ACCEPT_POLL),
        }
    }
}

impl Quiet {
    /// A dialled connection: two minutes of nothing is a peer that vanished.
    pub(crate) fn dialled() -> Quiet {
        Quiet {
            gone: IDLE_TIMEOUT,
            ping: None,
        }
    }

    /// A punched connection, held between asks and pinged through its silence.
    pub(crate) fn held() -> Quiet {
        Quiet {
            gone: IDLE_TIMEOUT,
            ping: Some(PING),
        }
    }
}

/// One connection: handshake (inside the first read), then request → reply
/// stream → terminator, until the peer goes away or a frame refuses.
///
/// `quiet` is how long a read may find nothing before the peer counts as gone
/// — [`Quiet::dialled`] is the production bound, and a test names a short one
/// rather than sleeping for real ([`Mailbox::holding`](crate::registry::mailbox::Mailbox::holding)'s
/// own shape) — and, on a held connection, how often the silence is pinged
/// ([`peer`]). A socket that refuses the timeout is served without one: the
/// engine having no bound is the behaviour it had before, never a reason to
/// hang up on a peer that has done nothing wrong.
pub(crate) fn serve(
    tcp: TcpStream,
    config: &Arc<ServerConfig>,
    answerer: &dyn Answerer,
    presence: &Presence,
    quiet: Quiet,
) {
    let _ = tcp.set_read_timeout(Some(quiet.read_timeout()));
    let Ok(conn) = ServerConnection::new(Arc::clone(config)) else {
        return;
    };
    let mut tls = StreamOwned::new(conn, tcp);
    // The §3 version preface, stated and checked before any gesture (bl-a670).
    // What an admitted peer hands back is its corpus EDITION (REMOTE §3.2,
    // bl-1be7) — stated, never adjudicated — which rides on the presence entry
    // below because that is where this connection's identity already lives.
    let Some(edition) = super::hello::admit(&mut tls) else {
        peer::hang_up(&mut tls);
        return;
    };
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
        let request = match frame::read_value(&mut tls) {
            Ok(Some(request)) => request,
            // A held connection's silence is pinged, one timeout at a time,
            // until the bound says the peer is gone; every other end is the
            // peer's own — an EOF, a refused frame, a vanished socket.
            Err(e) if quiet.pings_at(quiet_for, &e) && peer::ping(&mut tls) => {
                quiet_for += quiet.read_timeout();
                continue;
            }
            _ => return,
        };
        quiet_for = Duration::ZERO;
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
        let _ = live.get_or_insert_with(|| presence.enter(&peer.client, edition));
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

#[cfg(test)]
mod tests;
