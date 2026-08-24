//! **Every channel a window's off-frame thread dials** (REMOTE §8.2, bl-670c):
//! the engine-side twin of [`Channels`](super::channels::Channels), which is the
//! model-side one.
//!
//! The two halves are the same set seen from the two ends of the same links.
//! The model's half holds a slice per channel and decides which one a standing
//! question belongs to; this half holds a **seat** per channel — its own
//! engine's, plus one per [entry](super::entries) on that entry's own material —
//! and is what the three threads with no link of their own dial through: the
//! [`poster`](super::poster) routing an act by the workspace it names, the
//! [`lane`](super::lane) dialling whichever channel hosts the focused
//! conversation, and the [`searcher`](crate::search) fanning out over all of
//! them. The [`asker`](super::asker) needs none of it: it holds one channel's
//! link and one channel's seat, one thread each, which is what makes an
//! unreachable entry cost only its own slice.
//!
//! **Which channel a gesture goes down is [`seat::channel`](super::seat)'s
//! rule, once more.** An entry is the answer to its leaf; every other name —
//! and every gesture naming no workspace — goes where it always went, this
//! window's own engine. That is also why **zero entries is byte for byte
//! today**: with nothing claiming a name there is nothing to resolve, and every
//! call below is the local seat's own.
//!
//! **The union's collision is not asked here, and that is not a gap.** A
//! collision is a fact about the composed *roster* — two rows wearing one token
//! — so it is read off the roster, where §8.2 puts it, and it refuses every
//! read of that name (`Channels::route`). An act, a follow and a search have no
//! roster in hand and need none: §8.2's other sentence answers them outright,
//! *"an entry that exists is the answer to its name even when it cannot be
//! dialled"*, which is the rule `yog seat` has resolved by since bl-4e31. One
//! remedy covers both readings — rename the entry.
//!
//! **A seat that will not open is one channel's sentence**, held here and
//! answered in place of every gesture routed to it, exactly as a half
//! provisioned entry refuses at `yog seat` rather than falling through to
//! another engine. Nothing here is fatal to the set: the local seat is the one
//! this window cannot exist without, and [`Engine::window_wire`] has already
//! refused the whole window (`shell::refusal`, bl-dc14) if that one could not
//! be opened.

use super::channel::Origin;
use super::client::Seat;
use super::entries::Entry;
use super::link::Landed;
use crate::boundary::codec;
use serde_json::Value;

/// One channel as a thread dials it: which channel it is, and its seat — or the
/// sentence saying why this box has none for it.
struct Reach {
    origin: Origin,
    seat: Result<Seat, String>,
}

/// Every channel this window holds, seated.
///
/// Two fields rather than one list for [`Channels`](super::channels::Channels)'
/// reason exactly: the local channel is the fall-through every unclaimed name
/// takes and the one wire the window cannot exist without, while an entry
/// channel is one participation among N that may come and go. Making that
/// structural means no arm asks which member of a list it is holding.
pub struct Dial {
    local: Reach,
    entries: Vec<Reach>,
}

impl Dial {
    /// The local channel alone — the zero-entry shape, named so a caller with
    /// no world to read entries from states that rather than composing an empty
    /// one. [`Channels::of`](super::channels::Channels::of)'s twin.
    pub fn of(local: Seat) -> Self {
        Self::compose(&[], local)
    }

    /// The whole set: this window's own engine over `local`, plus one seat per
    /// entry, in the order `entries` were read (leaf order).
    ///
    /// **Each thread composes its own**, which is why this takes the entries
    /// rather than the seats: the poster, the lane and the searcher already
    /// mint separate seats on loopback because they dial independently and a
    /// seat is a configuration and an address (REMOTE §6) — there is nothing to
    /// share, one channel or twenty.
    pub fn compose(entries: &[Entry], local: Seat) -> Self {
        Self {
            local: Reach {
                origin: Origin::Local,
                seat: Ok(local),
            },
            entries: entries.iter().map(Reach::entry).collect(),
        }
    }

