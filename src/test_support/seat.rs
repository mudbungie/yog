//! **The suite's own seat** — the client half of the wire, kept as test
//! scaffolding when the seat crate took the shipping one (bl-7942).
//!
//! A server whose listener nothing ever dials is a listener nothing proves, so
//! the crate keeps one client: this. It is deliberately the SAME code the seat
//! was built from (REMOTE §8, §9.5; bl-b6fa) — a re-implementation here would
//! prove the re-implementation — and it lives under `test_support` rather than
//! in `wire` because it is not a face yog ships and must not read as one.
//!
//! What a client of the engine holds, and the only thing it holds:
//!
//! A client owns its key material and RAM, nothing else (REMOTE §6) — so this
//! is a configuration and an address, and every ask is its own TCP connection
//! and its own handshake. **One connection per gesture** is the shape §3's
//! cadence ruling asks for: *"the seat polls"*, at human cadence, and a held
//! connection is an optimization of that same surface rather than a different
//! one (REMOTE §10 keeps it open as a question).
//!
//! **The server's name comes from the address, never from a second knob.** A
//! dotted quad or a bracketed v6 literal is verified as an IP address — the
//! server leaf must carry the matching `IP:` SAN — and anything else is a DNS
//! name. There is nothing to configure and nothing that can disagree with what
//! was dialled.

use crate::boundary::reply::Reply;
use crate::wire::frame;
use crate::wire::material::Material;
use rustls::pki_types::ServerName;
use rustls::{ClientConfig, ClientConnection, StreamOwned};
use serde_json::Value;
use std::net::{IpAddr, TcpStream};
use std::sync::Arc;
use std::time::Duration;

/// What one gesture earned: the decoded reply, or the one sentence that says
/// why there is none. A refusal, an unreadable answer and a socket that never
/// opened are the same fact to a caller, so a reader carries no case for which
/// layer said so.
pub(crate) type Landed = Result<Reply, String>;

/// How long a seat waits on one answer before giving up on the connection.
const ASK_TIMEOUT: Duration = Duration::from_mins(2);

/// A seat's end of the wire.
pub(crate) struct Seat {
    config: Arc<ClientConfig>,
    address: String,
    name: ServerName<'static>,
}

impl Seat {
    /// Build the seat from provisioned material. Nothing is dialled here: a
    /// seat is a fact about what this machine may say, not about whether an
    /// engine happens to be up.
    pub(crate) fn open(m: &Material) -> Result<Self, String> {
        Ok(Self {
            config: crate::wire::tls::client_config(m)?,
            address: m.address.clone(),
            name: server_name(&m.address)?,
        })
    }

    /// The address this seat dials.
    pub(crate) fn address(&self) -> String {
        self.address.clone()
    }

    /// Send one request envelope and read its whole reply stream — every frame
    /// up to the terminator. A stream of one is the ordinary answer.
    pub(crate) fn ask(&self, request: &Value) -> Result<Vec<Value>, String> {
        let mut tls = self.dial(request)?;
        let mut stream = Vec::new();
        loop {
            match frame::read_value(&mut tls).map_err(|e| format!("receive: {e}"))? {
                Some(chunk) => stream.push(chunk),
                None => return Ok(stream),
            }
        }
    }

    /// **Ask, and stay on the line** (REMOTE §3, §10; bl-73e7) — the same one
    /// request, with each frame decoded and handed over *as it arrives* rather
    /// than collected. This is the held connection §10 kept as a question and
    /// the follow lane finally pays for: no second envelope, no second reader,
    /// and nothing here that [`ask`](Self::ask) does not already do — the whole
    /// difference is that the caller is given the frames instead of the list.
    ///
    /// `on_frame` answers whether to stay: `false` ends the read, which is how
    /// a lane whose subject moved stops without a word to the engine (dropping
    /// the connection is the word). `Ok(())` is the engine terminating the
    /// stream — the ordinary end, not an event.
    pub(crate) fn followed(
        &self,
        request: &Value,
        on_frame: &mut dyn FnMut(Landed) -> bool,
    ) -> Result<(), String> {
        let mut tls = self.dial(request)?;
        while let Some(chunk) = frame::read_value(&mut tls).map_err(|e| format!("receive: {e}"))? {
            if !on_frame(crate::boundary::reply::decode(&chunk).unwrap_or_else(Err)) {
                return Ok(());
            }
        }
        Ok(())
    }

    /// Connect, handshake and send `request` — the half both spellings share.
    /// The TLS handshake happens inside the first write, and the one frame read
    /// here is the engine's version preface (bl-a670), so what this hands back
    /// is a socket with an envelope on it and no *answer* yet read.
    fn dial(&self, request: &Value) -> Result<StreamOwned<ClientConnection, TcpStream>, String> {
        let tcp = TcpStream::connect(&self.address)
            .map_err(|e| format!("connect {}: {e}", self.address))?;
        tcp.set_read_timeout(Some(ASK_TIMEOUT))
            .map_err(|e| format!("connect {}: {e}", self.address))?;
        let conn = ClientConnection::new(Arc::clone(&self.config), self.name.clone())
            .map_err(|e| format!("tls {}: {e}", self.address))?;
        let mut tls = StreamOwned::new(conn, tcp);
        // **Both ends state a protocol version before either reads** (REMOTE
        // §3, bl-a670). The request goes out in the same breath as the
        // preface, so confirming the engine's costs no round trip — and a
        // mismatch refuses here, before a frame of the answer is decoded.
        crate::wire::hello::state(&mut tls).map_err(|e| format!("send: {e}"))?;
        frame::write_value(&mut tls, request).map_err(|e| format!("send: {e}"))?;
        crate::wire::hello::confirm(&mut tls)?;
        Ok(tls)
    }
}

/// The name to verify the server certificate against, read off the address.
fn server_name(address: &str) -> Result<ServerName<'static>, String> {
    let host = address.rsplit_once(':').map_or(address, |(head, _)| head);
    let host = host.trim_start_matches('[').trim_end_matches(']');
    if let Ok(ip) = host.parse::<IpAddr>() {
        return Ok(ServerName::IpAddress(ip.into()));
    }
    ServerName::try_from(host.to_owned()).map_err(|e| format!("{address}: not a server name: {e}"))
}

/// **Loopback at the port the listener really bound**, whatever the `address`
/// file says the engine answers to off the box.
///
/// A local client is a client of `127.0.0.1` and of nothing else, which is why
/// [`provision`](crate::wire::provision) always puts loopback on the server
/// leaf. The **bound** port rather than the requested one: a `:0` in the file
/// is a request, and only the listener knows what it became.
pub(crate) fn loopback(bound: &str) -> String {
    let port = bound.rsplit_once(':').map_or("", |(_, port)| port);
    format!("{}:{port}", crate::wire::provision::LOOPBACK)
}

#[cfg(test)]
mod tests;
