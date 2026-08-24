//! **Composing the window's channel set from a world** (REMOTE §8.2, bl-670c):
//! the [entries](crate::wire::entries) this box holds, read once, turned into
//! one channel per entry and one link end per channel.
//!
//! Its own file for one reason: it is the only part of the set that touches the
//! disk. [`Channels`] itself is a union over values a caller hands it, which is
//! what lets a fixture compose a slice with no world behind it at all.

use super::Channels;
use crate::wire::channel::Channel;
use crate::wire::entries::Entry;
use crate::wire::link::{Link, LinkEnd};
use crate::xdg::Env;

/// **One entry channel's other end** (bl-670c): what the channel is, and the
/// link end its [`asker`](crate::wire::asker) answers on.
///
/// Minted beside the channel itself by [`compose`], for
/// [`link::pair`](crate::wire::link::pair)'s reason: neither end is useful alone, so
/// the pairing is the composition's own act rather than a join two lists have
/// to keep in step afterwards.
pub struct EntryEnd {
    /// The entry as it was read — the material its threads seat on, and the two
    /// names [`Origin`](crate::wire::channel::Origin) maps between.
    pub entry: Entry,
    /// The asker's end of this channel's slice.
    pub end: LinkEnd,
}

/// **The whole set and the ends it is answered on** (REMOTE §8.2): the model's
/// half — this window's own engine over `local`, plus one channel per entry
/// `world` holds, in leaf order — and one [`EntryEnd`] per entry channel for
/// the engine to put an asker on.
///
/// A free function rather than a constructor because it hands back two things
/// and only one of them is a [`Channels`]; the model takes the set, the engine
/// keeps the ends until a face asks for its off-frame halves.
pub fn compose(world: &Env, local: Link) -> (Channels, Vec<EntryEnd>) {
    let mut channels = Vec::new();
    let mut ends = Vec::new();
    for entry in crate::wire::entries::entries(world) {
        let (link, end) = crate::wire::link::pair();
        channels.push(Channel::entry(&entry, link));
        ends.push(EntryEnd { entry, end });
    }
    (Channels::held(local, channels), ends)
}

#[cfg(test)]
mod tests;
