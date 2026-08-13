//! The §11 Altitude-2 inspector's cross-frame RAM (§5.3, per-instance viewport
//! state — *which data you look at*, never durable), its own file per §12's line
//! budget: the selections the inspector's render writes back to, and the
//! per-snapshot memos that keep its disk and `git` reads off the paint path
//! (§7.2). Inert data; the tabs themselves live in `shell/inspector/*`.

use crate::app::SnapMemo;
use crate::git_tree::AgentState;
use crate::inspector::Ephemera;
use crate::steps_view::{StepTab, StepsView};
use crate::transcript::Transcript;
use std::path::PathBuf;
use std::sync::Arc;

/// The §11 Altitude-2 inspector's RAM ephemera (§5.3, per-instance viewport
/// state — *which data you look at*, never durable): the Raw toggle (one flag,
/// honoured by Transcript, Steps and Inbox — the tabs that parse a file, §11),
/// the Steps selection + drill-in tab, and the [`Ephemera`] bundle every other
/// selection lives in. All re-derive at startup; nothing here reaches
/// `ui.json` — the transcript's *auto-state* does (§4.1), but which rows you
/// have since flipped by hand does not.
pub struct InspectorState {
    pub raw: bool,
    pub step_sel: Option<usize>,
    pub step_tab: StepTab,
    /// Every selection the inspector's own render writes back to (§5.3): the
    /// jsonview collapse set, the Files selection, the transcript's per-row
    /// fold overrides, the rail's pinned notch and the Work tab's picked file.
    /// One bundle, handed to [`crate::inspector::render`] whole.
    pub eph: Ephemera,
    /// Per-snapshot memo of the focused agent's steps view (§7.2 `SnapMemo`,
    /// bl-e90a): read by the center's auth/wound banners every frame and by the
    /// Steps tab and the transcript's crossing rules — one disk build per
    /// snapshot, shared by all of them, never one per frame.
    pub(crate) steps_memo: SnapMemo<(PathBuf, String, AgentState), StepsView>,
    /// Per-snapshot memo of the focused agent's transcript (bl-e90a). `Arc` so
    /// a frame hands the view-model on without copying the payload bytes.
    pub(crate) tx_memo: SnapMemo<(PathBuf, String), Arc<Transcript>>,
    /// Per-snapshot memo of the focused agent's rail (§7.2 `SnapMemo`). The
    /// build folds each child's `steps/<id>` spend and asks its governing
    /// config, so it is a disk read and must never run per frame.
    pub(crate) rail_memo: SnapMemo<(PathBuf, String), crate::rail::Rail>,
    /// Per-snapshot memo of the Files listing, live or pinned. The pinned arm
    /// is a `git ls-tree`, which is exactly the per-frame git read STORIES
    /// §S7 point 3 refused; the commit rides the key so a re-pin re-reads and
    /// nothing else does.
    pub(crate) files_memo:
        SnapMemo<(PathBuf, String, Option<String>), crate::files_view::FilesView>,
    /// Per-snapshot memo of the §11 Work tab's read (§5.1 #32). Every arm of
    /// it forks `git` against a *project* repo, so it must never run per frame
    /// — the same discipline the pinned Files listing beside it keeps.
    pub(crate) work_memo: SnapMemo<PathBuf, Vec<crate::workdiff::Attempt>>,
    /// Per-snapshot memo of the selected file's patch, keyed on the file so
    /// picking another re-reads and scrolling does not.
    pub(crate) work_patch_memo:
        SnapMemo<(PathBuf, Option<crate::workdiff::WorkFile>), Option<crate::files_view::Preview>>,
    /// The fork composer at the pinned notch (VISION V2), and the choices it
    /// offers. RAM by the same §13.1 argument as `notch_sel` beside it: a
    /// half-typed counterfactual is a draft, and drafts are ephemera until
    /// they are fired. `composer` is `None` until the operator pins something
    /// and the seat seeds one; the choices are memoized per snapshot because
    /// deriving them reads the workspace's config branches off disk.
    pub(crate) fork: Option<crate::fork::composer::Composer>,
    pub(crate) fork_memo: SnapMemo<(PathBuf, String), crate::fork::Choices>,
}

impl Default for InspectorState {
    fn default() -> Self {
        Self {
            raw: false,
            step_sel: None,
            step_tab: StepTab::Meta,
            eph: Ephemera::default(),
            steps_memo: SnapMemo::default(),
            tx_memo: SnapMemo::default(),
            rail_memo: SnapMemo::default(),
            files_memo: SnapMemo::default(),
            work_memo: SnapMemo::default(),
            work_patch_memo: SnapMemo::default(),
            fork: None,
            fork_memo: SnapMemo::default(),
        }
    }
}
