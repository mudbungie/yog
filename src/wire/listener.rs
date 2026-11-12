//! The server's listener (REMOTE §8, §9.5; bl-b6fa): a synchronous mTLS accept
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
//! rest of the server already uses — a stop flag, a loop, a `Drop` that joins
//! — and it costs one thread per live seat, which is the number of seats an
//! operator has.
//!
//! **A connection with no certificate is not the wire** (REMOTE §4, §15.2).
//! The handshake is driven to its end before anything is read, and a peer that
//! presented nothing never reaches [`frame`](super::frame): it is offered the
//! registration [`door`] — one HTTP request, answered and closed — and a peer
//! whose certificate the operator CA did not issue fails the handshake itself.

use super::material::Material;
use crate::registry::Peer;
use crate::registry::presence::Presence;
use rustls::ServerConfig;
use serde_json::Value;
use std::net::TcpListener;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;
use std::time::Duration;

/// How often the accept loop looks for a connection. A latency knob on
/// *shutdown*, not on connections: the socket backlog holds an arriving seat
/// until the next look.
const ACCEPT_POLL: Duration = Duration::from_millis(20);

/// What an address asks the server to bind (REMOTE §15.3).
mod binding;
pub(crate) use binding::binding;

/// Who the peer is, how long its silence is waited on, and how it is hung up on.
pub(crate) mod peer;

/// One accepted connection: the handshake, then the wire or the door.
mod connection;
use connection::dialled;

/// What a connection with no certificate may say (REMOTE §15.2).
pub(crate) mod door;

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
/// the server resolves to the workspaces that client is registered in — beside
/// the grade the same subject carries, which decides what it may say at all. It
/// is a parameter rather than connection state because the answer is the only
/// thing that ever needs it, and a field would be a second copy of what the
/// certificate says.
pub trait Answerer: Send + Sync {
    fn answer(&self, peer: &Peer, request: Value) -> Box<dyn Iterator<Item = Value>>;

    /// **What a certless peer's one registration earns** (REMOTE §15.2,
    /// bl-9ea6): the claim's JSON object, or `None` — which the door says as a
    /// bare `404`. The default is `None`, so an answerer that holds no offers
    /// is a door that opens for nobody. It is a second method rather than a
    /// request through [`answer`](Self::answer) because that one takes a
    /// [`Peer`], and this caller is not one.
    fn register(&self, _code: &str, _body: &[u8]) -> Option<Value> {
        None
    }
}

/// The listener thread. Owns its join handle and a stop flag; [`Drop`] signals
/// stop and joins, the server's own shutdown shape (§7.2).
pub struct Listener {
    address: String,
    stop: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl Listener {
    /// Bind what `m`'s address asks for ([`binding`]) and serve `answerer` over
    /// mTLS until dropped.
    pub fn bind(
        m: &Material,
        answerer: Arc<dyn Answerer>,
        presence: Presence,
    ) -> Result<Self, String> {
        let config = super::tls::server_config(m)?;
        let tcp = TcpListener::bind(binding(&m.address)?)
            .map_err(|e| format!("bind {}: {e}", m.address))?;
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
/// look again, never a reason for the server to stop having a wire — so there
/// is one arm and it is the same as having nothing to accept.
fn accept_loop(
    tcp: &TcpListener,
    config: &Arc<ServerConfig>,
    answerer: &Arc<dyn Answerer>,
    presence: &Presence,
    stop: &Arc<AtomicBool>,
) {
    let slots = door::Slots::default();
    while !stop.load(Ordering::Relaxed) {
        match tcp.accept() {
            Ok((stream, _)) => {
                let config = Arc::clone(config);
                let answerer = Arc::clone(answerer);
                let presence = presence.clone();
                let slots = slots.clone();
                std::thread::spawn(move || {
                    let _ = stream.set_nonblocking(false);
                    dialled(
                        stream,
                        &config,
                        answerer.as_ref(),
                        &presence,
                        &slots,
                        peer::IDLE_TIMEOUT,
                    );
                });
            }
            Err(_) => std::thread::sleep(ACCEPT_POLL),
        }
    }
}

#[cfg(test)]
mod tests;
