//! The focused agent's view-model build — every [`TabData`] field, for the
//! active tab, folded through the operator's pin.
//!
//! Split from [`super`] at §12's per-file budget for the shell, on the seam
//! between *painting the inspector* and *assembling what it paints*.
//! Coverage-excluded like the rest of `shell/*`: each field below is one
//! [`reads`] ask and one selection out of what landed.
//!
//! **Nothing here reads disk any more** (REMOTE §9.7, bl-13f9). Every subject
//! this file used to build in process is a standing question now, so what is
//! left is the two things that are genuinely the *seat's*: which tab is open,
//! and the pin. The pin stayed seat-side and §8.5 stayed unamended — a viewport
//! fold gains no boundary representation — because bl-44e9 moved the answers to
//! the altitude that makes it a **selection**: the rail's notches carry their
//! own budget rollups, the pinned transcript is a prefix of the chat that was
//! answered, and the two reads whose subject really is a different *tree*
//! (Files, config-frozen-at) take that tree as a query parameter.

use std::path::Path;

use super::super::ShellState;
use super::super::wire::Said;
use super::{rail, reads, work};
use crate::AppModel;
use crate::inspector::TabData;
use crate::keymap::InspectorTab;
use crate::nav::convs::Selection;
use crate::steps_view::StepsView;

/// Every view-model the focused agent's inspector renders, built for the
/// active tab and folded through the operator's pin. The **rail and the pin
/// come first**: the pinnable tabs are asked at the pin's commit (VISION V1.2 —
/// one mechanism, four tabs), so the pin must be resolved out of the landed
/// spine before their questions are declared. A caller with no focused agent
/// never reaches here.
///
/// The second half of the answer is **the engine's sentences for whatever this
/// frame's questions were refused with** (REMOTE §9.7). They ride beside
/// [`TabData`] rather than in it: `TabData` is what the tested renderer paints,
/// and a refusal is a fact about this seat's transport, which the caller paints
/// above the tab. [`Said`] keeps them distinct, because the whole family
/// refuses in one sentence when what failed was the address.
pub(super) fn tab_data(
    active: InspectorTab,
    model: &mut AppModel,
    state: &mut ShellState,
    ws: &Path,
    focus: &Selection,
) -> (TabData, Vec<String>) {
    let agent_id = focus.agent_id.clone();
    // Who the transcript's model turns are (bl-2335): the §3.3 ladder over the
    // selection's *conversation root*, folded into the seat's own view since
    // bl-1eb0 and picked out of the landed forest since bl-48ae, so this reads
    // an answer rather than a roster.
    let speaker = focus.name.clone();
    // **A conversation with no branch is asked nothing** (§3.4, bl-56c6). The
    // start window is a healthy state, not a fault: the root has no
    // `agents/<id>` ref until the detached driver writes one, so every question
    // below refuses at the address — and the ichor sentences that painted for
    // the whole window told the operator their own new conversation was
    // unknown. What the empty view shows is what the world honestly holds about
    // it, and their text is not lost with it: the §11 queue above the box
    // carries every send (§7.2), faded, from the fire onward.
    if focus.pending {
        return (nothing_yet(state, model, speaker), Vec::new());
    }
    let mut said = Said::default();
    // The two asked on every tab: the steps view because the centre's auth and
    // wound banners read it beside the Steps tab, and the transcript because a
    // notch's seat is a row in the chat — so the spine, and therefore the pin
    // every pinnable tab reads through, is a function of it (bl-1802).
    let steps: StepsView = said
        .take(reads::steps(model, ws, &agent_id))
        .unwrap_or_default();
    let live = std::sync::Arc::new(
        said.take(reads::transcript(model, ws, &agent_id))
            .unwrap_or_default(),
    );
    let history = said
        .take(reads::rail(model, ws, &agent_id))
        .unwrap_or_default();

    let pin = rail::pinned(&history, state.inspector.eph.notch_sel);
    let commit = pin.as_ref().map(|p| p.commit.clone());
    // One question answers the listing and the picked file's bytes together,
    // live or as of the pin — so the seat-side branch the preview used to make
    // dissolves rather than moving (REMOTE §9.7's own residual, closed).
    let files = (active == InspectorTab::Files)
        .then(|| {
            said.take(reads::files(
                model,
                ws,
                &agent_id,
                state.inspector.eph.files_sel.clone(),
                commit.clone(),
            ))
        })
        .flatten()
        .unwrap_or_default();
    let step_detail = detail_seq(active, &steps, state.inspector.step_sel)
        .and_then(|seq| said.take(reads::detail(model, ws, &agent_id, &seq)));
    let inbox = (active == InspectorTab::Inbox)
        .then(|| said.take(reads::inbox(model, ws, &agent_id)))
        .flatten()
        .unwrap_or_default();
    let governing = (active == InspectorTab::Config)
        .then(|| said.take(reads::governing(model, ws, &agent_id, commit)))
        .flatten();
    // The Work tab's subject is the *project* repo, so its question is
    // addressed at the workspace rather than the agent and the pin never
    // reaches it (bl-f297).
    let picked = state.inspector.eph.work_sel.clone();
    let work = work::read(active, model, ws, picked, &mut said);
    let data = TabData {
        transcript: rail::transcript(&live, pin.as_ref()),
        speaker,
        raw: state.inspector.raw,
        auto: model.transcript_auto_expand(),
        step_sel: state.inspector.step_sel,
        step_detail,
        step_tab: state.inspector.step_tab,
        steps,
        inbox,
        file_preview: files.1,
        files: files.0,
        science: work.science,
        work_patch: work.patch,
        governing,
        rail: history,
        pin,
    };
    (data, said.sentences())
}

/// The inspector of a conversation the world does not carry yet (§3.4,
/// bl-56c6): every answer at its empty value, which is what a question nobody
/// asked honestly holds — the same value each arm above already falls back to
/// when its own question has not landed. Only the seat's own ephemera survive,
/// because those are facts about the operator rather than about the branch.
fn nothing_yet(state: &ShellState, model: &AppModel, speaker: String) -> TabData {
    TabData {
        transcript: std::sync::Arc::default(),
        speaker,
        raw: state.inspector.raw,
        auto: model.transcript_auto_expand(),
        step_sel: None,
        step_detail: None,
        step_tab: state.inspector.step_tab,
        steps: StepsView::default(),
        inbox: Vec::new(),
        files: crate::files_view::FilesView::default(),
        file_preview: None,
        science: Vec::new(),
        work_patch: None,
        governing: None,
        rail: crate::rail::Rail::default(),
        pin: None,
    }
}

/// Which step the drill-in is about: the selected row of the list that landed,
/// resolved in the frame that has it. `None` on any other tab, and for a
/// selection the answer no longer carries — a re-derivation that drops steps
/// can never strand the drill-in on a step that is gone.
fn detail_seq(tab: InspectorTab, steps: &StepsView, sel: Option<usize>) -> Option<String> {
    (tab == InspectorTab::Steps)
        .then(|| steps.steps.get(sel?).map(|step| step.seq.clone()))
        .flatten()
}
