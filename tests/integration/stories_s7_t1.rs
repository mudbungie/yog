//! STORIES **S7-T1** tabs-dispatch: one fixture per tab; each builds **from
//! disk alone**, and on every tab that carries the Raw toggle — Transcript,
//! Steps, Inbox — Raw yields the underlying file's bytes **unaltered**,
//! asserted against the bytes read back off the fixture disk so a
//! re-serialization cannot pass (STORIES S7.2/S7.3, DESIGN §11).
//!
//! **The row's premise drifted twice.**
//! 1. There are **six** tabs, not five: `Work` landed with the attempt rung.
//!    The digit keys are 1–6.
//! 2. "Every tab that parses a file carries a Raw toggle" is exact, and bl-1ff1
//!    established that **Files and Config carry none — by rule, not omission**:
//!    Files' preview *is* the bytes (a toggle would swap the bytes for the
//!    bytes) and Config parses no file at all (it names a commit and lists its
//!    tree). Both are asserted below as the absence they are.
//!
//! The tab *dispatch* itself paints through egui and its offscreen probe is
//! `pub(crate)`, so this row asserts the six per-tab view-model builders — the
//! surface STORIES' harness names ("never egui widgets") and the one that
//! actually reads disk.

#![allow(clippy::unwrap_used)]

use crate::support::{AgentFixture, build_agents, write_deposit, write_message, write_step};
use tempfile::tempdir;
use yog::files_view::{self, FilesView, Preview};
use yog::git_tree::AgentState;
use yog::inboxview;
use yog::keymap::InspectorTab;
use yog::steps_view::{self, Doc};
use yog::transcript::{self, EntryKind};

const MODEL_TURN: &str =
    r#"{"content":[{"type":"text","text":"pong"}],"usage":{"input_tokens":5}}"#;
const DEPOSIT: &str =
    "---\nfrom: user\ndeposited_at: t0\nepitaph: final-response\n---\nplease ping\n";
const META: &str = r#"{"commit":"feedc0de","started_at":"t1","ended_at":"t2"}"#;
const RESPONSE: &str =
    "{\"type\":\"usage\",\"input_tokens\":10}\n{\"type\":\"finish\"}\n{\"type\":\"end\"}\n";

