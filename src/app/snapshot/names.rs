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
//! …and [`addressable`] is **which** sets those are at the boundary: the §3.1
//! workspace enumeration and the §5.1 #1 project one as disk holds them this
//! instant, not the copies the last derivation cached. See its own doc for the
//! barrier that buys.

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

    /// The **armed** workspace `name` addresses (§4.3, bl-ef16) — the round
    /// trip for [`fleet::Facts`](crate::fleet::Facts), whose wire spelling is
    /// the §3.1 name and whose pilot needs the directory.
    ///
    /// It resolves over the `cadence.yaml` **arming table's own keys**, not
    /// over the §3.1 enumeration [`ws_path`](Self::ws_path) reads. The two sets
    /// are not the same: an entry arms a directory verbatim, so a loop may be
    /// armed on a workspace the enumeration has not reached — and answering
    /// that loop "unknown workspace" would stop it planning at all, which is a
    /// behaviour change and not an addressing fix. `None` for a name no armed
    /// entry carries, and for one two of them share: an ambiguous leaf is
    /// refused here exactly as [`naming::by_leaf`] refuses it.
    pub fn armed_path(&self, name: &str) -> Option<PathBuf> {
        let mut hit = self
            .fleet
            .keys()
            .map(PathBuf::from)
            .filter(|p| naming::leaf(p) == name);
        let one = hit.next()?;
        hit.next().is_none().then_some(one)
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
/// make it one, which keeps the rule ("the derivation, never the §7.2 fold")
/// intact: this is disk answering, not optimism.
///
/// The same rule runs backwards for free: a workspace the §3.6 unmaking has
/// deleted leaves the resolution at once rather than at the next sweep.
///
/// It stands **at every intake**, which since bl-7942 is every caller there is.
/// bl-6c9e left exactly one on the cached copy — `AppModel::boundary_deps`,
/// whose one production caller was inside the render pass, where a frame does
/// no IO (§7.2) and holds the §3.4 raise claim instead. The window is gone and
/// so is that caller (bl-ab32): what still builds a `Deps` off a model is the
/// acceptance world standing in for the transport, and it stands where the
/// intake stands, so it asks here too. Every intake is off-frame, so the
/// authority is cheap enough to ask per gesture. The *derived*
/// per-workspace facts — trees, bills, the §3.5 join — stay as published,
/// because those are the walks that are not cheap, and every read of them is
/// aimed by the path this resolution produced. A newborn wall therefore answers
/// with the zeros it honestly has ([`workspace_stats`](crate::boundary::answer::workspace_stats)
/// over a tree no derivation holds yet), exactly as the raise claim's own fold
/// does one frontend up.
///
/// **Both nouns, since bl-3377.** bl-6c9e stated its own ruling as a rule about
/// existence — *"birth is a barrier because existence is a query"* — and then
/// folded one set, leaving the project noun on the cached copy. So a project
/// primed into the world was refused by every ball gesture, byte-identically to
/// a typo, until the next full sweep: `yog bl prime` then `/create` earned
/// `unknown project "proj"`, and the same line one sweep later succeeded. A
/// project is enumerated the same way a workspace is (§5.1 #1, one readdir of
/// the balls clones dir), and it simply was not included; it runs backwards for
/// free the same way, so a project removed stops resolving at once.
///
/// The *derived* per-project facts — the ball lists, the §3.5 join — stay as
/// published, exactly as the per-workspace ones do and for the same reason:
/// those are the walks that are not cheap, and every read of them is aimed by
/// the path this resolution produced.
///
/// `Arc` in, `Arc` out, so the steady state pays nothing: with both pairs of
/// sets already agreeing — every gesture but the one after a birth or a death —
/// the published derivation is handed straight back rather than cloned.
pub(crate) fn addressable(
    published: Arc<Snapshot>,
    workspaces: Vec<Workspace>,
    projects: Vec<PathBuf>,
) -> Arc<Snapshot> {
    if workspaces == published.workspaces && projects == published.projects {
        return published;
    }
    Arc::new(Snapshot {
        workspaces,
        projects,
        ..(*published).clone()
    })
}

#[cfg(test)]
mod tests;
