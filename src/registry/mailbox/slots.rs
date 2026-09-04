//! **Where in-flight invocations live** (REMOTE §5, bl-024b): the map behind
//! [`Mailbox`](super::Mailbox), split from the vocabulary it carries at §12's
//! per-file budget — [`super`] says what a routed invocation *is*, and this
//! says where one waits.
//!
//! **The lock is here, and it is the fourth sanctioned carve-out** from
//! AGENTS.md rule 7 (`rules/locks-outside-state.yml`). It is presence's
//! carve-out for presence's measured reason: adding a second `Arc<Mutex<…>>`
//! alias to `state.rs` cost that file its 100 % coverage floor once already —
//! llvm-cov attributes phantom uncovered regions to its declaration lines when
//! a new monomorphization lands there, and moving the addition to the end of
//! the file did not dissolve it. This map is the presence map's sibling in
//! every other way — same lifetime, same rate of change, same client key — so
//! it sits beside it rather than in the chokepoint, and the confinement's
//! *auditability* is bought back by naming the file in the rule and saying
//! here what it holds.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Duration;

use super::{Call, Capture, Invocation, unknown};

/// The follow-class read, its one-reader claim and the bound on its lease.
pub(crate) mod read;

use read::{HOLD_TICK, HOLD_WAITS};

/// How long an uncollected slot survives before the next post sweeps it: an
/// hour. A driver that died mid-invocation collects nothing, and a map that
/// grew by one entry per such death would be the only unbounded thing in the
/// engine.
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
    capture: Option<Capture>,
    at: i64,
}

/// The map behind the handle: every live invocation by its id, the serial the
/// next id is minted from, and which identities are parked on a follow-class
/// read right now.
#[derive(Default)]
struct Slots {
    live: BTreeMap<String, Slot>,
    seq: u64,
    /// **One reader per client identity** (REMOTE §5.1, bl-1462). Not a
    /// refcount, unlike presence: two connections holding one machine's *queue*
    /// is the pathology itself, where two holding one machine's *presence* is
    /// an operator with two seats.
    reading: BTreeSet<String>,
}

type MailCell = Arc<Mutex<Slots>>;

/// Lock it, poison-immune. Kept on one line for `state.rs`'s own reason — a
/// split isolates the never-taken recovery, which reads as uncovered under
/// `ignore-panics`.
fn lock_mail(cell: &MailCell) -> MutexGuard<'_, Slots> {
    cell.lock().unwrap_or_else(PoisonError::into_inner)
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
        slots.live.retain(|_, slot| now - slot.at <= TTL_SECONDS);
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
                capture: None,
                at: now,
            },
        );
        id
    }

    /// Answer one invocation **as the client it was addressed to**, and hand
    /// back what is stored after the write — the
    /// [`Marks`](crate::boundary::reply::Reply::Marks) discipline: a receipt is
    /// a re-read, never an echo. A handle this engine does not hold, and a
    /// handle addressed to somebody else, earn the same sentence.
    pub fn complete(
        &self,
        client: &str,
        invocation: &str,
        capture: &Capture,
    ) -> Result<Capture, String> {
        let mut slots = lock_mail(&self.cell);
        let slot = slots
            .live
            .get_mut(invocation)
            .filter(|slot| slot.client == client)
            .ok_or_else(|| unknown(invocation))?;
        slot.capture = Some(capture.clone());
        Ok(capture.clone())
    }

    /// The asker's poll: `Some` once the host has answered — and the slot is
    /// released in the same breath, because a capture is read once and the map
    /// is the only thing that would keep it. `None` is *still running*; a
    /// handle `by` did not post is absent, exactly as an unheld one is.
    pub fn collect(&self, by: &str, invocation: &str) -> Result<Option<Capture>, String> {
        let mut slots = lock_mail(&self.cell);
        let slot = slots
            .live
            .get(invocation)
            .filter(|slot| slot.by == by)
            .ok_or_else(|| unknown(invocation))?;
        let Some(capture) = slot.capture.clone() else {
            return Ok(None);
        };
        slots.live.remove(invocation);
        Ok(Some(capture))
    }
}

#[cfg(test)]
mod tests;
