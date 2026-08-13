//! Every tab paints its signature content from one populated fixture (§11):
//! all five [`InspectorTab`] values cycled over one build.

use super::{SPEAKER, paint, populated};
use crate::config_edit::branch::governing_config;
use crate::files_view;
use crate::git_tree::GitTree;
use crate::inboxview::list_inbox;
use crate::inspector::TabData;
use crate::keymap::InspectorTab;
use crate::steps_view::{self, StepTab};
use crate::transcript::{self, AutoExpand};

#[test]
fn every_tab_paints_its_signature_from_a_populated_fixture() {
    let (fx, agent, tip) = populated();
    let ws = &fx.path;
    let state = GitTree::from_repo(ws)
        .unwrap()
        .agents
        .iter()
        .find(|a| a.agent_id == agent)
        .unwrap()
        .state;
    let transcript = std::sync::Arc::new(transcript::build(ws, &agent));
    let steps = steps_view::build(ws, &agent, state);
    let detail = steps_view::detail(ws, &agent, "001");
    let inbox = list_inbox(ws, &agent);
    let files = files_view::build(ws, &agent);
    // Select goal.md (a work product); its body is the fixture message "hello".
    let preview = files_view::preview(&files_view::agent_worktree(ws, &agent).join("goal.md"));
    let governing = governing_config(ws, &tip).unwrap();
    let governing_short = governing.short_oid.clone();

    // The step spine, built the way the shell builds it — the chat is where the
    // rules live now, so a default spine would leave the Transcript tab with
    // no boundary rule to paint (bl-1802).
    let rail = crate::rail::build("root", &[], &steps, &transcript, &[]);

    let data = TabData {
        transcript,
        speaker: SPEAKER.to_string(),
        raw: false,
        auto: AutoExpand::default(),
        steps,
        step_sel: Some(0),
        step_detail: Some(detail),
        step_tab: StepTab::Tools,
        inbox,
        files,
        file_preview: Some(preview),
        work: Vec::new(),
        work_patch: None,
        governing: Some(governing),
        rail,
        pin: None,
    };

    // 1 — Transcript: the delivered body, the model reply, the tool result —
    // and the boundary rule above the crossing, carrying step 001's meta
    // commit in short-oid form (§11 "Every crossing leaves its line").
    let tx = paint(InspectorTab::Transcript, &data);
    assert!(tx.contains("please ping"), "transcript:\n{tx}");
    assert!(tx.contains("pong reply"));
    assert!(tx.contains("tool said hi"));
    assert!(tx.contains("feedc0d"), "boundary rule id:\n{tx}");
    // Raw toggle flips the whole tab to verbatim bytes.
    let raw = paint(
        InspectorTab::Transcript,
        &TabData {
            raw: true,
            ..data.clone()
        },
    );
    assert!(raw.contains("001-user.md"), "raw:\n{raw}");

    // 2 — Steps: the headed step row (tokens folded) and the tool drill-in i/o.
    let st = paint(InspectorTab::Steps, &data);
    assert!(st.contains("Tokens"), "steps column heading:\n{st}");
    assert!(st.contains("15"), "steps:\n{st}");
    assert!(st.contains("toolu_1"));
    assert!(st.contains("input"));
    assert!(st.contains("output"));

    // 3 — Inbox: the deposit header and body.
    let ib = paint(InspectorTab::Inbox, &data);
    assert!(ib.contains("✉ user · t0"), "inbox:\n{ib}");
    assert!(ib.contains("follow-up message"));

    // 4 — Files: listing + selected-file preview reach paint; root messages/ hidden.
    let fl = paint(InspectorTab::Files, &data);
    assert!(fl.contains("goal.md"), "files tree:\n{fl}");
    assert!(fl.contains("hello"), "goal.md preview reached paint:\n{fl}");
    assert!(!fl.contains("messages"), "root messages/ excluded:\n{fl}");

    // 5 — Config: the governing frozen label and its file listing.
    let cfg = paint(InspectorTab::Config, &data);
    assert!(cfg.contains("policy frozen at"), "config:\n{cfg}");
    assert!(cfg.contains(&governing_short));
    assert!(cfg.contains("version"), "governing files listed:\n{cfg}");
}
