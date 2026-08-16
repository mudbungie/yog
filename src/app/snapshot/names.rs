//! The boundary's addressing over one snapshot (REMOTE §8, bl-f5f6): the four
//! reads of [`crate::naming`]'s single rule.
//!
//! **This is where a path becomes a name and back again, and it is one place.**
//! The frame holds paths — a focused workspace, a selected project — and spells
//! [`ws_name`](Snapshot::ws_name) / [`project_name`](Snapshot::project_name) at
//! the seam where a seat's selection becomes a gesture. The engine holds the
//! gesture and spells [`ws_path`](Snapshot::ws_path) /
//! [`project_path`](Snapshot::project_path) at the dispatch chokepoint. Both
//! read the *same enumerated sets off the same snapshot*, so the two directions
//! cannot disagree about what a name means — which is the whole reason the
//! resolution sits here rather than at each caller.
//!
//! …and [`addressable`] is **which** set that is at the boundary: the §3.1
//! enumeration as disk holds it this instant, not the copy the last derivation
//! cached. See its own doc for the barrier that buys.

use super::Snapshot;
use crate::binding::Workspace;
use crate::naming;
use std::path::{Path, PathBuf};
use std::sync::Arc;

impl Snapshot {
    /// Every enumerated workspace's path — the set a workspace name is derived
    /// over (§3.1).
    fn ws_paths(&self) -> Vec<PathBuf> {
        self.workspaces.iter().map(|w| w.path.clone()).collect()
    }

    /// The wire name of the workspace at `path` (§3.1: its leaf **is** its
    /// name). Total over any path — the enumeration is not consulted, because
    /// a workspace's name is not derived from the set it sits in.
    pub fn ws_name(&self, path: &Path) -> String {
        naming::leaf(path)
    }

    /// The workspace `name` addresses, or the refusal naming the token.
    pub fn ws_path(&self, name: &str) -> Result<PathBuf, String> {
        naming::by_leaf(&self.ws_paths(), name)
    }

    /// The wire name of the project at `path` (§5.1 #1) — the §11 roster label
    /// before its cosmetic elision.
    pub fn project_name(&self, path: &Path) -> String {
        naming::name_of(&self.projects, path)
    }

    /// The project `name` addresses, or the refusal naming the token.
    pub fn project_path(&self, name: &str) -> Result<PathBuf, String> {
        naming::resolve(&self.projects, name)
    }
}

/// **The workspace set the boundary addresses over is ASKED, never remembered**
/// (bl-6c9e): the published derivation with `live` — the §3.1 enumeration, three
/// readdirs — standing in for the workspace set it cached.
///
/// A gesture that founds a workspace returns before the worker has read it
/// (§7.2: a derivation is a pass, not a transaction), so a chokepoint resolving
/// names over the cached set refused the very name the reply it had just sent
/// made addressable: the documented `/prepare` → `/prompt` flow could not
/// compose two processes deep, and the window's own posted receipt earned
/// `unknown workspace` for the wall its previous act had founded. **Birth is a
/// barrier because existence is a query** — and no claim is held anywhere to
/// make it one, which is what keeps `boundary_deps`' rule ("the derivation,
/// never the §7.2 fold") intact: this is disk answering, not optimism.
///
/// The same rule runs backwards for free: a workspace the §3.6 unmaking has
/// deleted leaves the resolution at once rather than at the next sweep.
///
/// It stands **at the intake and only there**. The frame does no IO (§7.2) and
/// keeps its cached copy plus the §3.4 raise claim; every intake is already
/// off-frame, so the authority is cheap enough to ask per gesture. The *derived*
/// per-workspace facts — trees, bills, the §3.5 join — stay as published,
/// because those are the walks that are not cheap, and every read of them is
/// aimed by the path this resolution produced. A newborn wall therefore answers
/// with the zeros it honestly has ([`workspace_stats`](crate::boundary::answer::workspace_stats)
/// over a tree no derivation holds yet), exactly as the raise claim's own fold
/// does one frontend up.
///
/// `Arc` in, `Arc` out, so the steady state pays nothing: with the two sets
/// already agreeing — every gesture but the one after a birth or a death — the
/// published derivation is handed straight back rather than cloned.
pub(crate) fn addressable(published: Arc<Snapshot>, live: Vec<Workspace>) -> Arc<Snapshot> {
    if live == published.workspaces {
        return published;
    }
    Arc::new(Snapshot {
        workspaces: live,
        ..(*published).clone()
    })
}

#[cfg(test)]
mod tests;
