//! **The pushing read** (REMOTE §5.8, bl-f5cd): a host that states what it
//! holds is handed each job the moment it is queued, over one read that stays
//! open, and is never asked to finish before it is handed more.
//!
//! It is [`take`](Mailbox::take)'s own look with two things changed, and both
//! are the host's statement doing what the server used to infer from silence:
//!
//! - **What the read leaves alone is a set, not a guess.** It starts as the
//!   host's `holding` and grows by every slot this read hands out, so a later
//!   look sees only what the host has never been handed. A serial read could
//!   only say *I asked again, so I hold nothing*; a pushing read says which.
//! - **It does not end when it has work.** Each job is one frame, written as it
//!   is found; a quiet read writes an empty frame on the hold's own bound, the
//!   heartbeat, so a dead connection is found by a write rather than never.
//!   It ends only when a newer activation takes the queue ([`Taken::Superseded`]),
//!   or when its connection does.

use std::collections::{BTreeSet, VecDeque};

use super::super::doubt::NOT_HELD;
use super::read::{Reading, Taken};
use super::run::LISTEN_SECONDS;
use super::{Invocation, Mailbox, lock_mail};

/// The server's wall clock in unix seconds, read at every look: the one input
/// a held read takes that moves while it is held.
pub type Now = Box<dyn Fn() -> i64 + Send>;

/// One pushing read, as the frames it answers: one job each, an empty
/// heartbeat when it has been quiet for a hold, and `Superseded` last.
pub struct Push {
    mail: Mailbox,
    reading: Reading,
    /// Every id this host holds, as far as this read knows: what it said it
    /// holds, and everything handed to it since.
    held: BTreeSet<String>,
    /// Found at the last look and not yet written — one frame per job.
    ready: VecDeque<Invocation>,
    now: Now,
    quiet: u32,
    ended: bool,
}

impl Mailbox {
    /// **Open a pushing read for `client`** (REMOTE §5.8), holding `holding`.
    /// It takes the reader slot from any read that holds it, and its first
    /// look settles the lease: a held slot is left alone, an unheld one that
    /// nothing was posted for goes out again under the hand-off bound, and an
    /// unheld one that was posted for is answered in doubt.
    pub fn push(&self, client: &str, holding: &[String], now: Now) -> Push {
        let reading = self.claim(&mut lock_mail(&self.cell), client, true);
        Push {
            mail: self.clone(),
            reading,
            held: holding.iter().cloned().collect(),
            ready: VecDeque::new(),
            now,
            quiet: 0,
            ended: false,
        }
    }

    /// One look: every slot of this client that is not over and not held is
    /// met as an acknowledging look meets it — this host holds exactly what
    /// it said, so anything else it was handed it no longer has — and each
    /// one handed joins `held`. `None` when the claim is no longer this read's.
    fn look(&self, push: &mut Push) -> Option<()> {
        let now = (push.now)();
        let mut slots = lock_mail(&self.cell);
        if !push.reading.holds(&slots) {
            return None;
        }
        for (id, slot) in &mut slots.live {
            if slot.client != push.reading.name() || slot.run.over() || push.held.contains(id) {
                continue;
            }
            let listening = now - slot.heard <= LISTEN_SECONDS;
            if let Some(job) = slot.met(true, listening, NOT_HELD) {
                push.held.insert(id.clone());
                push.ready.push_back(job);
            }
        }
        Some(())
    }
}

impl Iterator for Push {
    type Item = Taken;

    /// The next frame: a job as soon as one is found, an empty one after a
    /// quiet hold, or the supersession that ends the read. Parks between
    /// looks on the mailbox's own tick.
    fn next(&mut self) -> Option<Taken> {
        if self.ended {
            return None;
        }
        let mail = self.mail.clone();
        loop {
            if let Some(job) = self.ready.pop_front() {
                self.quiet = 0;
                return Some(Taken::Work(vec![job]));
            }
            if mail.look(self).is_none() {
                self.ended = true;
                return Some(Taken::Superseded);
            }
            if self.ready.is_empty() {
                if self.quiet >= mail.waits {
                    self.quiet = 0;
                    return Some(Taken::Work(Vec::new()));
                }
                self.quiet += 1;
                std::thread::sleep(mail.tick);
            }
        }
    }
}
