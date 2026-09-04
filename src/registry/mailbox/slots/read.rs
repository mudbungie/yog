//! **The follow-class read, and the lease it hands out** — REMOTE §3, the
//! routing leg of REMOTE §5.3, and the bound REMOTE §5.6 put on it. [`super`]
//! is where a slot waits: the map, the post, the completion and the collect.
//! This is the one act that takes work *out* of it, the claim that says only
//! one connection per machine may, and the count that says how many times one
//! slot may go.
//!
//! Split from the map at §12's per-file budget, on the seam the two halves
//! already had: everything here is about a *read in progress* — it parks, it
//! holds a claim for exactly its own duration, and it decides what a client is
//! handed — where everything there is about a slot's own life.

use std::time::Duration;

use super::super::in_doubt;
use super::{Invocation, MailCell, Mailbox, lock_mail};

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
/// engine writes the slot's capture itself ([`in_doubt`]).
pub(super) const HAND_OFFS: u32 = 3;

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
    /// **The claim's life is this call, not the connection's** (REMOTE §5.1,
    /// bl-0a74) — it is dropped on the way out, before the caller has written a
    /// byte of the answer. That is the contract a redialling foot rests on: a
    /// peer that vanished without a FIN leaves a thread asleep in this loop,
    /// and its slot comes free within one hold's width rather than whenever
    /// some later socket act notices, so the one-reader refusal a redial meets
    /// is a dying predecessor and is **retryable**. Handing the claim back to
    /// the caller would read as a tidier lifetime and would silently make the
    /// first network blip permanent.
    pub fn take(&self, client: &str) -> Result<Vec<Invocation>, String> {
        let _reading = self.reading(client)?;
        let mut ack = true;
        for _ in 0..self.waits {
            let taken = self.drain(client, ack);
            if !taken.is_empty() {
                return Ok(taken);
            }
            ack = false;
            std::thread::sleep(self.tick);
        }
        Ok(self.drain(client, ack))
    }

    /// **One reader per identity** (REMOTE §5.1, bl-1462): the claim a parked
    /// read holds, released however the read leaves. A second connection under
    /// the same certificate is two processes claiming one machine's name, and
    /// it is refused in band rather than silently taking the work the first is
    /// parked for.
    pub(crate) fn reading(&self, client: &str) -> Result<Reading, String> {
        if !lock_mail(&self.cell).reading.insert(client.to_owned()) {
            return Err(format!(
                "invocations: {client:?} is already holding this engine's follow-class \
                 read — one machine's queue has one reader, because a second would take \
                 work the first is parked for and neither end would learn it. Something \
                 else is presenting this certificate: stop it, or stop this"
            ));
        }
        Ok(Reading {
            cell: self.cell.clone(),
            name: client.to_owned(),
        })
    }

    /// Is `client` parked on a follow-class read right now? The advertisement's
    /// own gate reads it (REMOTE §5.1): a set may not be replaced under a
    /// machine that is serving.
    pub fn serving(&self, client: &str) -> bool {
        lock_mail(&self.cell).reading.contains(client)
    }

    /// One look at `client`'s queue, handing over what it finds and counting
    /// each hand-off. `ack` is the read's first look and carries **the
    /// acknowledgement** (bl-e658, REMOTE §5.3): a slot this client already
    /// holds and this engine has no capture for goes out again, because the
    /// hand-off mark is a lease and not a latch — a parked read cannot learn
    /// its peer went away, so treating the drain as the delivery loses
    /// whatever was posted into a dead one. Later looks in the same hold see
    /// only slots nobody has been handed, which is the whole of what the old
    /// `taken` flag said.
    ///
    /// **The lease is bounded** (REMOTE §5.6, ruling 2): a slot already handed
    /// [`HAND_OFFS`] times is not handed a fourth, and the engine writes its
    /// capture in doubt instead. From there it is a capture like any other —
    /// never offered again, and collected by the driver's ordinary poll.
    fn drain(&self, client: &str, ack: bool) -> Vec<Invocation> {
        let mut slots = lock_mail(&self.cell);
        let mut out = Vec::new();
        for slot in slots.live.values_mut() {
            if slot.client != client || slot.capture.is_some() {
                continue;
            }
            if slot.handed >= HAND_OFFS {
                slot.capture = Some(in_doubt(client, slot.handed));
            } else if ack || slot.handed == 0 {
                slot.handed += 1;
                out.push(slot.invocation.clone());
            }
        }
        out
    }
}

/// One parked follow-class read, as a claim on its client's reader slot.
/// Releasing it is [`Drop`] and nothing else — presence's own shape and its
/// reason: a read leaves by answering, by refusing, by its peer vanishing and
/// by a thread panicking, and a leave verb would be forgotten at one of them.
pub(crate) struct Reading {
    cell: MailCell,
    name: String,
}

impl Drop for Reading {
    fn drop(&mut self) {
        lock_mail(&self.cell).reading.remove(&self.name);
    }
}
