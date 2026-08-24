//! **Every channel this box serves** (REMOTE §8.2, bl-4e31): the tool host's
//! own resolution over the [`entries`](crate::wire::entries) beside its flat
//! root, and the fan-out that serves them all at once.
//!
//! A tool host presents what its machine can run and then waits for work. Which
//! *engines* it presents to is the §8.2 question one noun over from the seat's:
//! none of the three gestures it speaks names a workspace (an advertisement and
//! the routing leg's two address a **machine**, `boundary::address`), so there
//! is no name here to resolve and no name to rewrite — what an entry adds is a
//! second engine to be present at. So the host serves the flat channel *and*
//! one channel per entry, each on that entry's own material, which is what
//! makes its advertisement land under that channel's own identity: one
//! certificate is one client (REMOTE §2), and separation is the absence of a
//! mechanism rather than one.
//!
//! **Serial per channel, concurrent across them.** REMOTE §10's
//! deferred-concurrency row is per-host and untouched: one thread per channel,
//! each running the same one-invocation-at-a-time loop it ran when the flat
//! root was the only channel there was. A channel is a held connection for as
//! long as its follow-read takes, so they cannot share one.
//!
//! **A refusal is one channel's, never the set's** — the same discipline
//! `entries` draws, one layer up. A half-provisioned entry is said once and its
//! neighbours are served; only a box with *no* channel at all refuses outright,
//! and with zero entries that refusal is the flat root's own sentence, verbatim
//! and alone, which is exactly what a tool host said before §8.2 existed.

use super::config::Local;
use crate::wire::material::Material;
use crate::wire::{entries, seat};
use crate::xdg::Env;

/// One engine this host serves.
pub(crate) struct Channel {
    /// The entry's leaf, or `None` for the flat directory. The box's own root
    /// carries no name because it is not one relationship among others: it is
    /// the one this box holds without naming it (§8.2).
    pub(crate) name: Option<String>,
    /// The material that reaches it.
    pub(crate) material: Material,
}

impl Channel {
    /// `sentence`, said as this channel's own. An entry's is prefixed with the
    /// leaf, because a box holding several needs to know which one spoke; the
    /// flat root's is bare, so a box with no entries reads exactly as it always
    /// has.
    pub(crate) fn said(&self, sentence: &str) -> String {
        match &self.name {
            Some(leaf) => format!("{leaf}: {sentence}"),
            None => sentence.to_owned(),
        }
    }
}

/// Every channel this box is provisioned for, and the sentence of every one it
/// is not. Both halves, because a host with three good entries and one bad one
/// must serve three and say one.
pub(crate) fn channels(world: &Env) -> (Vec<Channel>, Vec<String>) {
    let mut held = Vec::new();
    let mut refused = Vec::new();
    match seat::flat(world) {
        Ok(material) => held.push(Channel {
            name: None,
            material,
        }),
        Err(reason) => refused.push(reason),
    }
    for entry in entries::entries(world) {
        match entry.channel {
            Ok(material) => held.push(Channel {
                name: Some(entry.leaf),
                material,
            }),
            Err(reason) => refused.push(reason),
        }
    }
    (held, refused)
}

/// Serve every channel until each has stopped, and answer what stopped them.
///
/// One thread per channel and a channel to collect their sentences on: the
/// collection ends when every sender has gone, which is total — a thread that
/// died in any way has dropped its sender, so there is no join verdict to read
/// and no arm here that no state of the world can reach.
pub(crate) fn fan(set: &[Local], held: Vec<Channel>) -> String {
    let (tx, rx) = std::sync::mpsc::channel();
    for channel in held {
        let tx = tx.clone();
        let set = set.to_vec();
        std::thread::spawn(move || {
            let said = channel.said(&super::hold(&set, &channel.material));
            drop(tx.send(said));
        });
    }
    drop(tx);
    rx.iter().collect::<Vec<String>>().join("\n")
}

#[cfg(test)]
mod tests;
