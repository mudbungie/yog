//! **What the loop says on the engine's stderr** (REMOTE §13.4, bl-355c): one
//! line per event, in the `yog: wire: listening on …` shape every other
//! subsystem already speaks. Each line is built here and nowhere else, so the
//! one rule they share is enforced by the only file that could break it:
//!
//! **counts, sequence numbers, nonces and address families — never an
//! address, a key, a salt or a sealed byte.** An engine's log lands in a
//! public CI log in some deployments. It is also why a DHT failure is said
//! without its reason: the walk's refusals name the nodes it asked and the
//! target it walked toward, a derivation of the key and the salt.
//!
//! The sink is injected ([`Say`]) so the suite reads the lines the loop
//! emits; the engine's is [`stderr`].

use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;

/// Where the loop's lines go.
pub(crate) type Say = Arc<dyn Fn(&str) + Send + Sync>;

/// The engine's sink: the process's stderr, a line at a time.
pub(crate) fn stderr() -> Say {
    Arc::new(|line: &str| eprintln!("{line}"))
}

const P: &str = "yog: rendezvous:";

pub(crate) fn started(punch_port: u16, dht_port: u16) -> String {
    format!("{P} started — punch port {punch_port}, DHT socket port {dht_port}")
}

pub(crate) fn published(seq: i64, endpoints: usize, acks: usize) -> String {
    format!("{P} presence published — seq {seq}, {endpoints} endpoint(s), {acks} ack(s)")
}

pub(crate) fn not_published() -> String {
    format!("{P} presence not published — the DHT walk failed (reason withheld: it names nodes)")
}

pub(crate) fn poll_failed() -> String {
    format!("{P} inbox poll failed — the DHT walk failed (reason withheld: it names nodes)")
}

pub(crate) fn unverified(seq: i64) -> String {
    format!("{P} inbox item seq {seq} did not verify under the pairing's seal key — no punch")
}

pub(crate) fn unopened(seq: i64) -> String {
    format!("{P} inbox item seq {seq} verified but is not a call — no punch")
}

pub(crate) fn seen(nonce: u64) -> String {
    format!("{P} call nonce {nonce} already punched — no punch")
}

pub(crate) fn opened(nonce: u64, endpoints: &[SocketAddr]) -> String {
    let ips: Vec<IpAddr> = endpoints.iter().map(SocketAddr::ip).collect();
    format!(
        "{P} call nonce {nonce} opened — {} endpoint(s) ({}) — punch started",
        endpoints.len(),
        families(&ips)
    )
}

pub(crate) fn landed(nonce: u64, peers: &[IpAddr]) -> String {
    format!(
        "{P} punch for nonce {nonce} landed {} stream(s) ({})",
        peers.len(),
        families(peers)
    )
}

pub(crate) fn expired(nonce: u64, window: Duration) -> String {
    format!(
        "{P} punch for nonce {nonce} expired after {}s with no stream",
        window.as_secs()
    )
}

pub(crate) fn ended(nonce: u64) -> String {
    format!("{P} served stream for nonce {nonce} ended")
}

/// How many of `ips` are of each family, v6 first — `1 v6, 2 v4`, or `none`.
fn families(ips: &[IpAddr]) -> String {
    let v6 = ips.iter().filter(|ip| ip.is_ipv6()).count();
    let v4 = ips.len() - v6;
    let parts: Vec<String> = [(v6, "v6"), (v4, "v4")]
        .into_iter()
        .filter(|(n, _)| *n > 0)
        .map(|(n, family)| format!("{n} {family}"))
        .collect();
    if parts.is_empty() {
        "none".to_owned()
    } else {
        parts.join(", ")
    }
}

#[cfg(test)]
mod tests;
