//! **A machine that is working, and what it last answered** (REMOTE §5.7,
//! bl-3b4b): the two readings that keep a busy machine in sight.
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
//! - [`Mailbox::known`] is **the one replica** of an answer whose home is the
//!   far machine: what it last said to the same read. Every fresh answer
//!   replaces it, and it is served **only while that machine is busy** — a
//!   machine that can be asked is asked, and one that is gone is not spoken
//!   for.

use super::super::{Call, Capture};
use super::{Mailbox, Slots, lock_mail};

/// One read of one machine: the machine, the tool, and the input as text.
pub(super) type Key = (String, String, String);

fn key(call: &Call) -> Key {
    (
        call.client.clone(),
        call.tool.clone(),
        call.input.to_string(),
    )
}

impl Slots {
    fn busy(&self, now: i64, client: &str) -> bool {
        self.live
            .values()
            .any(|slot| slot.client == client && slot.works(now))
    }
}

impl Mailbox {
    /// Is `client` running something this server handed it and still believes
    /// in? The other half of *is it there*, beside
    /// [`serving`](Self::serving): a machine is parked or it is working, and
    /// one that is neither is asleep or gone.
    pub fn busy(&self, now: i64, client: &str) -> bool {
        lock_mail(&self.cell).busy(now, client)
    }

    /// `call`'s machine answered it `capture` at `now`: the replica, replaced.
    pub fn remember(&self, now: i64, call: &Call, capture: &Capture) {
        lock_mail(&self.cell)
            .known
            .insert(key(call), (now, capture.clone()));
    }

    /// What `call`'s machine last answered this same read — **while it is
    /// busy**, and nothing otherwise.
    pub fn known(&self, now: i64, call: &Call) -> Option<Capture> {
        let slots = lock_mail(&self.cell);
        slots
            .known
            .get(&key(call))
            .filter(|_| slots.busy(now, &call.client))
            .map(|(_, capture)| capture.clone())
    }
}
