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
use crate::registry::Client;
use crate::registry::presence::Presence;
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

/// What answers a request frame: the reply stream it becomes, one [`Value`] per
/// frame. Today every answer is one element long; a follow-class read is the
/// same signature with more of them (see [`frame`](super::frame)).
///
/// **`client` is the connection's authorization** (REMOTE §4, bl-8bbc): the
/// identity read off the certificate the peer presented, which the engine
/// resolves to the workspaces that client is registered in. It is a parameter
/// rather than connection state because the answer is the only thing that ever
/// needs it, and a field would be a second copy of what the certificate says.
pub trait Answerer: Send + Sync {
    fn answer(&self, client: &Client, request: Value) -> Vec<Value>;
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
                    serve(stream, &config, answerer.as_ref(), &presence);
                });
            }
            Err(_) => std::thread::sleep(ACCEPT_POLL),
        }
    }
}

/// One connection: handshake (inside the first read), then request → reply
/// stream → terminator, until the peer goes away or a frame refuses.
pub(crate) fn serve(
    tcp: TcpStream,
    config: &Arc<ServerConfig>,
    answerer: &dyn Answerer,
    presence: &Presence,
) {
    let Ok(conn) = ServerConnection::new(Arc::clone(config)) else {
        return;
    };
    let mut tls = StreamOwned::new(conn, tcp);
    // **Presence is this scope** (REMOTE §5, bl-4e08): the guard is taken when
    // the connection first names its client and released when this function
    // leaves, however it leaves — a clean close, a refused frame, a peer that
    // vanished. There is no leave verb to forget, which is what makes
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
        let Some(client) = peer_client(tls.conn.peer_certificates()) else {
            return;
        };
        let _ = live.get_or_insert_with(|| presence.enter(&client));
        for chunk in answerer.answer(&client, request) {
            if frame::write_value(&mut tls, &chunk).is_err() {
                return;
            }
        }
        if frame::write_end(&mut tls).is_err() {
            return;
        }
    }
}

/// The client identity a presented chain names (REMOTE §2): the **leaf's**
/// subject common name. The leaf is the first certificate — TLS sends the end
/// entity first and the chain toward the anchor after it, so the issuer's own
/// common name is never mistaken for the peer's.
pub(crate) fn peer_client(chain: Option<&[CertificateDer<'_>]>) -> Option<Client> {
    let name = crate::registry::leaf::common_name(chain?.first()?)?;
    Client::parse(&name).ok()
}

#[cfg(test)]
mod tests;
