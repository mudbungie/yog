//! **Where in-flight invocations live** (REMOTE §5, bl-024b): the map behind
//! [`Mailbox`](super::Mailbox), split from the vocabulary it carries at §12's
//! per-file budget — [`super`] says what a routed invocation *is*, and this
//! says where one waits.
//!
//! **The lock lives in [`state`](crate::state)** (AGENTS.md rule 7), beside
//! presence's: [`MailCell`] and [`lock_mail`] there.

use std::collections::BTreeMap;
use std::time::Duration;

use crate::state::{MailCell, lock_mail};

use super::doubt::unfinished;
use super::{Call, Invocation};

/// The two ends' exchange over one slot: the foot's post, the asker's poll.
mod exchange;
/// A machine that is working, and what it last answered (REMOTE §5.7).
mod known;
/// The follow-class read, its one-reader claim and the bound on its lease.
pub(crate) mod read;
/// One run's text as it arrives (REMOTE §5.7): appended, read, bounded.
mod run;

use run::{LISTEN_SECONDS, Run};

use read::{HOLD_TICK, HOLD_WAITS};

/// How long a slot nobody polls survives before the next post sweeps it: an
/// hour. A driver that died mid-invocation collects nothing, and a map that
/// grew by one entry per such death would be the only unbounded thing in the
/// server.
const TTL_SECONDS: i64 = 3600;

/// One invocation's whole life: queued for a host, taken by one, or answered.
#[derive(Debug, Clone, PartialEq)]
struct Slot {
    /// Who it is addressed to — the only identity that may answer it.
    client: String,
    /// Who asked — the only identity that may read the answer. Two fields
    /// rather than one because the two are never the same caller, and a
    /// mismatch at either end earns the **absent** sentence rather than a
    /// forbidden one (REMOTE §4: a refusal that confirms existence is a
    /// disclosure).
    by: String,
    invocation: Invocation,
    /// **How many follow-class reads this slot has been handed to** — the one
    /// stored fact about the lease, and `taken` derived from it rather than
    /// stored beside it (REMOTE §5.6). "Handed to the read that is running
    /// now" is what the old flag meant, and that is a question about *this*
    /// call rather than about the slot: [`Mailbox::take`] redelivers on its
    /// first look and offers only fresh work on the rest, so a count and a
    /// per-look predicate say everything two fields did, and cannot disagree.
    handed: u32,
    /// What has arrived, and whether it is over (REMOTE §5.7).
    run: Run,
    /// When its asker last asked — **a run lives while its asker listens**
    /// (REMOTE §5.7). The post that queued it counts, and so does every poll;
    /// it is also what the hour sweep measures, since a slot somebody is
    /// still polling is not an abandoned one.
    heard: i64,
}

impl Slot {
    /// End this slot's run in doubt, naming why and how much had arrived.
    fn doubt(&mut self, why: &str) {
        let said = unfinished(&self.client, self.handed, why, self.run.arrived());
        self.run.doubt(&said);
    }

    /// **Is a machine running this, as far as this server believes** (REMOTE
    /// §5.7, bl-3b4b): handed out, not over, its asker still listening and its
    /// run not gone silent. No clock of its own — each clause is a rule that
    /// already ends a run, so *busy* is believed exactly as long as the job is.
    fn works(&self, now: i64) -> bool {
        self.handed > 0
            && !self.run.over()
            && !self.run.silent(now)
            && now - self.heard <= LISTEN_SECONDS
    }
}

/// The map behind the handle: every live invocation by its id, the serial the
/// next id is minted from, and which identities are parked on a follow-class
/// read right now.
#[derive(Default)]
pub(crate) struct Slots {
    live: BTreeMap<String, Slot>,
    seq: u64,
    /// **One reader per client identity** (REMOTE §5.1, bl-1462), as the
    /// token of the read that holds it. Not a refcount, unlike presence: two
    /// connections holding one machine's *queue* is the pathology itself,
    /// where two holding one machine's *presence* is an operator with two
    /// seats.
    reading: BTreeMap<String, u64>,
    /// The serial a read's token is minted from.
    reads: u64,
    /// **What each machine last answered a read** (REMOTE §5.7, bl-3b4b), by
    /// machine, tool and input, with when: the server's one replica of an
    /// answer whose home is that machine. Swept with the slots, at the hour.
    known: BTreeMap<known::Key, (i64, super::Capture)>,
}

/// The process's invocation mailbox, shared by handle exactly as
/// [`Presence`](super::presence::Presence) is. A default one is the posture of
/// a box with no wire: nothing queued and nobody to drain it, which is the
/// general path with no input rather than a case of its own.
#[derive(Clone)]
pub struct Mailbox {
    cell: MailCell,
    waits: u32,
    tick: Duration,
}

impl Default for Mailbox {
    fn default() -> Self {
        Self {
            cell: MailCell::default(),
            waits: HOLD_WAITS,
            tick: HOLD_TICK,
        }
    }
}

impl Mailbox {
    /// A mailbox whose follow-class read holds for `waits` looks `tick` apart —
    /// the production bound is [`Default`], and a test names a short one rather
    /// than sleeping for real.
    pub fn holding(waits: u32, tick: Duration) -> Self {
        Self {
            cell: MailCell::default(),
            waits,
            tick,
        }
    }

    /// Queue `call` for its client on `by`'s behalf and answer the handle it is
    /// known by. `now` is the caller's wall clock (§4.2 unix seconds), which is
    /// also when the sweep of everything older than an hour happens — one pass,
    /// at the one moment the map can grow.
    pub fn post(&self, now: i64, by: &str, call: &Call) -> String {
        let mut slots = lock_mail(&self.cell);
        slots.live.retain(|_, slot| now - slot.heard <= TTL_SECONDS);
        slots.known.retain(|_, (at, _)| now - *at <= TTL_SECONDS);
        slots.seq += 1;
        let id = format!("inv-{}", slots.seq);
        slots.live.insert(
            id.clone(),
            Slot {
                client: call.client.clone(),
                by: by.to_owned(),
                invocation: Invocation {
                    id: id.clone(),
                    tool: call.tool.clone(),
                    input: call.input.clone(),
                    cwd: call.cwd.clone(),
                },
                handed: 0,
                run: Run::default(),
                heard: now,
            },
        );
        id
    }
}

#[cfg(test)]
mod tests;
