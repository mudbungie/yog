//! The seat's transport (REMOTE §8, §9.5; bl-b6fa): what a client of the
//! engine holds, and the only thing it holds.
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

use super::frame;
use super::material::Material;
use rustls::pki_types::ServerName;
use rustls::{ClientConfig, ClientConnection, StreamOwned};
use serde_json::Value;
use std::net::{IpAddr, TcpStream};
use std::sync::Arc;
use std::time::Duration;

/// How long a seat waits on one answer before giving up on the connection.
const ASK_TIMEOUT: Duration = Duration::from_mins(2);

/// A seat's end of the wire.
pub struct Seat {
    config: Arc<ClientConfig>,
    address: String,
    name: ServerName<'static>,
}

impl Seat {
    /// Build the seat from provisioned material. Nothing is dialled here: a
    /// seat is a fact about what this machine may say, not about whether an
    /// engine happens to be up.
    pub fn open(m: &Material) -> Result<Self, String> {
        Ok(Self {
            config: super::tls::client_config(m)?,
            address: m.address.clone(),
            name: server_name(&m.address)?,
        })
    }

    /// The address this seat dials.
    pub fn address(&self) -> String {
        self.address.clone()
    }

    /// Ask once and decode the answer — **the last frame of the stream**, which
    /// today is the only frame and tomorrow is a follow-class read's newest
    /// state.
    ///
    /// One `Err` for a refusal, an unreadable answer and a socket that never
    /// opened alike (REMOTE §9.7): all three are the same fact to a caller —
    /// this cannot be painted, and here is the sentence. Spent by both of the
    /// window's off-frame threads, the [`asker`](super::asker) reading and the
    /// [`poster`](super::poster) acting, because "what one gesture earned" is
    /// one function and not one per direction.
    pub fn answered(&self, request: &Value) -> crate::wire::link::Landed {
        let stream = self.ask(request)?;
        let last = stream
            .last()
            .ok_or_else(|| "the engine ended the stream without answering".to_owned())?;
        crate::boundary::reply::decode(last).unwrap_or_else(Err)
    }

    /// Send one request envelope and read its whole reply stream — every frame
    /// up to the terminator. A stream of one is the ordinary answer.
    pub fn ask(&self, request: &Value) -> Result<Vec<Value>, String> {
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
    pub fn followed(
        &self,
        request: &Value,
        on_frame: &mut dyn FnMut(crate::wire::link::Landed) -> bool,
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
    /// The handshake happens inside the first read, so what this hands back is
    /// a socket with an envelope on it and nothing yet read.
    fn dial(&self, request: &Value) -> Result<StreamOwned<ClientConnection, TcpStream>, String> {
        let tcp = TcpStream::connect(&self.address)
            .map_err(|e| format!("connect {}: {e}", self.address))?;
        tcp.set_read_timeout(Some(ASK_TIMEOUT))
            .map_err(|e| format!("connect {}: {e}", self.address))?;
        let conn = ClientConnection::new(Arc::clone(&self.config), self.name.clone())
            .map_err(|e| format!("tls {}: {e}", self.address))?;
        let mut tls = StreamOwned::new(conn, tcp);
        frame::write_value(&mut tls, request).map_err(|e| format!("send: {e}"))?;
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

#[cfg(test)]
mod tests;
