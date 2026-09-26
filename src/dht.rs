//! **The DHT rendezvous client** (REMOTE §13.2, bl-df31) — a pure client of
//! the BitTorrent mainline DHT: outbound-UDP iterative lookups and BEP 44
//! signed mutable get/put, and never a node. No listener, no routing answers,
//! no storage offered — nothing for the commons to dial back, which is REMOTE
//! §13's ruling 2 applied to the rendezvous itself.
//!
//! The shape the punched-wire chain consumes is this module root: a caller
//! hands [`Dht::new`] a [`Transport`] (the std UDP socket [`Udp`], or a
//! stand-in), the bootstrap addresses and a [`Config`], then asks
//! [`Dht::lookup`] for the nodes nearest an id, [`Dht::get`] for the newest
//! item under a key and salt, or [`Dht::put`] to store one a [`Keypair`]
//! signed — and, after any of them, [`Dht::observed`] for where the commons
//! saw it come from. Everything the commons answers is untrusted until it verifies:
//! an item without a good signature under the asked-for key is not an item.
//!
//! Four files under this root, one concern each: `bencode` the encoding,
//! `krpc` the datagram shapes, `mutable` the signed item, `transport` the
//! socket seam; `lookup` is the walk and `items` the two BEP 44 verbs over
//! it. Synchronous throughout — `std::net` with socket timeouts, no tokio
//! (AGENTS.md rule 8) — and every duration is a [`Config`] field a test can
//! shorten, so the fake DHT the suite runs on loopback UDP answers in
//! milliseconds where the commons answers in seconds.

pub mod bencode;
mod items;
pub mod krpc;
mod lookup;
pub mod mutable;
pub mod transport;

pub use krpc::{Node, NodeId};
pub use mutable::{Keypair, Mutable, target_of};
pub use transport::{Transport, Udp};

use ring::rand::SecureRandom;
use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::time::Duration;

/// The walk's parameters — stated so a test can shrink them and a caller
/// can widen them. α and K are BEP 5's; the round is measured (REMOTE §13.7
/// ruling 3): on the live mainline p99 of answers landed inside 0.9 s.
#[derive(Clone, Debug)]
pub struct Config {
    /// Queries in flight per round.
    pub alpha: usize,
    /// How many closest nodes a walk converges on and a `put` writes to.
    pub k: usize,
    /// How long one round waits for its answers.
    pub round: Duration,
    /// The most queries one walk may send, however the commons answers.
    pub max_queries: usize,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            alpha: 3,
            k: 8,
            round: Duration::from_secs(1),
            max_queries: 64,
        }
    }
}

/// One client: a transport, the nodes it starts from, and the id it queries
/// as. The id is random per client — a client is not a node and nobody
/// routes by it, so nothing is lost by minting a fresh one every run.
pub struct Dht {
    transport: Box<dyn Transport>,
    bootstrap: Vec<SocketAddr>,
    config: Config,
    id: NodeId,
    tid: u16,
    /// What the last walk's answering nodes said this client's address is
    /// (BEP 42), one claim per node — raw, because [`Dht::observed`] is the
    /// vote over them and a vote is computed, not kept.
    claims: Vec<SocketAddr>,
}

impl Dht {
    /// A client over `transport`, starting every walk from `bootstrap`.
    pub fn new(
        transport: Box<dyn Transport>,
        bootstrap: Vec<SocketAddr>,
        config: Config,
    ) -> Result<Dht, String> {
        let mut id = [0u8; 20];
        random(&mut id)?;
        Ok(Dht {
            transport,
            bootstrap,
            config,
            id: NodeId(id),
            tid: 0,
            claims: Vec::new(),
        })
    }

    /// The id this client queries as.
    pub fn id(&self) -> NodeId {
        self.id
    }

    /// The nodes nearest `target`, closest first — BEP 5's `find_node` walk.
    pub fn lookup(&mut self, target: NodeId) -> Result<Vec<Node>, String> {
        let out = self.search(target, "find_node")?;
        Ok(out
            .replies
            .into_iter()
            .map(|(n, _)| n)
            .take(self.config.k)
            .collect())
    }

    /// Where the commons sees this client, per address family, as the last
    /// walk's answering nodes voted (BEP 42's `ip`): the endpoint the most of
    /// them named, and nothing for a family whose top count is tied or that
    /// no node spoke to. One node's claim is only a claim — a single liar
    /// can outvote nobody. The vote is over the whole endpoint: nodes that
    /// agree on the address but not the port are seeing a mapping that moves
    /// per destination, and no one port of it is this client's.
    pub fn observed(&self) -> Vec<SocketAddr> {
        [true, false]
            .into_iter()
            .filter_map(|v4| plurality(self.claims.iter().copied().filter(|a| a.is_ipv4() == v4)))
            .collect()
    }

    /// The next transaction id: two bytes, wrapping, never reused within a
    /// walk (a walk sends at most `max_queries`, far under 65 536).
    pub(crate) fn next_tid(&mut self) -> Vec<u8> {
        self.tid = self.tid.wrapping_add(1);
        self.tid.to_be_bytes().to_vec()
    }
}

/// The one claim named more often than any other, or nothing.
fn plurality(claims: impl Iterator<Item = SocketAddr>) -> Option<SocketAddr> {
    let mut counts: BTreeMap<SocketAddr, usize> = BTreeMap::new();
    for claim in claims {
        *counts.entry(claim).or_default() += 1;
    }
    let top = counts.values().max()?;
    let mut leaders = counts.iter().filter(|(_, n)| *n == top);
    let (winner, _) = leaders.next()?;
    leaders.next().is_none().then_some(*winner)
}

/// Fill `buf` from the system's randomness.
pub(crate) fn random(buf: &mut [u8]) -> Result<(), String> {
    ring::rand::SystemRandom::new()
        .fill(buf)
        .map_err(|_| "the system offered no randomness".to_string())
}

#[cfg(test)]
pub(crate) mod tests;
