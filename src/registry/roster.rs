//! **The workspace's registered clients, joined with what each one is** (REMOTE
//! §5, bl-4e08) — the derivation behind `Query::Clients` and behind the
//! navigator's own clients section.
//!
//! REMOTE §5: *"The workspace surface renders its registered clients — present
//! or absent — and each one's advertised tools, live: the seat sees the flap;
//! the model's prefix does not."* This is the one function that says it, so the
//! window and a headless seat read the identical rows (§8.5's parity
//! discipline).
//!
//! **Nothing here is stored.** The registered set is the §4.1 directory
//! listing, the advertised set is that client's own document, and presence is
//! the wire server's RAM — three reads joined at the moment they are asked,
//! which is why a client seated by `touch` and a client that just disconnected
//! both show up correctly with nothing to invalidate.
//!
//! **`local` is filtered by the rule that already reserves it.** The window and
//! every other in-world caller own a directory here but hold no certificate and
//! are not scoped (§4.1), so
//! [`Client::parse`](super::Client::parse) refuses the name and the roster
//! skips it — one rule, not a second special case.

use std::collections::BTreeSet;
use std::path::Path;

use super::presence::Presence;
use super::tools::{self, Tool};
use super::{CLIENTS, Client};

/// One registered client of a workspace, as every seat renders it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientRow {
    /// Its identity — the certificate common name (REMOTE §2).
    pub client: String,
    /// Whether it holds a live connection right now (REMOTE §5).
    pub present: bool,
    /// What it advertises. Empty for a client that has advertised nothing,
    /// which is every client until it first connects as a tool host.
    pub tools: Vec<Tool>,
}

/// The clients registered in `workspace`, sorted by identity, each with its
/// presence and its advertised set.
pub fn roster(state_root: &Path, presence: &Presence, workspace: &str) -> Vec<ClientRow> {
    let live = presence.live();
    names(state_root)
        .into_iter()
        .filter_map(|name| Client::parse(&name).ok())
        .filter(|client| super::registered(state_root, client).contains(workspace))
        .map(|client| ClientRow {
            present: live.contains(&client.name()),
            tools: tools::read(state_root, &client),
            client: client.name(),
        })
        .collect()
}

/// Every directory name under `clients/`, sorted — a registry that is not there
/// yet is the empty set, which is the posture of every box before the operator
/// seats a first client (§4.1).
fn names(state_root: &Path) -> BTreeSet<String> {
    let Ok(entries) = std::fs::read_dir(state_root.join(CLIENTS)) else {
        return BTreeSet::new();
    };
    entries
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect()
}

#[cfg(test)]
mod tests;
