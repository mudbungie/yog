//! The §11 Work tab's glue (§5.1 #32): the focused conversation's workspace,
//! its attempts, and the file whose patch the operator asked for.
//!
//! Coverage-excluded like the rest of `shell/*`: both calls here are into
//! [`crate::workdiff`], which tests them. What lives here is the **memo**
//! discipline — every arm of this read forks `git` against a project repo, so
//! it runs at most once per published snapshot (§7.2 `SnapMemo`), never per
//! frame. The listing is keyed on the workspace and the patch on the picked
//! file, so re-picking re-reads exactly one file and scrolling re-reads
//! nothing.

use std::path::Path;
use std::sync::Arc;

use crate::AppModel;
use crate::files_view::Preview;
use crate::keymap::InspectorTab;
use crate::workdiff::{self, Attempt, WorkFile};

use super::super::InspectorState;

/// The workspace's attempts, once per snapshot. An inactive tab reads nothing
/// at all — the same rule the Files and Transcript builds keep.
pub fn build(
    tab: InspectorTab,
    model: &AppModel,
    inspector: &mut InspectorState,
    ws: &Path,
) -> Vec<Attempt> {
    if tab != InspectorTab::Work {
        return Vec::new();
    }
    let snap = Arc::clone(model.derivation());
    inspector
        .work_memo
        .read(&snap, ws.to_path_buf(), &mut || workdiff::read(&snap, ws))
        .clone()
}

/// The picked file's patch, once per (snapshot, file). Nothing picked is
/// `None`, which the tab renders as its own invitation rather than an error.
pub fn patch(
    model: &AppModel,
    inspector: &mut InspectorState,
    ws: &Path,
    attempts: &[Attempt],
    picked: Option<&WorkFile>,
) -> Option<Preview> {
    let snap = Arc::clone(model.derivation());
    let key = (ws.to_path_buf(), picked.cloned());
    inspector
        .work_patch_memo
        .read(&snap, key, &mut || {
            picked.and_then(|file| workdiff::patch(&snap, attempts, file))
        })
        .clone()
}
