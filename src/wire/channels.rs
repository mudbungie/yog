//! **The window's channel set, and the union over it** (REMOTE §8.2, bl-aaec):
//! one [`Channel`] for the engine in this process plus one per
//! [entry](super::entries), the roster composed across them, and the name
//! resolution that refuses a collision.
//!
//! §8.2, verbatim: *"The roster is the union: a workspace is a workspace, and
//! which engine hosts it is a fact painted on it, never a mode the window is
//! in. Names resolve over the union — local leaves and entry leaves in one
//! namespace — and a collision refuses naming the token, §8's
//! two-roots-one-leaf rule with the same remedy shape (rename the entry)."*
//!
//! **Which channel a question goes down is [`seat::channel`](super::seat)'s
//! rule, read from the frame instead of from argv.** An entry is the answer to
//! its leaf; every other name — and every question naming no workspace — goes
//! where it always went, the window's own engine. That is the whole of the
//! routing, and it is why **zero entries is byte for byte today**: with nothing
//! claiming a name, nothing is resolved, nothing extra is asked, and the union
//! is one channel's slice handed straight back.
//!
//! **The collision is the union's own fact, so it is read off the union.** An
//! entry's leaf is unique on disk, so the only token two channels can both hold
//! is one an entry claims and another channel's roster already names. That
//! question costs a look at the composed roster, and it is asked only when an
//! entry claims the name — the general path with empty inputs, not a case.
//!
//! **What is NOT here.** No channel is *dialled* from this file. [`compose`]
//! mints each entry channel's link **pair** and hands the far end back to the
//! engine, which puts one [`asker`](super::asker) on each — its own thread, its
//! own seat, its own material — so a routed question is answered by the engine
//! that hosts it and an entry nobody can reach costs only its own slice. The
//! seats those threads dial are [`Dial`](super::dial::Dial), this set seen from
//! the other end of the same links.

/// **Composing the set from a world** (bl-670c) — the entries read, a link pair
/// minted per channel, and the far ends handed back for the engine's askers.
/// Split from the set itself at §12's band: this is the one arm that touches
/// the disk, and everything below it is a union over values.
mod compose;
pub use compose::{EntryEnd, compose};

use super::channel::{Channel, RosterRow};
use super::link::{Landed, Link};
use crate::boundary::codec;
use serde_json::Value;

/// Every channel this window holds: its own engine's, and one per entry.
///
/// Two fields rather than one list because the two are not interchangeable —
/// the local channel is the one wire the window cannot exist without (bl-dc14)
/// and the fall-through for every unclaimed name, while an entry channel is one
/// participation among N that may come and go. Making that structural means no
/// arm ever has to ask which member of a list it is holding.
pub struct Channels {
    local: Channel,
    entries: Vec<Channel>,
}

/// **A window that holds nothing but its own engine**, on a link nobody
/// answers — the model's posture from boot until the engine hands the real one
/// over, and the same code path as an answer that has not arrived.
impl Default for Channels {
    fn default() -> Self {
        Self::of(Link::default())
    }
}

impl Channels {
    /// The local channel alone, over `local`. The zero-entry shape, named so a
    /// caller that has no world to read entries from (a fixture, the model's
    /// own boot) states that rather than composing an empty one.
    pub fn of(local: Link) -> Self {
        Self::held(local, Vec::new())
    }

    /// The set from channels already built — what the two constructors above
    /// fold into, and the seam a second slice is composed at (§8.2's slices are
    /// values; no second engine is needed to hold one).
    pub fn held(local: Link, entries: Vec<Channel>) -> Self {
        Self {
            local: Channel::local(local),
            entries,
        }
    }

    /// Declare `question` standing on the channel that answers it, and read
    /// whatever landed. A collision refuses in place of an answer, which is the
    /// [`Landed`] `Err` every read surface already paints (`shell::wire`).
    pub fn ask(&mut self, question: &Value) -> Option<Landed> {
        let claimed = match self.route(question) {
            Ok(claimed) => claimed,
            Err(said) => return Some(Err(said)),
        };
        let channel = self
            .entries
            .iter_mut()
            .find(|c| claimed.as_ref().is_some_and(|name| c.claims(name)));
        channel.unwrap_or(&mut self.local).ask(question)
    }

    /// **The union roster** (§8.2): every channel's slice, each row labelled
    /// with the channel it came from. Local first, then the entries in leaf
    /// order — one namespace, one listing, and no mode.
    pub fn roster(&mut self) -> Vec<RosterRow> {
        let mut rows = self.local.rows();
        for channel in &mut self.entries {
            rows.extend(channel.rows());
        }
        rows
    }

    /// Every channel's frame duty (§7.2): take what landed, re-declare what is
    /// standing. Two channel drains per channel and no lock — nothing here can
    /// wait on a socket.
    pub fn settle(&mut self) {
        self.local.settle();
        for channel in &mut self.entries {
            channel.settle();
        }
    }

    /// Whether anything standing is still unanswered — **the local channel's
    /// question**, which is a driven frame's settle condition (bl-44e9).
    ///
    /// Deliberately not the union's. An entry channel's answer comes from
    /// another box, so a driver that settled on it would wait on somebody
    /// else's network to decide when a *local* frame is finished — and a driven
    /// world composes no entries at all, so widening this would describe
    /// nothing it does not already describe.
    #[cfg(test)]
    pub fn awaiting(&self) -> bool {
        self.local.awaiting()
    }

    /// Which entry claims the workspace `question` names, `None` for the local
    /// channel, or the refusal when the union holds that token twice.
    fn route(&mut self, question: &Value) -> Result<Option<String>, String> {
        let Some(named) = codec::decode(question).ok().and_then(|g| g.workspace()) else {
            return Ok(None);
        };
        if !self.entries.iter().any(|c| c.claims(&named)) {
            return Ok(None);
        }
        let held = self.holders(&named);
        if held.len() > 1 {
            return Err(collision(&named, &held));
        }
        Ok(Some(named))
    }

    /// Which channels hold `name` in the union roster — the resolution's whole
    /// evidence, asked of the composition rather than of a table beside it.
    fn holders(&mut self, name: &str) -> Vec<String> {
        self.roster()
            .into_iter()
            .filter(|r| r.row.workspace == name)
            .map(|r| r.origin.held_by())
            .collect()
    }
}

/// **One token, one workspace** — §8's two-roots-one-leaf refusal with §8.2's
/// remedy shape. It opens in the resolver's own words (`naming::by_leaf`:
/// *"ambiguous workspace …"*) because it is the same fact one namespace up, and
/// it names the remedy the ruling names: a client-side rename, which is `mv`,
/// never a rewrite on somebody else's host.
fn collision(name: &str, held: &[String]) -> String {
    format!(
        "ambiguous workspace {name:?}: {} hold that name and the union is one \
         namespace — rename the entry (`mv` its directory under `{}/`), never \
         the workspace on its host",
        held.join(" and "),
        super::entries::ENTRIES,
    )
}

#[cfg(test)]
mod tests;
