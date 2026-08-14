//! The focused agent's view-model build — every [`TabData`] field, for the
//! active tab, folded through the operator's pin.
//!
//! Split from [`super`] at §12's per-file budget for the shell, on the seam
//! between *painting the inspector* and *assembling what it paints*.
//! Coverage-excluded like the rest of `shell/*`: each build below is one call
//! into a module that tests it.

use std::path::Path;

use super::super::{InspectorState, ShellState};
use super::{rail, work};
use crate::AppModel;
use crate::boundary::answer::agent::AgentView;
use crate::boundary::answer::inspector;
use crate::files_view::{FilesView, Preview};
use crate::inboxview::{InboxEntry, list_inbox};
use crate::inspector::TabData;
use crate::keymap::InspectorTab;
use crate::steps_view::{self, StepDetail, StepsView};
use crate::transcript::{self, Transcript};

/// Every view-model the focused agent's inspector renders, built for the
/// active tab and folded through the operator's pin. The **rail and the pin
/// come first**: each build below reads through the pin (VISION V1.2 — one
/// mechanism, four tabs), so the pin must be resolved before anything asks
/// disk what to show. A caller with no focused agent never reaches here.
pub(super) fn tab_data(
    active: InspectorTab,
    model: &AppModel,
    state: &mut ShellState,
    ws: &Path,
    focus: &AgentView,
) -> TabData {
    let (agent_id, tip, agent_state) = (&focus.agent_id, &focus.tip, focus.state);
    // Who the transcript's model turns are (bl-2335): the §3.3 ladder over the
    // selection's *conversation root*, through the boundary's own derivation —
    // one function, never a second spelling (bl-6233). It rides the seat's own
    // view since bl-1eb0, so this build reads a payload rather than a roster.
    let speaker = focus.name.clone();
    // The heavy view-models, once per snapshot (§7.2 `SnapMemo`, bl-e90a).
    let steps = state
        .inspector
        .steps_memo
        .read(
            model.derivation(),
            (ws.to_path_buf(), agent_id.clone(), agent_state),
            &mut || steps_view::build(ws, agent_id, agent_state),
        )
        .clone();
    // The transcript is built on **every** tab since bl-1802, not just its own:
    // a notch's seat is a row in the chat, so the spine — and therefore the pin
    // every pinnable tab reads through — is a function of it. One memoized read
    // per snapshot, the same trade the steps view already makes, and one tab
    // arm fewer.
    let live = build_transcript(model, &mut state.inspector, ws, agent_id);
    let history = rail::build(model, &mut state.inspector, ws, agent_id, &steps, &live);
    let pin = rail::pinned(&history, state.inspector.eph.notch_sel);
    let files = build_files(
        active,
        model,
        &mut state.inspector,
        ws,
        agent_id,
        pin.as_ref(),
    );
    // The Work tab's subject is the *project* repo, so it is keyed on the
    // workspace rather than the agent and the pin never reaches it.
    let picked = state.inspector.eph.work_sel.clone();
    let work = work::build(active, model, &mut state.inspector, ws);
    let work_patch = work::patch(model, &mut state.inspector, ws, &work, picked.as_ref());
    TabData {
        transcript: rail::transcript(&live, pin.as_ref()),
        speaker,
        raw: state.inspector.raw,
        auto: model.transcript_auto_expand(),
        step_sel: state.inspector.step_sel,
        step_detail: build_detail(active, ws, agent_id, &steps, state.inspector.step_sel),
        step_tab: state.inspector.step_tab,
        steps,
        inbox: build_inbox(active, ws, agent_id),
        file_preview: build_file_preview(
            active,
            ws,
            agent_id,
            &files,
            state.inspector.eph.files_sel,
            pin.as_ref(),
        ),
        files,
        work,
        work_patch,
        governing: (active == InspectorTab::Config)
            .then(|| rail::governing(ws, tip, pin.as_ref()))
            .flatten(),
        rail: history,
        pin,
    }
}

fn build_files(
    tab: InspectorTab,
    model: &AppModel,
    inspector: &mut crate::shell::InspectorState,
    ws: &Path,
    agent: &str,
    pin: Option<&crate::rail::Pin>,
) -> FilesView {
    if tab == InspectorTab::Files {
        rail::files(model, inspector, ws, agent, pin)
    } else {
        FilesView::default()
    }
}

/// The selected file's preview, when the Files tab is active and the selection
/// points at a file entry (a dir or a stale index yields `None`). Pinned, the
/// bytes come out of that commit's tree instead of the worktree.
fn build_file_preview(
    tab: InspectorTab,
    ws: &Path,
    agent: &str,
    files: &FilesView,
    sel: Option<usize>,
    pin: Option<&crate::rail::Pin>,
) -> Option<Preview> {
    if tab != InspectorTab::Files {
        return None;
    }
    let FilesView::Present { entries, .. } = files else {
        return None;
    };
    let entry = entries.get(sel?)?;
    if entry.is_dir {
        return None;
    }
    Some(rail::preview(ws, agent, &entry.rel_path, pin))
}

/// The transcript: **the committed half once per derivation, the live tail
/// every frame** (§7.2, bl-e90a + bl-54f7).
///
/// The `messages/` read + parse used to run on every frame, which is the cost
/// the operator felt as sluggish, sticky chat scroll — so it is memoized, and
/// the memo is keyed on the derivation ([`AppModel::derivation`]) because that
/// is what tracks the files it read. The **live tail moves on its own clock**
/// and is folded on here from the rendered snapshot's own
/// [`Stream`](crate::git_tree::Stream), which the §7.2 follower keeps fresh: no
/// disk read, so a growing answer costs a clone of the committed entries rather
/// than a re-read of the conversation. Folding the two into one build is what
/// made the tail as slow as the derivation.
///
/// **Which tail, and whether there is one at all, is the boundary's ruling**
/// ([`inspector::live_tail`], bl-6233) — the same fold the headless answer
/// makes, so the two seats cannot describe one moment differently. It is the
/// only half the frame keeps for itself: the committed read is memoized here
/// because a memo is a frame concern, and the answer chokepoint is off-frame
/// and re-reads.
pub(in crate::shell) fn build_transcript(
    model: &AppModel,
    inspector_state: &mut InspectorState,
    ws: &Path,
    agent: &str,
) -> std::sync::Arc<Transcript> {
    let committed = std::sync::Arc::clone(inspector_state.tx_memo.read(
        model.derivation(),
        (ws.to_path_buf(), agent.to_string()),
        &mut || std::sync::Arc::new(transcript::build(ws, agent)),
    ));
    match inspector::live_tail(model.derivation(), ws, agent) {
        Some(stream) => std::sync::Arc::new(committed.with_live(&stream)),
        None => committed,
    }
}

fn build_detail(
    tab: InspectorTab,
    ws: &Path,
    agent: &str,
    steps: &StepsView,
    sel: Option<usize>,
) -> Option<StepDetail> {
    if tab != InspectorTab::Steps {
        return None;
    }
    let seq = &steps.steps.get(sel?)?.seq;
    Some(steps_view::detail(ws, agent, seq))
}

fn build_inbox(tab: InspectorTab, ws: &Path, agent: &str) -> Vec<InboxEntry> {
    if tab == InspectorTab::Inbox {
        list_inbox(ws, agent)
    } else {
        Vec::new()
    }
}
