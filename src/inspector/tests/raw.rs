//! S7-T1's second half: the Raw toggle yields each parsing tab's underlying
//! file bytes, unaltered.

use super::{SPEAKER, paint, populated};
use crate::files_view::FilesView;
use crate::git_tree::AgentState;
use crate::inboxview::list_inbox;
use crate::inspector::TabData;
use crate::keymap::InspectorTab;
use crate::rail::Rail;
use crate::steps_view::{self, StepTab};
use crate::transcript::{self, AutoExpand};

/// S7-T1's second half, for every tab that carries the toggle: **the Raw
/// toggle yields the underlying file's bytes unaltered.** Each assertion reads
/// the file off the fixture disk and looks for exactly those bytes in the
/// paint, so a re-serialization (jsonview's tree, the deposit envelope the
/// parse drops) cannot pass. The two tabs without a toggle are the ruling this
/// ball settled: Files already *is* a bytes preview, and Config renders no
/// file's bytes at all (STORIES.md S7 point 3).
#[test]
fn raw_toggle_yields_each_parsing_tabs_underlying_bytes() {
    let (fx, agent, _tip) = populated();
    let ws = &fx.path;
    let read = |rel: &str| String::from_utf8(std::fs::read(ws.join(rel)).unwrap()).unwrap();

    let data = TabData {
        transcript: std::sync::Arc::new(transcript::build(ws, &agent)),
        speaker: SPEAKER.to_string(),
        raw: true,
        auto: AutoExpand::default(),
        steps: steps_view::build(ws, &agent, AgentState::Quiescent),
        step_sel: Some(0),
        step_detail: Some(steps_view::detail(ws, &agent, "001")),
        step_tab: StepTab::Tools,
        inbox: list_inbox(ws, &agent),
        files: FilesView::default(),
        file_preview: None,
        work: Vec::new(),
        work_patch: None,
        governing: None,
        rail: Rail::default(),
        pin: None,
    };

    // Transcript — the tool message's bytes, which the parsed view renders as
    // a result row and never as the JSON that carried it.
    let tx = paint(InspectorTab::Transcript, &data);
    let tool_msg = read("agents/c-1/messages/003-tool.json");
    assert!(tx.contains(&tool_msg), "transcript raw:\n{tx}");

    // Steps — the selected step's tool input/output records, verbatim.
    let st = paint(InspectorTab::Steps, &data);
    for rel in [
        "steps/c-1/001/tools/toolu_1/input.json",
        "steps/c-1/001/tools/toolu_1/output.json",
    ] {
        assert!(st.contains(&read(rel)), "steps raw missing {rel}:\n{st}");
    }

    // Inbox — the deposit file's bytes, envelope and all; the parsed header the
    // envelope becomes is gone, which is what "instead of" means.
    let ib = paint(InspectorTab::Inbox, &data);
    assert!(
        ib.contains(&read("inbox/c-1/user-001.md")),
        "inbox raw:\n{ib}"
    );
    assert!(!ib.contains("✉ user · t0"), "parsed header in raw:\n{ib}");
}
