//! The hourly half of the loop (REMOTE §13.2): presence sealed under the
//! pairing and published under the rendezvous key. Split off `cycle` at §12's
//! budget (bl-355c) on the seam the two deadlines already draw.

use super::super::item::Presence as Published;
use super::super::say;
use super::Cycle;
use std::net::SocketAddr;

impl Cycle {
    /// Seal and publish presence under the rendezvous key: the route-local
    /// addresses, then every address the last walk's nodes agreed they saw
    /// us at (`Dht::observed`) that is not one of them — all at the punch
    /// port. The observed PORT is the DHT socket's UDP mapping, not the
    /// punch port's TCP one, so only the address is taken and port
    /// preservation is trusted (REMOTE §13.8 measured it at home; a carrier
    /// that rewrites the port is the case this does not reach). Answers the
    /// line that says it.
    pub(super) fn publish(&mut self) -> Result<String, String> {
        let observed = self.dht()?.observed();
        let mut ips = self.advertise.clone();
        for ip in observed.iter().map(SocketAddr::ip) {
            if !ips.contains(&ip) {
                ips.push(ip);
            }
        }
        let seq = self.clock.unix().max(self.last_seq + 1);
        let port = self.punch_port();
        let endpoints: Vec<SocketAddr> = ips
            .into_iter()
            .map(|ip| SocketAddr::new(ip, port))
            .collect();
        let count = endpoints.len();
        let sealed = Published { endpoints }.seal(&self.pairing.seal_key())?;
        let item = self
            .keypair
            .sign(self.pairing.presence_salt(), seq, sealed)?;
        let acks = self.dht()?.put(item)?;
        self.last_seq = seq;
        Ok(say::published(seq, count, acks))
    }
}
