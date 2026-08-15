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
pub trait Answerer: Send + Sync {
    fn answer(&self, request: Value) -> Vec<Value>;
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
    pub fn bind(m: &Material, answerer: Arc<dyn Answerer>) -> Result<Self, String> {
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
        let handle = std::thread::spawn(move || accept_loop(&tcp, &config, &answerer, &flag));
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
    stop: &Arc<AtomicBool>,
) {
    while !stop.load(Ordering::Relaxed) {
        match tcp.accept() {
            Ok((stream, _)) => {
                let config = Arc::clone(config);
                let answerer = Arc::clone(answerer);
                std::thread::spawn(move || {
                    let _ = stream.set_nonblocking(false);
                    serve(stream, &config, answerer.as_ref());
                });
            }
            Err(_) => std::thread::sleep(ACCEPT_POLL),
        }
    }
}

/// One connection: handshake (inside the first read), then request → reply
/// stream → terminator, until the peer goes away or a frame refuses.
pub(crate) fn serve(tcp: TcpStream, config: &Arc<ServerConfig>, answerer: &dyn Answerer) {
    let Ok(conn) = ServerConnection::new(Arc::clone(config)) else {
        return;
    };
    let mut tls = StreamOwned::new(conn, tcp);
    while let Ok(Some(request)) = frame::read_value(&mut tls) {
        for chunk in answerer.answer(request) {
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
