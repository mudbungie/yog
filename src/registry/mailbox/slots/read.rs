//! **The follow-class read, and the lease it hands out** — REMOTE §3, the
//! routing leg of REMOTE §5.3, and the bound REMOTE §5.6 put on it. [`super`]
//! is where a slot waits: the map, the post, the completion and the collect.
//! This is the one act that takes work *out* of it, the claim that says only
//! one connection per machine may — and how an advertisement ends that claim
//! (REMOTE §5.1, bl-96e0) — and the count that says how many times one slot
//! may go.
//!
//! Split from the map at §12's per-file budget, on the seam the two halves
//! already had: everything here is about a *read in progress* — it parks, it
//! holds a claim for exactly its own duration, and it decides what a client is
//! handed — where everything there is about a slot's own life.

use std::time::Duration;

use super::super::doubt::{ASKED_AGAIN, in_doubt};
use super::{Invocation, MailCell, Mailbox, Slot, Slots, lock_mail};

/// How long a follow-class read waits for work before answering with none:
/// 240 looks, 125 ms apart — thirty seconds. A bound rather than an open wait,
/// because the read holds a connection thread and a peer that went away must
/// not hold one forever.
pub(super) const HOLD_WAITS: u32 = 240;
pub(super) const HOLD_TICK: Duration = Duration::from_millis(125);

/// **How many follow-class reads one invocation may be handed to**
/// (REMOTE §5.6, ruling 2): three. The first is the delivery; the second is the
/// redelivery bl-e658 bought, which is the at-least-once leg and is kept; the
/// third separates a blip from a poison invocation. A fourth would be a loop
/// only the hour sweep ends, so at the read that would hand it again the
/// server writes the slot's capture itself ([`in_doubt`]).
pub(super) const HAND_OFFS: u32 = 3;

/// **How a follow-class read ended**: with what it was handed, or told that it
/// is no longer this machine's reader (REMOTE §5.1, bl-96e0).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Taken {
    /// What was queued, which is ordinarily nothing: the hold expired.
    Work(Vec<Invocation>),
    /// **A newer activation of this certificate advertised while this read was
    /// parked.** It is an answer and never a silence, because the read it ends
    /// may belong to a process that is still alive, and a machine that lost its
    /// queue without being told is the failure the one-reader rule was for.
    Superseded,
}

impl Taken {
    /// What this read was handed — nothing, when it was superseded: a read
    /// that lost the queue is handed no more of it.
    pub fn work(self) -> Vec<Invocation> {
        match self {
            Self::Work(work) => work,
            Self::Superseded => Vec::new(),
        }
    }
}

impl Mailbox {
    /// **The follow-class read** (REMOTE §3): everything queued for `client`,
    /// waiting up to this mailbox's hold for the first of it. It answers the
    /// empty set when the hold expires, which is not a failure — the host asks
    /// again, and an answer that never came would be the hang the deadline
    /// exists to exclude.
    ///
    /// Two things happen before the wait, and REMOTE §5.3 is the authority on
    /// both: the read **claims this client's one reader slot**, refusing a
    /// second connection that is already holding it (bl-1462), and its first
    /// look **acknowledges the previous read** (bl-e658) — every slot this
    /// client holds with no capture goes back out, under the id it was first
    /// handed. Every later look in the same hold offers only work posted
    /// since, which is what keeps one read from handing one slot twice.
    ///
    /// **Every look asks first whether the claim is still this read's**
    /// (REMOTE §5.1, bl-96e0), under the lock the hand-off itself takes: an
    /// advertisement from the same certificate ends the claim
    /// ([`supersede`](Self::supersede)), and a read that has lost it is handed
    /// nothing more and answers [`Taken::Superseded`] within one look.
    ///
    /// **The claim's life is this call, not the connection's** (REMOTE §5.1,
    /// bl-0a74) — it is dropped on the way out, before the caller has written a
    /// byte of the answer. A peer that vanished without a FIN leaves a thread
    /// asleep in this loop, and its slot comes free within one hold's width
    /// rather than whenever some later socket act notices. Handing the claim
    /// back to the caller would read as a tidier lifetime and would silently
    /// make the first network blip permanent for a client that only reads.
    pub fn take(&self, client: &str) -> Result<Taken, String> {
        let reading = self.reading(client)?;
        let mut ack = true;
        for _ in 0..self.waits {
            match self.drain(&reading, ack) {
                None => return Ok(Taken::Superseded),
                Some(taken) if !taken.is_empty() => return Ok(Taken::Work(taken)),
                Some(_) => {}
            }
            ack = false;
            std::thread::sleep(self.tick);
        }
        Ok(self
            .drain(&reading, ack)
            .map_or(Taken::Superseded, Taken::Work))
    }

    /// **One reader per identity** (REMOTE §5.1, bl-1462): the claim a parked
    /// read holds, released however the read leaves. A second *read* under the
    /// same certificate never takes it — a process that is entitled to the
    /// queue says so by advertising, which is the gesture it makes on taking a
    /// channel up, and a bare read made no such statement.
    pub(crate) fn reading(&self, client: &str) -> Result<Reading, String> {
        let mut slots = lock_mail(&self.cell);
        if slots.reading.contains_key(client) {
            return Err(format!(
                "invocations: {client:?} is already holding this server's follow-class \
                 read — one machine's queue has one reader, and a read never takes it \
                 from the read that holds it: an activation that is entitled to the \
                 queue advertises first, which ends that read and tells it so. \
                 Something else is presenting this certificate: stop it, or stop this"
            ));
        }
        Ok(self.claim(&mut slots, client, false))
    }

