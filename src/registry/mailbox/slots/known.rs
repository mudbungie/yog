//! **A machine that is working, and the reads put to it** (REMOTE §5.7,
//! bl-3b4b, bl-d9ad): what keeps a busy machine in sight, and what makes
//! *busy* mean *queue*.
//!
//! A foot holds a read only while it waits (REMOTE §5.3), so *parked on its
//! read* is false for the whole of every job — and a predicate that asks only
//! that hides the one machine that is certainly there. The mailbox already
//! holds the other half: the job it handed that machine.
//!
//! - [`Mailbox::busy`] is **derived, never stored**: some slot handed to the
//!   machine is still believed to be running. It ends by the rules that end a
//!   run — the capture, a doubt, thirty seconds without a post, thirty seconds
//!   without its asker — so a machine that dies mid-job is believed busy for
//!   as long as its job is believed alive, and no longer.
//! - **A read is asked, never refused** ([`Mailbox::ask`]). It is deposited
//!   like any job and waits behind whatever its machine is running; the foot
//!   is handed it at its next read. **One read is in flight at most once** —
//!   the ask in flight is the read's own field, so a second asker joins it
//!   rather than queueing a twin.
//! - [`Mailbox::known`] is **the one replica** of an answer whose home is the
//!   far machine: what it last said to the same read. The flight's capture
//!   lands here whoever is or is not waiting, so an asker's wait is bounded
//!   while the ask stays queued, and whoever asks next is answered.

use super::super::doubt::WENT_QUIET;
use super::super::{Call, Capture};
use super::{Mailbox, Slots, lock_mail};

/// One read of one machine: the machine, the tool, and the input as text.
pub(super) type Key = (String, String, String);

/// Who a read's ask is posted by: the server itself, a name no certificate
/// and no in-world caller bears, so no seat's `capture` can collect it.
const OWN: &str = "";

/// One read put to one machine.
#[derive(Default)]
pub(super) struct Asked {
    /// When it was last asked — what the hour sweep measures.
    pub(super) at: i64,
    /// The ask in flight, by its invocation.
    flight: Option<String>,
    /// What the machine last answered.
    last: Option<Capture>,
}

fn key(call: &Call) -> Key {
    (
        call.client.clone(),
        call.tool.clone(),
        call.input.to_string(),
    )
}

impl Slots {
    /// A pushing host (REMOTE §5.8) is never busy: it takes work while it
    /// works, so what it is running stands in front of nothing.
    fn busy(&self, now: i64, client: &str) -> bool {
        !self.reading.get(client).is_some_and(|claim| claim.push)
            && self
                .live
                .values()
                .any(|slot| slot.client == client && slot.works(now))
    }

    /// **Look at `key`'s flight**: listened to, ended in doubt if its machine
    /// went quiet, and — once over — folded into what is known. A flight the
    /// sweep took is no flight. Answers whether one is still out.
    fn settle(&mut self, now: i64, key: &Key) -> bool {
        let read = self.known.entry(key.clone()).or_default();
        read.at = now;
        let Some(id) = read.flight.clone() else {
            return false;
        };
        let landed = self
            .live
            .get_mut(&id)
            .and_then(|slot| {
                slot.heard = now;
                if slot.run.silent(now) {
                    slot.doubt(WENT_QUIET);
                }
                slot.run.answer(None)
            })
            .map(|standing| standing.capture);
        if landed == Some(None) {
            return true;
        }
        self.live.remove(&id);
        if let Some(capture) = landed.flatten() {
            read.last = Some(capture);
        }
        read.flight = None;
        false
    }
}

impl Mailbox {
    /// Is `client` running something this server handed it and still believes
    /// in? The other half of *is it there*, beside
    /// [`serving`](Self::serving): a machine is parked or it is working.
    pub fn busy(&self, now: i64, client: &str) -> bool {
        lock_mail(&self.cell).busy(now, client)
    }

    /// **Put `call` to its machine as a read**: queued behind whatever that
    /// machine is running, unless this same read is already in flight.
    pub fn ask(&self, now: i64, call: &Call) {
        let (mut slots, read) = (lock_mail(&self.cell), key(call));
        if !slots.settle(now, &read) {
            let id = slots.queue(now, OWN, call);
            slots.known.entry(read).or_default().flight = Some(id);
        }
    }

    /// Is `call`'s ask still out — queued, or being run?
    pub fn queued(&self, now: i64, call: &Call) -> bool {
        lock_mail(&self.cell).settle(now, &key(call))
    }

    /// What `call`'s machine last answered this same read.
    pub fn known(&self, call: &Call) -> Option<Capture> {
        let slots = lock_mail(&self.cell);
        slots
            .known
            .get(&key(call))
            .and_then(|read| read.last.clone())
    }
}
