//! **The window's channel set, seated** (REMOTE §8.2, bl-670c): one
//! [`Asker`](crate::wire::asker::Asker) per channel, and the
//! [`Dial`](crate::wire::dial::Dial) the three routing threads each take one of.
//!
//! §8.2 verbatim: *"the window attaches to every channel it holds — the loopback
//! engine plus one channel per entry, each on the entry's own material"*, and
//! *"per channel, everything §3 ruled holds unchanged: the asker's pass at human
//! cadence, the poster's exactly-once act routed by the workspace it names, the
//! follow lane dialled at whichever channel hosts the focused conversation, the
//! searcher fanned out and unioned."* This module is the first clause; the other
//! three are one `Dial` each, and the difference between them is only what they
//! do with it.
//!
//! **A thread per channel is the isolation**, and it is why the askers are here
//! rather than being one thread over a set. A seat's dial has the kernel's own
//! connect for its bound, so a channel whose host is off the network is a thread
//! parked for as long as that takes — and the roster, the conversations and the
//! transcript of every *other* channel are answered on their own threads
//! meanwhile. §8.2 prices this outright: *"the window's cost is linear — one
//! channel, one asker pass per cadence period, per entry."*
//!
//! **A channel that cannot be dialled is not a channel that is missing.** Its
//! asker holds the sentence instead of a seat and answers every question
//! standing on that channel with it, so the slice says why rather than staying
//! empty — the bl-dc14 refusal discipline applied per entry, never the whole
//! shell, which stays reserved for the one wire the window cannot exist without.

use super::Engine;
use crate::wire::asker::{Asker, AskerThread};
use crate::wire::channels::EntryEnd;
use crate::wire::dial::Dial;
use crate::wire::entries::Entry;
use crate::xdg::Env;
use std::sync::Arc;

impl Engine {
    /// **One asker per channel, started**: this engine's own over loopback —
    /// the only one that seats the window (REMOTE §4.1) — and then one per
    /// entry, on that entry's own material and over the link end composed with
    /// its slice.
    ///
    /// `None` only for the loopback channel's own reasons (no listener, no
    /// window leaf, an end already taken), because that is the one channel this
    /// window cannot be a window without. An entry whose seat will not open is
    /// still given its asker: the refusal is what that channel answers.
    pub(crate) fn askers(&mut self, world: &Env, held: Vec<EntryEnd>) -> Option<Vec<AskerThread>> {
        let mut started = vec![self.asker(world)?.start()];
        for EntryEnd { entry, end } in held {
            started.push(Asker::entry(entry.seat(), end, Arc::clone(&self.repaint)).start());
        }
        Some(started)
    }

    /// **A seat on every channel** for one routing thread: this window's own
    /// engine, plus one per entry on that entry's own material.
    ///
    /// One per thread rather than one shared, for the reason each thread already
    /// minted its own loopback seat (REMOTE §6: a seat is a configuration and an
    /// address, so there is nothing to share). `None` on the one condition every
    /// off-frame half has always shared — a loopback seat this box cannot open.
    pub(crate) fn dial(&self, world: &Env, entries: &[Entry]) -> Option<Dial> {
        Some(Dial::compose(entries, self.window_seat(world).ok()?))
    }
}

#[cfg(test)]
mod tests;
