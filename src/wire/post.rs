//! **The frame's half of the act path** (REMOTE §1.2, §9.8; bl-4841): what the
//! window has sent over the wire, and what came back for it.
//!
//! The read half ([`link`](super::link)) keys a standing question by its own
//! encoded envelope, because asking twice is asking once. **An act cannot be
//! keyed that way.** A gesture is not idempotent — two clicks of Nudge are two
//! nudges, and a resend is never free — so the envelope is not a handle, and
//! nothing about the act's own bytes can be. Something has to *mint* one:
//! [`Ticket`], a number from a counter the frame owns, minted at the send and
//! spent at the receipt. It is what survives the repaints in between, because
//! the surface that fired holds it in its own RAM while the frame it was
//! clicked in is long gone.
//!
//! **Every ticket earns exactly one receipt, so there is no "never came".** The
//! poster is the only thing that can answer, and it answers on every path it
//! has: the engine's reply, the engine's refusal, a decode it could not read, a
//! socket it could not open. A send that cannot even reach the poster — a window
//! whose engine minted no material, so nothing is behind this end of the channel
//! — is answered *in the send*, with the same one `Err` a refusal is. No
//! timeout, no clock and no expiry sweep: the one bound that exists is
//! [`Seat`](super::client::Seat)'s own read timeout, which turns an engine that
//! has stopped answering into a sentence.
//!
//! **The receipts a nobody holds are dropped.** A landed receipt waits to be
//! read, and the map is bounded at [`RECEIPTS_KEPT`] by dropping the oldest
//! ticket: a receipt still unread after that many later gestures has no holder,
//! and a bound is what makes that a fact rather than a hope.

use super::link::Landed;
use serde_json::Value;
use std::collections::BTreeMap;
use std::sync::mpsc::{Receiver, Sender, channel};

/// How many landed-but-unread receipts are kept. A receipt is normally taken
/// the frame after it lands; this is the ceiling on the ones nobody ever asks
/// for, so the map is bounded by construction rather than by every caller
/// remembering to collect.
pub const RECEIPTS_KEPT: usize = 64;

/// What a window with nothing behind it says. The same one `Err` a refusal is
/// (REMOTE §9.8): a frame cannot paint the act's answer, and here is why.
/// `pub(crate)` since bl-dc14: the wireless window's whole-frame refusal
/// (`shell::refusal`) heads itself with the same sentence every act receipt
/// carries — one sentence, one home.
pub(crate) const NO_WIRE: &str = "this window has no wire behind it";

/// **An act's receipt identity** — minted at the send, spent at the read.
///
/// Opaque and mintable only by [`Post::send`], which is what makes "one act,
/// one receipt" structural: nothing else can name a ticket it did not earn.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Ticket(u64);

/// The window's end: what it has sent and what has landed for it.
pub struct Post {
    acts: Sender<(Ticket, Value)>,
    receipts: Receiver<(Ticket, Landed)>,
    next: u64,
    landed: BTreeMap<Ticket, Landed>,
}

/// The poster's end: the acts to send, and where their receipts go.
pub struct Outbox {
    acts: Receiver<(Ticket, Value)>,
    receipts: Sender<(Ticket, Landed)>,
}

/// A fresh pair, minted together for [`link::pair`](super::link::pair)'s reason
/// exactly: neither end is useful alone, so neither can be attached later.
pub fn pair() -> (Post, Outbox) {
    let (a_tx, a_rx) = channel();
    let (r_tx, r_rx) = channel();
    (
        Post {
            acts: a_tx,
            receipts: r_rx,
            next: 0,
            landed: BTreeMap::new(),
        },
        Outbox {
            acts: a_rx,
            receipts: r_tx,
        },
    )
}

/// **A post nobody sends.** The model holds one from the moment it boots and
/// the engine hands it a live one when there is a wire to send over — so firing
/// a gesture is the same call whether or not this box got a listener up, and
/// what a surface paints is the sentence rather than a branch.
impl Default for Post {
    fn default() -> Self {
        pair().0
    }
}

impl Post {
    /// Send one act and hand back the ticket its receipt will arrive under.
    /// Never blocks: the poster does the dialling.
    pub fn send(&mut self, act: &Value) -> Ticket {
        let ticket = Ticket(self.next);
        self.next = self.next.wrapping_add(1);
        if self.acts.send((ticket, act.clone())).is_err() {
            self.keep(ticket, Err(NO_WIRE.to_owned()));
        }
        ticket
    }

    /// Take whatever has landed since the last call, and say which tickets it
    /// was — the caller's chance to do an act's own aftermath once, at the
    /// moment it actually happened rather than at the moment it was asked for.
    pub fn settle(&mut self) -> Vec<Ticket> {
        let arrived: Vec<(Ticket, Landed)> = self.receipts.try_iter().collect();
        let tickets = arrived.iter().map(|(ticket, _)| *ticket).collect();
        for (ticket, landed) in arrived {
            self.keep(ticket, landed);
        }
        tickets
    }

    /// Take one act's receipt, if it has landed. Spent by the read: a receipt
    /// is one act's answer, and the surface that reads it holds the ticket no
    /// longer.
    pub fn receipt(&mut self, ticket: Ticket) -> Option<Landed> {
        self.landed.remove(&ticket)
    }

    /// Hold a receipt for its reader, oldest first out when the map is full.
    fn keep(&mut self, ticket: Ticket, landed: Landed) {
        self.landed.insert(ticket, landed);
        while self.landed.len() > RECEIPTS_KEPT {
            self.landed.pop_first();
        }
    }
}

impl Outbox {
    /// The next act to send — **blocking**, because a poster with nothing to
    /// post has nothing else to do. `None` is the window having gone away,
    /// which is the whole of what ends the poster's thread: no stop flag, no
    /// unpark and no join, the channel's own lifetime being the thread's.
    pub fn next(&self) -> Option<(Ticket, Value)> {
        self.acts.recv().ok()
    }

    /// The next act if there is one — the same take without the wait, for a
    /// caller that is draining rather than serving: the acceptance world's own
    /// answerer, which has no thread to park and must return when the queue is
    /// empty.
    pub fn try_next(&self) -> Option<(Ticket, Value)> {
        self.acts.try_recv().ok()
    }

    /// Publish one receipt. A send that fails is a window that has gone away.
    pub fn publish(&self, ticket: Ticket, landed: Landed) -> bool {
        self.receipts.send((ticket, landed)).is_ok()
    }
}

#[cfg(test)]
mod tests;