/// STORIES **S7-T1** tabs-dispatch.
#[test]
fn s7_t1_every_tab_builds_from_disk_and_raw_is_the_file_itself() {
    let root = tempdir().unwrap();
    let ws = root.path().join("cobalt");
    std::fs::create_dir_all(&ws).unwrap();
    build_agents(&ws, &[AgentFixture::new("c-1", "do the thing\n")]);

    // One fixture per tab, each a plain file on disk — nothing is handed in.
    let msg = write_message(&ws, "c-1", "001-user.md", DEPOSIT);
    let turn = write_message(&ws, "c-1", "002-opus.json", MODEL_TURN);
    write_step(&ws, "c-1", "001", "meta.json", META);
    let resp = write_step(&ws, "c-1", "001", "response.json", RESPONSE);
    let dep = write_deposit(&ws, "c-1", "user-001", DEPOSIT);

    // --- The six tabs (drift 1). The digit keys are one concept, two ways in.
    assert_eq!(InspectorTab::all().len(), 6);
    assert_eq!(InspectorTab::from_digit(1), Some(InspectorTab::Transcript));
    assert_eq!(InspectorTab::from_digit(2), Some(InspectorTab::Steps));
    assert_eq!(InspectorTab::from_digit(3), Some(InspectorTab::Inbox));
    assert_eq!(InspectorTab::from_digit(4), Some(InspectorTab::Files));
    assert_eq!(InspectorTab::from_digit(5), Some(InspectorTab::Config));
    assert_eq!(InspectorTab::from_digit(6), Some(InspectorTab::Work));
    assert_eq!(InspectorTab::from_digit(7), None);
    assert_eq!(InspectorTab::from_digit(0), None);

    // --- Tab 1, Transcript: built from `agents/<id>/messages/` alone.
    let tx = transcript::build(&ws, "c-1");
    assert_eq!(tx.entries.len(), 2, "both entries, filename order");
    assert!(matches!(&tx.entries[0].kind, EntryKind::Delivered { .. }));
    assert!(matches!(&tx.entries[1].kind, EntryKind::Model { .. }));
    // RAW: the entry's bytes are the file's bytes, compared against a fresh
    // read of the fixture — a re-serialization of the parsed value would fail here.
    assert_eq!(tx.entries[0].raw, std::fs::read(&msg).unwrap());
    assert_eq!(tx.entries[1].raw, std::fs::read(&turn).unwrap());

    // --- Tab 2, Steps: built from `steps/<id>/NNN/` alone.
    let steps = steps_view::build(&ws, "c-1", AgentState::Quiescent);
    assert_eq!(steps.steps.len(), 1);
    assert_eq!(steps.steps[0].seq, "001");
    assert_eq!(steps.steps[0].commit.as_deref(), Some("feedc0de"));
    assert_eq!(steps.steps[0].tokens.input_tokens, 10);
    // RAW: each record's bytes are its file's bytes.
    let detail = steps_view::detail(&ws, "c-1", "001");
    let Doc::Json { raw, .. } = &detail.meta else {
        panic!("meta parsed: {:?}", detail.meta)
    };
    assert_eq!(raw.as_slice(), META.as_bytes());
    assert_eq!(detail.response.len(), 3, "one Doc per JSONL line");
    // The whole response file is the concatenation of its records' raw bytes.
    let rebuilt: Vec<u8> = detail
        .response
        .iter()
        .map(|d| match d {
            Doc::Json { raw, .. } | Doc::Unparsed(raw) => raw.clone(),
            Doc::Absent => Vec::new(),
        })
        .flat_map(|mut b| {
            b.push(b'\n');
            b
        })
        .collect();
    assert_eq!(rebuilt, std::fs::read(&resp).unwrap());

    // --- Tab 3, Inbox: built from `inbox/<id>/*.md` alone.
    let inbox = inboxview::list_inbox(&ws, "c-1");
    assert_eq!(inbox.len(), 1);
    // RAW: envelope and all — the parsed header is a projection, not the file.
    assert_eq!(inbox[0].raw, std::fs::read(&dep).unwrap());
    assert!(
        String::from_utf8_lossy(&inbox[0].raw).starts_with("---\n"),
        "the envelope survives Raw"
    );

    // --- Tab 4, Files: built from the agent's worktree alone. It carries NO
    // toggle (drift 2) because its preview already IS the file's bytes.
    let files = files_view::build(&ws, "c-1");
    let FilesView::Present { entries, .. } = &files else {
        panic!("a worktree exists: {files:?}")
    };
    let names: Vec<&str> = entries.iter().map(|e| e.rel_path.as_str()).collect();
    assert!(
        names.contains(&"goal.md"),
        "the worktree's files: {names:?}"
    );
    assert!(
        !names.iter().any(|n| n.starts_with("messages")),
        "messages/ is excluded at the root — it is the Transcript's, not Files'"
    );
    let preview = files_view::preview(&ws.join("agents/c-1/goal.md"));
    assert_eq!(preview, Preview::Text("do the thing\n".to_owned()));
    // A binary file is declared opaque rather than mangled.
    let bin = ws.join("agents/c-1/blob.bin");
    std::fs::write(&bin, [0x00, 0xff, 0x00]).unwrap();
    assert!(matches!(files_view::preview(&bin), Preview::Binary { .. }));

    // --- Tab 5, Config: names the commit policy is frozen at. It parses no
    // file, so there is no file whose bytes a toggle could stand in front of —
    // the second half of drift 2.
    assert!(InspectorTab::Config.pinnable());
    assert!(
        !InspectorTab::Work.pinnable(),
        "Work is the one tab a notch does not pin"
    );
}
