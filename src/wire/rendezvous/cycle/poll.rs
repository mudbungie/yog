//! The fifteen-second half of the loop (REMOTE §13.3): the inbox read, a new
//! call handed to [`Answer`](super::super::call::Answer), and the one line a
//! poll earns. Split off `cycle` at §12's budget (bl-5276) on the seam the two
//! deadlines already draw, beside [`publish`](super::publish).

use super::super::item::{Call, Unopened};
use super::super::say;
use super::Cycle;
use std::sync::atomic::Ordering;

impl Cycle {
    /// Read the inbox; a call that verifies, unseals and is new is punched
    /// on its own thread, and every stream that lands is served. Answers the
    /// line the outcome earns — none for an empty inbox.
    pub(super) fn poll(&mut self) -> Result<Option<String>, String> {
        let (key, salt) = (self.inbox_key, self.pairing.inbox_salt());
        let Some(item) = self.dht()?.get(key, salt)? else {
            return Ok(None);
        };
        let call = match Call::open(&self.pairing.seal_key(), &item.value) {
            Ok(call) => call,
            Err(Unopened::Unverified) => return Ok(Some(say::unverified(item.seq))),
            Err(Unopened::NotACall) => return Ok(Some(say::unopened(item.seq))),
        };
        if self.last_nonce == Some(call.nonce) {
            let fresh = self.seen_said.replace(call.nonce) != Some(call.nonce);
            return Ok(fresh.then(|| say::seen(call.nonce)));
        }
        self.last_nonce = Some(call.nonce);
        self.stats.calls.fetch_add(1, Ordering::Relaxed);
        // Said before the punch starts, so its own lines follow this one;
        // the loop's `tell` of the same line is then the no-op it looks like.
        let line = Some(say::opened(call.nonce, &call.endpoints));
        self.tell(line.clone());
        self.answer.clone().spawn(call);
        Ok(line)
    }

    /// Say a poll's line, unless it is the last poll's.
    pub(super) fn tell(&mut self, line: Option<String>) {
        if line != self.last_said
            && let Some(said) = &line
        {
            (self.say)(said);
        }
        self.last_said = line;
    }
}
