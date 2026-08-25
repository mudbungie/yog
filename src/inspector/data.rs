//! What the §11 inspector renders **from** — the pre-built view-models the
//! shell moves in, and the caller-owned RAM ephemera the render writes back to.
//!
//! Split from [`super`] at §12's budget on the seam the pane already had: the
//! parent is the tab *dispatch* and the two views it owns outright, this is the
//! data that dispatch is over. Nothing here paints.

use std::collections::HashSet;

use crate::config_edit::branch::GoverningConfig;
use crate::files_view::{FilesView, Preview};
use crate::inboxview::InboxEntry;
use crate::rail::{Pin, Rail};
use crate::science;
use crate::steps_view::{StepDetail, StepTab, StepsView};
use crate::transcript::{AutoExpand, Transcript};
use crate::workdiff::WorkFile;

/// The caller-owned viewport ephemera the inspector mutates (§5.3) — the
/// jsonview collapse set, the Files selection, the transcript's per-row fold
/// overrides, the rail's pinned notch, and the Work tab's selected file.
///
/// **One bundle, not one parameter each.** They are the same kind of thing —
/// *which data you are looking at*, held in RAM and re-derived at startup — so
/// a tab that grows a selection grows a field here rather than widening every
/// signature between the shell and the paint.
#[derive(Default)]
pub struct Ephemera {
    pub json_collapsed: HashSet<String>,
    /// The Files tab's selected entry, by **path** — the parameter the next
    /// `Query::Files` carries (bl-13f9), never a row number into a listing that
    /// landed a round trip ago. The `work_sel` shape, one tab over.
    pub files_sel: Option<String>,
    pub tx_folded: HashSet<String>,
    pub notch_sel: Option<usize>,
    pub work_sel: Option<WorkFile>,
    /// The Work tab's fan group card seat (bl-77bc): the compare picks — the
    /// `work_sel` shape again — and the affordance clicked this frame, which
    /// the shell takes and spends as composer text.
    pub group: science::render::Seat,
}

/// The pre-built view-models + RAM ephemera the inspector renders for one
/// focused agent (§11). Owned: the shell moves each one in — since bl-13f9
/// every one of them is a **wire answer** rather than a disk build (REMOTE
/// §9.7), so no frame reads a file to paint this pane — beside the viewport
/// ephemera (§5.3); nothing borrows the shell's state, so the render holds the
/// whole tab in hand. Absent data (no steps, no governing config) is a value,
/// never a special case — and it is also what a question asked a moment ago
/// honestly holds.
#[derive(Clone)]
pub struct TabData {
    /// Behind an `Arc` (bl-e90a): the shell hands the landed answer on by
    /// pointer, so the pin's cut costs one clone of the entries in front of it
    /// and an unpinned chat costs none.
    pub transcript: std::sync::Arc<Transcript>,
    /// Who the model turns ARE (bl-2335): the conversation's §3.3 display name,
    /// derived by the shell through the same ladder the composer's target line
    /// reads (`root_of` → `display_name_of`) so there is never a second
    /// spelling of it. The model id is a config fact and hovers instead.
    pub speaker: String,
    /// The Raw toggle (§11): one flag, honoured by every tab that *parses* a
    /// file — Transcript, Steps, Inbox — because Raw is the escape from a
    /// parse, and a tab that shows the bytes already (Files) or shows no
    /// file's bytes at all (Config) has nothing to escape from. See
    /// STORIES.md S7 point 3, which this scoping made honest.
    pub raw: bool,
    /// The §11 transcript-density automatics, read from `ui.json` (§4.1) —
    /// which row classes arrive expanded. Durable policy, not ephemera.
    pub auto: AutoExpand,
    pub steps: StepsView,
    /// The selected step's list index (§5.3 ephemera), if any.
    pub step_sel: Option<usize>,
    /// The selected step's drill-in, built by the shell for `step_sel`.
    pub step_detail: Option<StepDetail>,
    pub step_tab: StepTab,
    pub inbox: Vec<InboxEntry>,
    /// The agent worktree's bounded listing (§11 Files tab).
    pub files: FilesView,
    /// The Files tab's selected-file preview, answered beside the listing for
    /// `files_sel`; `None` when nothing is selected.
    pub file_preview: Option<Preview>,
    /// The §11 Work tab's view-model (§5.1 #32, §3.9): every delivery attempt
    /// this workspace holds, **as science reads it** (bl-77bc) — each row
    /// carries the `target..source` diff row plus the agent side the fan group
    /// card compares by, so the listing and the card read one answer.
    pub science: Vec<science::Attempt>,
    /// The Work tab's selected file's patch, built by the shell from
    /// `work_sel`; `None` when nothing is selected.
    pub work_patch: Option<Preview>,
    /// The focused agent's governing config, when derivable (a defective or
    /// unfetched workspace yields `None`, shown as a note). **Pinned** it is
    /// the same derivation asked at the pinned commit instead of the tip
    /// (VISION V1.2 "config-frozen-at"), which is why this rung added no code
    /// to the Config tab at all.
    pub governing: Option<GoverningConfig>,
    /// The step spine for this agent (VISION V1) — its notches, each with the
    /// chat row its rule paints above, and the live child cards hanging off
    /// them. Read by the Transcript tab, which draws it, and by the pin, which
    /// every pinnable tab reads through.
    pub rail: Rail,
    /// The pinned notch, when the operator has picked one. The tab data above
    /// is **already folded to it** — the transcript cut, the files read out of
    /// that commit's tree, the governing config asked at it — so every arm of
    /// [`render`] paints the pin without knowing about it. This field is what
    /// the banner says and what the budget-as-of figure comes from.
    pub pin: Option<Pin>,
}
