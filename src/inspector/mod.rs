//! The §11 Altitude-2 inspector content — the per-agent tabbed pane's
//! **display**, dispatched by [`InspectorTab`].
//!
//! This is the one tested seam between the selected tab and the landed render
//! functions ([`transcript`](crate::transcript), [`steps_view`](crate::steps_view),
//! [`inboxview`](crate::inboxview), and the config-branch governing view). The
//! shell (coverage-excluded) builds a [`TabData`] for the focused agent — the
//! six view-models built fresh each frame and **moved** in — carries the RAM
//! ephemera beside them (the Raw toggle, the step selection/tab, the jsonview
//! collapse set — §5.3), and forwards the digit-key/click events; the dispatch
//! decision itself lives here so every arm is headless shape-walk tested.
//!
//! The Config tab shows only the agent's **governing config** ("policy frozen
//! at `<short-oid>`", §9.3); the three write surfaces (brazen / lernie-global /
//! config-branch editors) are the roster's Config panel (§11 "Config mode …
//! swaps the center"), interaction glue in the shell.

use std::collections::HashSet;

use crate::config_edit::branch::GoverningConfig;
use crate::files_view::{self, FilesView, Preview};
use crate::inboxview::{self, InboxEntry};
use crate::keymap::InspectorTab;
use crate::rail::{Pin, Rail};
use crate::science;
use crate::steps_view::{self, StepDetail, StepTab, StepsView};
use crate::theme;
use crate::transcript::{self, AutoExpand, Transcript};
use crate::workdiff::{self, WorkFile};

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

/// Render the pin banner and the selected tab's content. `eph` is the
/// caller-owned [`Ephemera`] every intrinsic interaction here writes back to —
/// a jsonview disclosure, a Files or Work row click, a transcript row's fold,
/// a notch pin. The return is the child agent a card click asks to open, which
/// the caller retargets to — the same selection gesture as the §11
/// descent-tree rows.
///
/// **The spine lives in the chat and the release lives on the banner**
/// (bl-1802). The pin reaches all four pinnable tabs (VISION V1.2), so its
/// *release* has to be reachable from each of them; the gesture that raises it
/// is a rule in the Transcript tab, so the banner — which already paints above
/// every pinnable tab — carries the way back. That is one existing gesture
/// given a second seat, not a second control: no new verb, no routing, and the
/// banner's own sentence is true wherever it paints.
///
/// `agents` is the frame's roster — the Inbox tab's one use of it is the §3.3
/// ladder over each deposit's sender (bl-b6d0). It is a parameter rather than a
/// [`TabData`] field because that bundle is the *view-models* this paints; the
/// roster is a snapshot fact the shell already holds, and copying it in would
/// be a second per-frame clone of it.
pub fn render(
    ui: &mut egui::Ui,
    tab: InspectorTab,
    data: &TabData,
    titles: &crate::nav::convs::Titles,
    eph: &mut Ephemera,
) -> Option<String> {
    // The banner claims the tab below it, so it paints only where the pin
    // actually reaches: the Work tab reads the project repo, which no
    // conversation commit indexes yet.
    if let Some(pin) = data.pin.as_ref().filter(|_| tab.pinnable()) {
        pinned_banner(ui, pin, &mut eph.notch_sel);
    }
    render_tab(ui, tab, data, titles, eph)
}

/// What a pin is showing, said outright above whichever tab is open, **and the
/// way out of it**: an operator who switched tabs after pinning must not have
/// to infer why the files look old, nor go hunting for the mark they set in
/// another tab. Clicking the banner releases the pin — the same release
/// clicking the pinned rule performs, seated where the pin is visible.
fn pinned_banner(ui: &mut egui::Ui, pin: &Pin, selected: &mut Option<usize>) {
    const RELEASE: &str = "Release the mark and return every tab to now. No key of its own: \
         Tab reaches it, Space presses it.";
    let click = |ui: &mut egui::Ui, text: egui::RichText| {
        ui.add(egui::Label::new(text).sense(egui::Sense::click()))
            .on_hover_text(RELEASE)
            .clicked()
    };
    let hit = ui
        .horizontal(|ui| {
            let mark = click(
                ui,
                egui::RichText::new(format!("as of {}", pin.short)).color(theme::BRAZEN),
            );
            let words = click(
                ui,
                egui::RichText::new(format!(
                    "— every tab below shows this conversation as it stood then; {} tokens spent \
                     by that point. Click here to come back.",
                    pin.tokens
                ))
                .weak(),
            );
            mark || words
        })
        .inner;
    if hit {
        *selected = None;
    }
}

fn render_tab(
    ui: &mut egui::Ui,
    tab: InspectorTab,
    data: &TabData,
    titles: &crate::nav::convs::Titles,
    eph: &mut Ephemera,
) -> Option<String> {
    // Only the Transcript arm answers with a child to open: it is the tab the
    // step spine is drawn through, so it is the only one holding a card to
    // click. Every other arm's answer is the general path with nothing in it.
    match tab {
        InspectorTab::Transcript => transcript::render(
            ui,
            &data.transcript,
            &transcript::Reading {
                speaker: data.speaker.clone(),
                raw: data.raw,
                auto: data.auto,
            },
            &mut eph.tx_folded,
            &data.rail,
            &mut eph.notch_sel,
        ),
        InspectorTab::Steps => {
            steps_view::render(
                ui,
                &data.steps,
                data.step_sel,
                data.step_detail.as_ref(),
                data.step_tab,
                &mut eph.json_collapsed,
                data.raw,
            );
            None
        }
        InspectorTab::Inbox => {
            inboxview::render(ui, &data.inbox, titles, data.raw);
            None
        }
        InspectorTab::Files => {
            files_view::render(
                ui,
                &data.files,
                data.file_preview.as_ref(),
                &mut eph.files_sel,
            );
            None
        }
        InspectorTab::Config => {
            render_config(ui, data.governing.as_ref());
            None
        }
        InspectorTab::Work => {
            // The fan group card first (bl-77bc), over the same answer the
            // attempt rows below drill into — with no fan it paints nothing.
            science::render::group(ui, &data.science, &mut eph.group);
            let diffs: Vec<workdiff::Attempt> =
                data.science.iter().map(|a| a.diff.clone()).collect();
            workdiff::render(ui, &diffs, data.work_patch.as_ref(), &mut eph.work_sel);
            None
        }
    }
}

/// The Config tab (§11): the focused agent's governing config — "policy frozen
/// at `<short-oid>`", the config commit its branch forks off — plus that
/// commit's file listing. The three config *editors* are the roster Config
/// panel (§9), reached by the shell's link.
fn render_config(ui: &mut egui::Ui, governing: Option<&GoverningConfig>) {
    let Some(gov) = governing else {
        ui.weak("no governing config for this agent");
        return;
    };
    ui.strong(gov.frozen_label());
    if let Some(name) = &gov.branch_name_if_tip_of_one {
        ui.weak(format!("tip of config/{name}"));
    }
    egui::ScrollArea::vertical().show(ui, |ui| {
        for file in &gov.files {
            ui.monospace(file);
        }
    });
}

#[cfg(test)]
mod tests;
