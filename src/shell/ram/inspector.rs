//! The §11 Altitude-2 inspector's cross-frame RAM (§5.3, per-instance viewport
//! state — *which data you look at*, never durable), its own file per §12's line
//! budget: the selections the inspector's render writes back to, and the fork
//! composer's own draft beside them. Inert data; the tabs themselves live in
//! `shell/inspector/*`.
//!
//! **The four view-model memos are gone** (REMOTE §9.7, bl-13f9). Transcript,
//! steps, rail and files were memoized per published snapshot because each
//! built itself off disk on the paint path; every one of them is a standing
//! wire question now, and an answer *is* the cached fold — refreshed at the
//! asker's cadence rather than the derivation's — so the memo was a second
//! cache in front of one that already existed. What remains is
//! [`fork_memo`](InspectorState::fork_memo), whose subject is the V2 composer's
//! choices and not a §11 tab.

use crate::app::SnapMemo;
use crate::inspector::Ephemera;
use crate::steps_view::StepTab;
use std::path::PathBuf;

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
            fork: None,
            fork_memo: SnapMemo::default(),
        }
    }
}
