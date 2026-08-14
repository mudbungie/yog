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

use super::Snapshot;
use crate::naming;
use std::path::{Path, PathBuf};

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

#[cfg(test)]
mod tests;
