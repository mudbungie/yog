//! The **§3.4 raise claim** (DESIGN §3.4, §7.2; REMOTE §9.7 class 2, bl-7407):
//! the wall a landed start just founded, held until the derivation reads it.
//!
//! Focus is a §3.1 **name** now, and a name resolves against the enumerated
//! workspaces. The raise is the one moment that ordering runs backwards:
//! `lernie new` has returned and the wall exists on disk, but the worker has
//! not enumerated it, so for as long as one derivation takes the focus names a
//! workspace no set carries — and the composer's bare Enter, which resolves the
//! focused name to fire into it, would resolve into the *previous* wall. That
//! is the defect the raise-adoption fix closed (bl-9acf) and it must not
//! re-open.
//!
//! **The answer is the §3.4 start claim's own shape, one noun up** (the
//! operator's ruling on this ball; the synchronous re-derive at the receipt was
//! refused for putting a disk walk on the receipt path). Both are the same
//! thing about different nouns:
//!
//! | | the conversation claim ([`Echo`](super::echo::Echo)) | the wall claim (this) |
//! |---|---|---|
//! | holder | `Option<Echo>`, per-instance RAM (§13.1) | `Option<PathBuf>`, per-instance RAM |
//! | held by | what the fire knows — the minted §3.3 name | what the fire knows — the path it founded |
//! | retired by | the derivation showing the message | the derivation showing the workspace |
//! | folded at | [`echo::compose`](super::echo::compose) | [`echo::compose`](super::echo::compose) |
//!
//! They are one mechanism at one seam, not two: `compose` is *"the one place
//! snapshot and the non-derived facts meet… a third such fact is a third
//! argument here rather than a third mechanism"*, and this is that third
//! argument. They stay two *values* because a raise carries no message and a
//! send raises no wall — the raise's whole content is the path, and folding it
//! into a struct whose every other field is about one message would make both
//! optional inside a value that is neither.
//!
//! Folding rather than resolving at each door is what keeps it one source: the
//! painted snapshot enumerates the raised wall, so `ws_path`, the tab bar, the
//! centre pane and the composer all read the same set and none of them knows a
//! claim exists.

use super::{AppModel, Snapshot};
use crate::binding::{Workspace, WorkspaceKind};
use crate::naming;
use std::path::Path;

/// Fold the claimed wall into the snapshot a frame paints: one enumerated
/// [`Workspace`] and the empty tree it honestly has. Named, because a raise
/// founds under yog's own flat names root and only ever there (§3.1) — the
/// operator typed the name and [`validate_workspace_name`](AppModel::validate_workspace_name)
/// already refused any that collides with an existing one, so the fold can
/// never make [`by_leaf`](crate::naming::by_leaf) ambiguous.
///
/// The tree is `or_default()` — no commits, no agents — because that is what a
/// wall raised one instant ago has, and it is what the centre pane should
/// paint: an empty workspace with the keyboard in its composer, not a blank
/// frame.
pub(super) fn fold(snap: &mut Snapshot, raised: &Path) {
    snap.workspaces.push(Workspace {
        path: raised.to_path_buf(),
        kind: WorkspaceKind::Named {
            name: naming::leaf(raised),
        },
    });
    snap.trees.entry(raised.to_path_buf()).or_default();
}

impl AppModel {
    /// Adopt the workspace a landed start resolved (§3.4 — *a start focuses
    /// what it started*): focus it by `name`, and **claim the wall** so the name
    /// resolves for as long as the derivation has not read it.
    ///
    /// The claim is taken unconditionally rather than only when the enumeration
    /// misses. Every landed `Prepare` comes here — the raise, ▶ Continue's own
    /// claimant wall, the bootstrap, an ordinary bare start into the workspace
    /// already focused — and a start into an enumerated one is simply the claim
    /// retired the instant it is made ([`Self::adopt_raised`] runs first). One
    /// rule, not a raise/re-focus branch asking the same question twice.
    ///
    /// **`raised` is `None` for a workspace a §8.2 entry hosts** (bl-e349), and
    /// that is the claim's own premise rather than an exception to it. The claim
    /// exists to carry a wall from the instant `lernie new` founded it **on this
    /// box** to the instant this box's derivation reads it; a workspace founded
    /// on its host has no local directory to enumerate and no such gap — its row
    /// arrives on the entry's own slice ([`start_path`](AppModel::start_path)).
    /// The focus is taken either way: a start focuses what it started, wherever
    /// that is.
    pub(crate) fn adopt_workspace(&mut self, name: &str, raised: Option<&Path>) {
        self.raised = raised.map(Path::to_path_buf);
        // The same two moves [`refresh`](AppModel::refresh) makes, in its
        // order, run here as well: a claim taken mid-frame must resolve for the
        // rest of that frame, and retiring before folding is what keeps the
        // painted snapshot from enumerating one workspace twice.
        self.adopt_raised();
        self.refold();
        self.focus_workspace(name);
    }

    /// **The claim at the ROW altitude** (REMOTE §9.7, bl-296f) — the same
    /// optimism [`fold`] puts into a snapshot, folded instead into an answered
    /// `Query::Workspaces` listing.
    ///
    /// Two projections of one fact, for [`rows::with_echo`](super::echo::rows)'
    /// own reason and not a second one: the §11 tab bar reads a `Reply` now, so
    /// it never sees the composed snapshot, and bl-adcb's *"a seat that reads
    /// over a wire has no optimism"* taken as the last word would re-open
    /// bl-9acf exactly — the bare Enter after a raise firing into the previous
    /// wall, because the wall it was raised in has no tab and no resolvable
    /// name. **A seat's optimism reaches whatever that seat actually reads**
    /// (bl-44e9's ruling), and what this one reads is rows.
    ///
    /// The claim is retired against the same derivation this answer is made
    /// from ([`adopt_raised`](Self::adopt_raised) reads `derived`), so the
    /// listing can never carry one wall twice.
    pub fn raised_rows(
        &self,
        mut rows: Vec<crate::boundary::reply::WsRow>,
    ) -> Vec<crate::boundary::reply::WsRow> {
        let Some(raised) = self.raised.as_ref() else {
            return rows;
        };
        let name = naming::leaf(raised);
        if rows.iter().any(|r| r.workspace == name) {
            return rows;
        }
        rows.push(crate::boundary::reply::WsRow {
            // A wall the derivation has not read has no lineage derived
            // either — the claim's own empty shape, not a case of its own.
            config_tip: None,
            workspace: name.clone(),
            kind: WorkspaceKind::Named { name },
            // A wall raised one instant ago holds nothing and is pinned
            // nowhere: the zeros are what it honestly has, not a placeholder.
            attention: 0,
            agents: 0,
            running: false,
            pinned: None,
        });
        rows
    }

    /// Retire the claim once the derivation carries the wall — the echo's
    /// retirement predicate one noun up, asked every frame and free with
    /// nothing claimed. Run **before** the fold, so the painted snapshot can
    /// never enumerate the same workspace twice.
    pub(super) fn adopt_raised(&mut self) {
        if self
            .raised
            .as_ref()
            .is_some_and(|ws| self.derived.workspaces.iter().any(|w| &w.path == ws))
        {
            self.raised = None;
        }
    }
}

#[cfg(test)]
mod tests;
