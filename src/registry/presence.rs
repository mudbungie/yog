//! **Presence: which registered clients are connected right now** (REMOTE §5,
//! bl-4e08) — the live half of the two facts a tool host has.
//!
//! REMOTE §5: *"Advertisement is durable; presence is live. … **Presence** —
//! connected right now — is connection-scoped RAM. Two facts because two rates
//! of change: a tool set changes when the operator reconfigures a machine;
//! presence changes with every network blip."* So this is deliberately **not**
//! a file: a registration is a fact about the world and converges over disk
//! (§4.1), while a connection is a fact about *this process at this instant*
//! and writing it would put a connectivity-rate fact into a substrate the file
//! religion promises is durable.
//!
//! **It is one entry per live connection, not a set.** One client may hold two
//! seats at once — a phone and a laptop presenting the same certificate — and
//! the second closing must not unsay the first. Counting dissolves that
//! without a case for it, and since bl-1be7 what is counted carries something:
//! each entry is the **corpus edition** that connection stated in its preface
//! (REMOTE §3.2). The count is the list's length, so there is no second number
//! to keep true — a seat's capability is a fact about the connection it
//! arrived on, and this map is where a connection's identity already lives.
//! Nothing answers with it yet; `reply/clients` is where it would go.
//!
//! **Entering is RAII and that is the whole protocol.** The wire server takes a
//! [`Live`] when a connection first names its client and drops it when the
//! connection ends, however it ends: a clean close, a refused frame, a peer
//! that vanished, a panicking thread. There is no leave verb to forget to call,
//! which is what makes "connected right now" true rather than aspirational.
//!
//! **The lock lives in [`state`](crate::state)** (AGENTS.md rule 7): this is
//! genuine cross-thread hand-off state, so the map's cell and its poison-immune
//! lock are [`PresenceCell`] and [`lock_presence`] there.

use std::collections::BTreeSet;

use super::Client;
use crate::state::{PresenceCell, lock_presence};

/// The process's live-connection map, shared by handle: the wire server enters
/// identities into it, and every answer reads them out of it. A default one is
/// the posture of a box with no wire at all — no connections, which is the
/// general path with no input rather than a case of its own.
#[derive(Clone, Default)]
pub struct Presence {
    cell: PresenceCell,
}

impl Presence {
    /// Register one live connection for `client`, stating the corpus
    /// `edition` its preface carried (REMOTE §3.2), until the returned [`Live`]
    /// drops.
    pub fn enter(&self, client: &Client, edition: u32) -> Live {
        let name = client.name();
        lock_presence(&self.cell)
            .entry(name.clone())
            .or_default()
            .push(edition);
        Live {
            cell: self.cell.clone(),
            name,
            edition,
        }
    }

    /// Whether `client` holds at least one live connection.
    pub fn is_live(&self, client: &str) -> bool {
        lock_presence(&self.cell).contains_key(client)
    }

    /// Every identity holding one, sorted — the roster's own read (REMOTE §5's
    /// *"present or absent"*).
    pub fn live(&self) -> BTreeSet<String> {
        lock_presence(&self.cell).keys().cloned().collect()
    }

    /// What every live connection of `client` said it can spell: one edition
    /// per connection, ascending, and empty for a client holding none.
    ///
    /// A list rather than a number, because capability is a fact about a
    /// *connection* and a client may hold two of different builds. Answering
    /// the newest would over-promise a routed call landing on the older one,
    /// and answering the oldest would under-report a seat that is present and
    /// current — so the judgement stays with the reader that needs one, and
    /// the read stays honest until there is one.
    pub fn editions(&self, client: &str) -> Vec<u32> {
        let mut out = lock_presence(&self.cell)
            .get(client)
            .cloned()
            .unwrap_or_default();
        out.sort_unstable();
        out
    }
}

/// One connection's presence. Dropping it releases exactly that connection's
/// count, and removes the identity when it was the last — an identity with a
/// zero beside it would be a client the map says it knows and does not.
pub struct Live {
    cell: PresenceCell,
    name: String,
    edition: u32,
}

impl Drop for Live {
    fn drop(&mut self) {
        let mut map = lock_presence(&self.cell);
        // `entry` rather than `get_mut`: an identity the map has already
        // forgotten is the same act with nothing to take out of it, and a case
        // for it would be a branch no test can reach.
        let held = map.entry(self.name.clone()).or_default();
        if let Some(at) = held.iter().position(|stated| *stated == self.edition) {
            held.remove(at);
        }
        if held.is_empty() {
            map.remove(&self.name);
        }
    }
}

#[cfg(test)]
mod tests;