    /// **What one gesture earned, down the channel its name resolves to** — the
    /// poster's whole routing, and the act path's half of REMOTE §7's amended
    /// fact-locality: a foreign workspace's seen and pin gestures land at its
    /// host because the workspace they name is that host's.
    ///
    /// Exactly-once is untouched and structural: routing *picks* a channel, so
    /// one act is one send down one seat, whatever it names.
    pub(crate) fn answered(&self, question: &Value) -> Landed {
        self.route(question).answered(question)
    }

    /// **Ask and stay on the line, at whichever channel hosts the subject**
    /// (§8.2's *"the follow lane dialled at whichever channel hosts the focused
    /// conversation"*). One lane still, because one conversation is focused: the
    /// lane does not fan out, it resolves.
    pub(crate) fn followed(
        &self,
        question: &Value,
        on_frame: &mut dyn FnMut(Landed) -> bool,
    ) -> Result<(), String> {
        self.route(question).followed(question, on_frame)
    }

    /// **Ask every channel** (§8.2's *"the searcher fanned out and unioned"*),
    /// handing each answer over as it lands, local channel first. `on_answer`
    /// answers whether to keep going: `false` ends the fan, which is how a
    /// superseded search stops without a cancel the boundary would have to
    /// carry.
    ///
    /// An **entry's** refusal is named with the entry that gave it, because a
    /// union that says only *"connect: no route to host"* cannot say which
    /// host. The local channel's is not: an unattributed sentence has always
    /// meant this window's own engine, and a box holding no entry must read
    /// byte for byte as it did. The routed calls above attribute nothing at
    /// all — their answer lands on the slice of the channel that was asked,
    /// which already wears its origin.
    pub(crate) fn fanned(&self, question: &Value, on_answer: &mut dyn FnMut(Landed) -> bool) {
        for reach in std::iter::once(&self.local).chain(&self.entries) {
            let landed = reach
                .answered(question)
                .map_err(|said| reach.origin.attributed(&said));
            if !on_answer(landed) {
                return;
            }
        }
    }

    /// Which channel answers `question`. An entry is the answer to its leaf;
    /// everything else is the local channel's, undecodable envelopes included —
    /// a gesture this box cannot read names no entry, and the engine that
    /// refuses it should be the one that would have answered it.
    fn route(&self, question: &Value) -> &Reach {
        let named = codec::decode(question).ok().and_then(|g| g.workspace());
        named
            .and_then(|name| self.entries.iter().find(|reach| reach.claims(&name)))
            .unwrap_or(&self.local)
    }
}

impl Reach {
    /// One entry, seated on its own material. Its `Err` is the entry's own
    /// sentence where the material would not read, and the seat's where the
    /// material read but will not open — one class from here, both being *this
    /// channel cannot be dialled, and here is why*.
    fn entry(held: &Entry) -> Self {
        Self {
            origin: Origin::of(held),
            seat: held.seat(),
        }
    }

    /// Whether this channel is the answer to `name`.
    fn claims(&self, name: &str) -> bool {
        self.origin.label().is_some_and(|leaf| leaf == name)
    }

    /// One gesture, carried out in the host's spelling and landed back in this
    /// box's — [`Origin`]'s two directions, spent at the one boundary they are
    /// spent at on the read path.
    fn answered(&self, question: &Value) -> Landed {
        let seat = self.seat.as_ref().map_err(Clone::clone)?;
        seat.answered(&self.origin.carried(question))
            .map(|reply| self.origin.labelled(reply))
    }

    /// [`answered`](Self::answered) held open. Each frame is labelled the same
    /// way, so a followed reply crosses back in this box's spelling too.
    fn followed(
        &self,
        question: &Value,
        on_frame: &mut dyn FnMut(Landed) -> bool,
    ) -> Result<(), String> {
        let seat = self.seat.as_ref().map_err(Clone::clone)?;
        seat.followed(&self.origin.carried(question), &mut |landed| {
            on_frame(landed.map(|reply| self.origin.labelled(reply)))
        })
    }
}

#[cfg(test)]
mod tests;