    /// The claim itself, under the lock its caller holds. A **pushing** read
    /// (REMOTE §5.8) makes it over whatever read holds it, as an advertisement
    /// does: a host that states what it holds is an activation entitled to its
    /// queue, and the read it ends is answered [`Taken::Superseded`].
    pub(super) fn claim(&self, slots: &mut Slots, client: &str, push: bool) -> Reading {
        slots.reads += 1;
        let token = slots.reads;
        slots
            .reading
            .insert(client.to_owned(), Claim { token, push });
        Reading {
            cell: self.cell.clone(),
            name: client.to_owned(),
            token,
        }
    }

    /// **A newer activation of `client` has spoken** (REMOTE §5.1, bl-96e0):
    /// whatever read its certificate holds stops being the reader, now. The
    /// slot is free before this returns, so the read the new activation is
    /// about to make is granted; the read that lost it learns at its next look
    /// and is answered [`Taken::Superseded`]. With nothing parked it is the
    /// general path with nothing to end.
    pub fn supersede(&self, client: &str) {
        lock_mail(&self.cell).reading.remove(client);
    }

    /// Is `client` parked on a follow-class read right now? A caller that
    /// would hand a machine work and wait reads it: a machine that is not
    /// waiting is not asked.
    pub fn serving(&self, client: &str) -> bool {
        lock_mail(&self.cell).reading.contains_key(client)
    }

    /// One look at this read's queue, handing over what it finds and counting
    /// each hand-off — or `None`, having handed over nothing, when the claim
    /// is no longer this read's. `ack` is the read's first look and carries **the
    /// acknowledgement** (bl-e658, REMOTE §5.3): a slot this client already
    /// holds and this server has no capture for goes out again, because the
    /// hand-off mark is a lease and not a latch — a parked read cannot learn
    /// its peer went away, so treating the drain as the delivery loses
    /// whatever was posted into a dead one. Later looks in the same hold see
    /// only slots nobody has been handed, which is the whole of what the old
    /// `taken` flag said.
    ///
    /// **The lease is bounded** (REMOTE §5.6, ruling 2): a slot already handed
    /// [`HAND_OFFS`] times is not handed a fourth, and the server writes its
    /// capture in doubt instead. From there it is a capture like any other —
    /// never offered again, and collected by the driver's ordinary poll.
    ///
    /// **An acknowledged slot is never handed out again** (REMOTE §5.7, which
    /// narrows that ruling): a slot holding a post was received, so its
    /// machine asking for work again means the run will not finish, and the
    /// server answers it in doubt at once. Redelivery was bought for the
    /// hand-off nobody received; a reader has already consumed part of this
    /// one, and a second run spliced behind it is a capture no run produced.
    fn drain(&self, reading: &Reading, ack: bool) -> Option<Vec<Invocation>> {
        let client = reading.name.as_str();
        let mut slots = lock_mail(&self.cell);
        if !reading.holds(&slots) {
            return None;
        }
        let mine = slots.live.values_mut();
        Some(
            mine.filter(|slot| slot.client == client && !slot.run.over())
                .filter_map(|slot| slot.met(ack, true, ASKED_AGAIN))
                .collect(),
        )
    }
}

impl Slot {
    /// **One slot, met by a look** (REMOTE §5.3, §5.6, §5.7): an acknowledged
    /// run is answered in doubt for `why`, an exhausted lease in doubt for its
    /// count, and otherwise the slot is handed — when the look acknowledges
    /// (`ack`) or nobody has been handed it, and only while its asker is
    /// `listening` (§5.8: an ask nobody waits on is not run for nobody).
    pub(super) fn met(&mut self, ack: bool, listening: bool, why: &str) -> Option<Invocation> {
        if self.run.acknowledged() {
            self.doubt(why);
        } else if self.handed >= HAND_OFFS {
            self.run.doubt(&in_doubt(&self.client, self.handed));
        } else if (ack || self.handed == 0) && listening {
            self.handed += 1;
            return Some(self.invocation.clone());
        }
        None
    }
}

/// Who holds one machine's reader slot: the read's token, and whether it is a
/// pushing read (REMOTE §5.8) — the one fact that says a machine's host takes
/// work while it works, so *busy* is not a thing it can be.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Claim {
    token: u64,
    pub(super) push: bool,
}

/// One parked follow-class read, as a claim on its client's reader slot.
/// Releasing it is [`Drop`] and nothing else — presence's own shape and its
/// reason: a read leaves by answering, by refusing, by its peer vanishing and
/// by a thread panicking, and a leave verb would be forgotten at one of them.
///
/// **It is one read's claim and not its client's**: `token` is what tells this
/// read from the one that follows it under the same name, so a read that was
/// superseded neither keeps reading nor, on its way out, releases the slot its
/// successor now holds.
pub(crate) struct Reading {
    cell: MailCell,
    name: String,
    token: u64,
}

impl Reading {
    /// The client this read is the reader for.
    pub(super) fn name(&self) -> &str {
        &self.name
    }

    pub(super) fn holds(&self, slots: &Slots) -> bool {
        slots.reading.get(&self.name).map(|claim| claim.token) == Some(self.token)
    }
}

impl Drop for Reading {
    fn drop(&mut self) {
        let mut slots = lock_mail(&self.cell);
        if self.holds(&slots) {
            slots.reading.remove(&self.name);
        }
    }
}
