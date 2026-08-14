//! **Which cached liveness observations get thrown away, and on which signal**
//! (DESIGN §10, §7.2).
//!
//! Only macOS has anything to throw away: its `lsof` probe is slow, so its
//! answers sit behind a 2 s TTL cache ([`probe_cache`](crate::git_tree)), and
//! an answer held is an answer that can be wrong. Linux re-scans `/proc` every
//! derivation — cheap, always definite — so every eviction here is a no-op
//! there and this module is pure policy either way.
//!
//! An agent's liveness changes in two directions, and **each direction has
//! exactly one signal that can carry it**:
//!
//! - A live agent **dies silently.** A released flock emits no filesystem
//!   event, so no watcher will ever mention it; the only way to learn it is to
//!   look again. That is the 2 s cheap sweep's poll ([`Deriver::reprobe_live`]),
//!   and it looks only at agents currently reading Live/InFlight, because they
//!   are the only ones that can die.
//! - A resting agent **comes alive.** A driver taking the lock writes, and
//!   writing is exactly what a watcher hears — so the event is the signal, and
//!   nothing needs polling. That is [`Deriver::refresh_liveness`], and it takes
//!   the agents the sweep does not.
//!
//! The two sets are complements, which is the point: neither pays for the
//! other's case. A streaming `response.json` append storm is a watcher event
//! per append on an agent **already known live** — so it evicts nothing, and
//! stays collapsed on the cache the cache exists for.

use super::Deriver;
use crate::git_tree::{AgentState, GitTree};
use crate::watch::Mark;
use std::path::{Path, PathBuf};

/// Whether `tree` holds an agent that could have died *silently* — a Live/
/// InFlight agent; a released flock emits no fs event (§7.2 targeted re-probe).
pub(crate) fn needs_liveness_reprobe(tree: &GitTree) -> bool {
    tree.agents
        .iter()
        .any(|a| matches!(a.state, AgentState::Live | AgentState::InFlight))
}

impl Deriver {
    /// The cheap sweep's half: evict every agent's lock observation in each
    /// workspace holding a live one, and mark that workspace [`Mark::Poll`] so
    /// the re-derivation observes rather than remembers.
    ///
    /// [`Mark::Poll`] and not [`Mark::Watch`] because this is a poll of
    /// **process state**, not of the filesystem: a change it finds is the poll
    /// working, never a dropped event (§7.2 provenance).
    pub(super) fn reprobe_live(&mut self) {
        let mut live: Vec<(PathBuf, Vec<String>)> = Vec::new();
        for (path, tree) in &self.trees {
            if needs_liveness_reprobe(tree) {
                let ids = tree.agents.iter().map(|a| a.agent_id.clone()).collect();
                live.push((path.clone(), ids));
            }
        }
        for (path, ids) in live {
            self.probes.invalidate_liveness(&path, &ids);
            self.schedule.mark([(path, Mark::Poll)]);
        }
    }

    /// The watcher's half: a filesystem change under an agent at rest may be a
    /// driver *arriving*, and the cache would answer from store for up to a
    /// whole TTL — so the row stays at rest, and the §11 flight strip stays
    /// down, while a model call is already running.
    ///
    /// A root with no tree is not a workspace (a root deriving for the first
    /// time, or one whose read failed) — nothing observed yet, so nothing to
    /// forget.
    pub(super) fn refresh_liveness(&self, root: &Path) {
        let Some(tree) = self.trees.get(root) else {
            return;
        };
        let resting: Vec<String> = tree
            .agents
            .iter()
            .filter(|a| !matches!(a.state, AgentState::Live | AgentState::InFlight))
            .map(|a| a.agent_id.clone())
            .collect();
        self.probes.invalidate_liveness(root, &resting);
    }
}

#[cfg(test)]
mod tests;
