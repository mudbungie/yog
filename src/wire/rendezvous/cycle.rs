//! The loop's body (bl-4263): two deadlines on the injected clock, and what
//! each does when it comes due. Split from the root at §12's cap on the seam
//! the root's doc already draws — the root is the handle and the composition,
//! this is what the thread runs.
//!
//! **A poll says its outcome only when it differs from the last poll's**
//! (bl-355c). An item sits in the inbox until the commons forgets it, so a
//! call already punched, or an item that will not verify, is read again every
//! fifteen seconds; said once, it is news, and said every poll it is the noise
//! that hides the next call. A quiet poll — no item — says nothing, and a
//! failure streak says itself once; `/doctor`'s `last_poll_unix` is the
//! liveness. A quiet poll resets that memory, so an already-punched call is
//! said once per NONCE (bl-1633): a thin swarm alternates it with nothing.

use super::accept::Toward;
use super::call::Answer;
use super::material::Pairing;
use super::{Cadence, Ctx, Say, Stats, say};
use crate::dht::{Dht, Keypair, Udp};
use crate::ui_state::Clock;
use std::net::{IpAddr, SocketAddr, ToSocketAddrs};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::Sender;
use std::time::Instant;

/// The inbox, read — its own file at §12's budget (bl-5276).
mod poll;
/// Presence, sealed and published — its own file at §12's budget.
mod publish;

pub(super) struct Cycle {
    pairing: Pairing,
    keypair: Keypair,
    inbox_key: [u8; 32],
    bootstrap: Vec<String>,
    config: crate::dht::Config,
    /// The transport, until the bootstrap resolves and the client takes it.
    transport: Option<Udp>,
    dht: Option<Dht>,
    advertise: Vec<IpAddr>,
    /// What an opened call is handed to — the punch and the serving; the
    /// acceptor is handed a clone.
    pub(super) answer: Answer,
    clock: Arc<dyn Clock>,
    cadence: Cadence,
    stats: Arc<Stats>,
    say: Say,
    /// The last poll's line, so an unchanged outcome is said once.
    last_said: Option<String>,
    /// The last sequence published, so two publishes in one second still
    /// move forward — a node refuses a `seq` that does not.
    last_seq: i64,
    /// The last call punched, so a poll that reads the same item again does
    /// not punch it again.
    last_nonce: Option<u64>,
    /// The call whose already-punched line was said — once per nonce.
    seen_said: Option<u64>,
    next_publish: Instant,
    next_poll: Instant,
}

impl Cycle {
    pub(super) fn new(ctx: Ctx, hints: Sender<Toward>) -> Result<Cycle, String> {
        let now = ctx.clock.now();
        let answer = Answer {
            punch: Arc::new(ctx.punch),
            tls: ctx.tls,
            answerer: ctx.answerer,
            presence: ctx.presence,
            window: ctx.cadence.window,
            quiet: ctx.cadence.quiet,
            stats: Arc::clone(&ctx.stats),
            say: Arc::clone(&ctx.say),
            hints,
        };
        Ok(Cycle {
            keypair: ctx.pairing.keypair()?,
            inbox_key: ctx.pairing.inbox_keypair()?.public(),
            pairing: ctx.pairing,
            bootstrap: ctx.bootstrap,
            config: ctx.config,
            transport: Some(ctx.transport),
            dht: None,
            advertise: ctx.advertise,
            answer,
            clock: ctx.clock,
            cadence: ctx.cadence,
            stats: ctx.stats,
            say: ctx.say,
            last_said: None,
            last_seq: 0,
            last_nonce: None,
            seen_said: None,
            next_publish: now,
            next_poll: now,
        })
    }

    /// The port every SYN leaves from — said at start.
    pub(super) fn punch_port(&self) -> u16 {
        self.answer.punch.port()
    }

    pub(super) fn run(mut self, stop: &AtomicBool) {
        while !stop.load(Ordering::Relaxed) {
            let now = self.clock.now();
            // The poll first: its walk is what tells a publish due at the
            // same moment — the first one, at boot — where the commons sees us.
            if now >= self.next_poll {
                let unix = self.clock.unix().max(0) as u64;
                self.stats.last_poll_unix.store(unix, Ordering::Relaxed);
                let outcome = self.poll();
                self.tell(outcome.clone().unwrap_or_else(|_| Some(say::poll_failed())));
                count(
                    outcome.map(drop),
                    &self.stats.polls,
                    &self.stats.poll_failures,
                );
                self.next_poll = now + self.cadence.poll;
            }
            if now >= self.next_publish {
                let outcome = self.publish();
                (self.say)(&outcome.clone().unwrap_or_else(|_| say::not_published()));
                count(
                    outcome.map(drop),
                    &self.stats.published,
                    &self.stats.publish_failures,
                );
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
            self.dht = Some(Dht::new(
                Box::new(transport),
                bootstrap,
                self.config.clone(),
            )?);
        }
        self.dht.as_mut().ok_or_else(|| "no DHT client".to_owned())
    }
}

/// One deadline's outcome, tallied.
fn count(outcome: Result<(), String>, ok: &AtomicUsize, failed: &AtomicUsize) {
    match outcome {
        Ok(()) => ok.fetch_add(1, Ordering::Relaxed),
        Err(_) => failed.fetch_add(1, Ordering::Relaxed),
    };
}
