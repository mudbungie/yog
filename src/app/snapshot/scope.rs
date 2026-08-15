//! **Scoping is authorization, and it is one filter** (REMOTE §1.5, §4;
//! bl-8bbc): the published derivation narrowed to the workspaces one client is
//! registered in.
//!
//! **Everything else is ABSENT, not forbidden.** §4: *"enumeration replies
//! simply do not contain unregistered workspaces, the same shape as a workspace
//! that does not exist. Scope errors that confirm existence are a
//! disclosure."* Narrowing the snapshot is what makes that structural rather
//! than a promise kept twenty times:
//!
//! - [`ws_rows`](crate::boundary::answer::ws_rows) maps `workspaces`, so the
//!   roster answers the registered set and nothing says a name was withheld.
//! - [`ws_path`](super::Snapshot::ws_path) resolves over `workspaces` too, so a
//!   gesture naming an unregistered workspace earns
//!   [`by_leaf`](crate::naming::by_leaf)'s own `unknown workspace` refusal —
//!   the **identical bytes** a name nobody ever founded earns. There is no
//!   scope error to write, because there is no scope branch.
//! - Every other read is aimed by the path that resolution produced, so none of
//!   them can be reached at all.
//!
//! One filter, at one place, ahead of the dispatch table — the same shape
//! REMOTE §8's name resolution took, and for the same reason: twenty arms
//! re-deriving an authorization is twenty chances to forget one.
//!
//! **The workspace is the whole trust domain** (§1.5, and §11's standing
//! rejection of per-verb ACLs). So the narrowing is exactly the
//! workspace-keyed fields; the project set, the balls projection, the §3.5 join
//! and the `ops.jsonl` trail are world-wide facts this design does not divide,
//! and pretending otherwise here would be a policy layer §11 refuses until a
//! second human exists.

use super::Snapshot;
use crate::naming::leaf;
use std::collections::BTreeSet;

impl Snapshot {
    /// This derivation as a client registered in `allowed` sees it: every
    /// workspace-keyed field filtered to those names, everything else untouched.
    ///
    /// Total over any name set — an empty one yields a snapshot with no
    /// workspace at all, which is the honest view a certificate the operator has
    /// not seated gets, and which every read surface already answers without a
    /// bootstrap branch (`Snapshot::empty` is that same shape).
    #[must_use]
    pub fn scoped(&self, allowed: &BTreeSet<String>) -> Snapshot {
        let keep = |path: &std::path::Path| allowed.contains(&leaf(path));
        Snapshot {
            workspaces: self
                .workspaces
                .iter()
                .filter(|w| keep(&w.path))
                .cloned()
                .collect(),
            trees: self
                .trees
                .iter()
                .filter(|(path, _)| keep(path))
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
            bills: self
                .bills
                .iter()
                .filter(|(path, _)| keep(path))
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
            growth: self
                .growth
                .iter()
                .filter(|g| keep(&g.workspace))
                .cloned()
                .collect(),
            // Keyed by workspace NAME rather than path (bl-66fb), so the same
            // predicate reads it one step shorter.
            fleet: self
                .fleet
                .iter()
                .filter(|(name, _)| allowed.contains(*name))
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
            ..self.clone()
        }
    }
}

#[cfg(test)]
mod tests;
