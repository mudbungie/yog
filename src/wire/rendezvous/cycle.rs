//! The loop's body (bl-4263): two deadlines on the injected clock, and what
//! each does when it comes due. Split from the root at §12's cap on the seam
//! the root's doc already draws — the root is the handle and the composition,
//! this is what the thread runs.

use super::item::{Call, Presence as Published};
use super::material::Pairing;
use super::{Cadence, Ctx, Punch, Stats};
use crate::dht::{Dht, Keypair, Transport};
use crate::registry::presence::Presence;
use crate::ui_state::Clock;
use crate::wire::server::{Answerer, serve};
use rustls::ServerConfig;
use std::net::{IpAddr, SocketAddr, ToSocketAddrs};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Instant;

pub(super) struct Cycle {
    pairing: Pairing,
    keypair: Keypair,
    inbox_key: [u8; 32],
    bootstrap: Vec<String>,
    config: crate::dht::Config,
    /// The transport, until the bootstrap resolves and the client takes it.
    transport: Option<Box<dyn Transport>>,
    dht: Option<Dht>,
    punch: Arc<Punch>,
    advertise: Vec<IpAddr>,
    tls: Arc<ServerConfig>,
    answerer: Arc<dyn Answerer>,
    presence: Presence,
    clock: Arc<dyn Clock>,
    cadence: Cadence,
    stats: Arc<Stats>,
    /// The last sequence published, so two publishes in one second still
    /// move forward — a node refuses a `seq` that does not.
    last_seq: i64,
    /// The last call punched, so a poll that reads the same item again does
    /// not punch it again.
    last_nonce: Option<u64>,
    next_publish: Instant,
    next_poll: Instant,
}

impl Cycle {
    pub(super) fn new(ctx: Ctx, stats: Arc<Stats>) -> Result<Cycle, String> {
        let now = ctx.clock.now();
        Ok(Cycle {
            keypair: ctx.pairing.keypair()?,
            inbox_key: ctx.pairing.inbox_keypair()?.public(),
            pairing: ctx.pairing,
            bootstrap: ctx.bootstrap,
            config: ctx.config,
            transport: Some(ctx.transport),
            dht: None,
            punch: Arc::new(ctx.punch),
            advertise: ctx.advertise,
            tls: ctx.tls,
            answerer: ctx.answerer,
            presence: ctx.presence,
            clock: ctx.clock,
            cadence: ctx.cadence,
            stats,
            last_seq: 0,
            last_nonce: None,
            next_publish: now,
            next_poll: now,
        })
    }

    pub(super) fn run(mut self, stop: &AtomicBool) {
        while !stop.load(Ordering::Relaxed) {
            let now = self.clock.now();
            // The poll first: its walk is what tells a publish due at the
            // same moment — the first one, at boot — where the commons sees us.
            if now >= self.next_poll {
                let outcome = self.poll();
                count(outcome, &self.stats.polls, &self.stats.poll_failures);
                self.next_poll = now + self.cadence.poll;
            }
            if now >= self.next_publish {
                let outcome = self.publish();
                count(outcome, &self.stats.published, &self.stats.publish_failures);
                self.next_publish = now + self.cadence.publish;
            }
            std::thread::sleep(self.cadence.tick);
        }
    }

    /// The DHT client, built on first use once the bootstrap resolves —
    /// a box that boots offline resolves nothing and tries again at the next
    /// deadline rather than never.
    fn dht(&mut self) -> Result<&mut Dht, String> {
        if self.dht.is_none() {
            let bootstrap: Vec<SocketAddr> = self
                .bootstrap
                .iter()
                .filter_map(|name| name.to_socket_addrs().ok())
                .flatten()
                .collect();
            if bootstrap.is_empty() {
                return Err("no bootstrap node resolved".to_owned());
            }
            let transport = self.transport.take().ok_or("the transport is spent")?;
            self.dht = Some(Dht::new(transport, bootstrap, self.config.clone())?);
        }
        self.dht.as_mut().ok_or_else(|| "no DHT client".to_owned())
    }

    /// Seal and publish presence under the rendezvous key: the route-local
    /// addresses, then every address the last walk's nodes agreed they saw
    /// us at (`Dht::observed`) that is not one of them — all at the punch
    /// port. The observed PORT is the DHT socket's UDP mapping, not the
    /// punch port's TCP one, so only the address is taken and port
    /// preservation is trusted (REMOTE §13.8 measured it at home; a carrier
    /// that rewrites the port is the case this does not reach).
    fn publish(&mut self) -> Result<(), String> {
        let observed = self.dht()?.observed();
        let mut ips = self.advertise.clone();
        for ip in observed.iter().map(SocketAddr::ip) {
            if !ips.contains(&ip) {
                ips.push(ip);
            }
        }
        let seq = self.clock.unix().max(self.last_seq + 1);
        let port = self.punch.port();
        let endpoints = ips
            .into_iter()
            .map(|ip| SocketAddr::new(ip, port))
            .collect();
        let sealed = Published { endpoints }.seal(&self.pairing.seal_key())?;
        let item = self
            .keypair
            .sign(self.pairing.presence_salt(), seq, sealed)?;
        self.dht()?.put(item)?;
        self.last_seq = seq;
        Ok(())
    }

    /// Read the inbox; a call that verifies, unseals and is new is punched
    /// on its own thread, and every stream that lands is served.
    fn poll(&mut self) -> Result<(), String> {
        let (key, salt) = (self.inbox_key, self.pairing.inbox_salt());
        let Some(item) = self.dht()?.get(key, salt)? else {
            return Ok(());
        };
        let Some(call) = Call::open(&self.pairing.seal_key(), &item.value) else {
            return Ok(());
        };
        if self.last_nonce == Some(call.nonce) {
            return Ok(());
        }
        self.last_nonce = Some(call.nonce);
        self.stats.calls.fetch_add(1, Ordering::Relaxed);
        let (punch, tls, answerer) = (
            Arc::clone(&self.punch),
            Arc::clone(&self.tls),
            Arc::clone(&self.answerer),
        );
        let (presence, cadence, stats) =
            (self.presence.clone(), self.cadence, Arc::clone(&self.stats));
        std::thread::spawn(move || {
            for stream in punch.punch(call.endpoints, cadence.window) {
                stats.served.fetch_add(1, Ordering::Relaxed);
                let (tls, answerer, presence) =
                    (Arc::clone(&tls), Arc::clone(&answerer), presence.clone());
                std::thread::spawn(move || {
                    serve(stream, &tls, answerer.as_ref(), &presence, cadence.quiet);
                });
            }
        });
        Ok(())
    }
}

/// One deadline's outcome, tallied.
fn count(outcome: Result<(), String>, ok: &AtomicUsize, failed: &AtomicUsize) {
    match outcome {
        Ok(()) => ok.fetch_add(1, Ordering::Relaxed),
        Err(_) => failed.fetch_add(1, Ordering::Relaxed),
    };
}
