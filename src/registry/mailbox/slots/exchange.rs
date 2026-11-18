//! **The exchange over one slot** (REMOTE §5.3, §5.7; bl-d414): the foot's
//! post and the asker's poll — the two acts that meet at a run's text.
//!
//! Split from the map at §12's per-file budget, on the seam the streamed form
//! drew: [`super`] is a slot's birth and the sweep, [`read`](super::read) is
//! the hand-off, and this is everything said *about a run in progress* — who
//! may append to it, who may read it, and the two silences that end it.

use super::super::doubt::{UNHEARD, WENT_QUIET, outside, redelivered, unheard, unknown};
use super::super::{Offsets, Post, Standing};
use super::run::{LISTEN_SECONDS, Run};
use super::{Mailbox, lock_mail};

impl Mailbox {
    /// Post for one invocation **as the client it was addressed to**, and
    /// hand back the slot as it stands after the write — the
    /// [`Marks`](crate::boundary::reply::Reply::Marks) discipline: a receipt is
    /// a re-read, never an echo. A handle this server does not hold, and a
    /// handle addressed to somebody else, earn the same sentence.
    ///
    /// **Posts absorb in order, and the one carrying a capture ends the run**
    /// (REMOTE §5.7). Three refusals, each in band:
    ///
    /// - a run already over **that a machine had posted for** is a spent
    ///   handle. One the server ended itself with nothing posted is reopened —
    ///   a real capture beats a doubt (§5.6);
    /// - **a `partial` for a slot nobody polled in thirty seconds is refused,
    ///   and the refusal is the stop**: the slot is ended in doubt so it is
    ///   never handed out again, and the foot ends its child. A post carrying
    ///   the capture is exempt — there is no child left to stop, and a
    ///   finished answer is worth more to a late asker than a doubt.
    ///
    /// **A capture for a slot handed out more than once is marked here**
    /// (REMOTE §5.6, bl-0655): the count is the slot's and the first post is
    /// the one place real output meets it, so the mark is stored rather than
    /// added by a reader ([`redelivered`]).
    pub fn complete(
        &self,
        now: i64,
        client: &str,
        invocation: &str,
        post: &Post,
    ) -> Result<Standing, String> {
        let mut slots = lock_mail(&self.cell);
        let slot = slots
            .live
            .get_mut(invocation)
            .filter(|slot| slot.client == client)
            .ok_or_else(|| unknown(invocation))?;
        if slot.run.over() {
            if slot.run.acknowledged() {
                return Err(unknown(invocation));
            }
            slot.run = Run::default();
        }
        if matches!(post, Post::Partial(_)) && now - slot.heard > LISTEN_SECONDS {
            slot.doubt(UNHEARD);
            return Err(unheard(invocation));
        }
        slot.run.absorb(now, redelivered(client, slot.handed), post);
        Ok(slot.run.receipt())
    }

    /// The asker's poll, which **stamps the slot** — a run lives while its
    /// asker listens. Once the run is over the capture is answered and the
    /// slot released in the same breath, because a capture is read once and
    /// the map is the only thing that would keep it. A handle `by` did not
    /// post is absent, exactly as an unheld one is.
    ///
    /// `since` is the asker's own byte counts (REMOTE §5.7): with it the
    /// answer is the text from there on and the server drops what was passed;
    /// without it the read is what it always was. **A run that posted and then
    /// said nothing for thirty seconds is ended in doubt here**, lazily — the
    /// poll is the only clock this leg has.
    pub fn collect(
        &self,
        now: i64,
        by: &str,
        invocation: &str,
        since: Option<Offsets>,
    ) -> Result<Standing, String> {
        let mut slots = lock_mail(&self.cell);
        let slot = slots
            .live
            .get_mut(invocation)
            .filter(|slot| slot.by == by)
            .ok_or_else(|| unknown(invocation))?;
        slot.heard = now;
        if slot.run.silent(now) {
            slot.doubt(WENT_QUIET);
        }
        let standing = slot.run.answer(since).ok_or_else(|| outside(invocation))?;
        if standing.capture.is_some() {
            slots.live.remove(invocation);
        }
        Ok(standing)
    }
}
