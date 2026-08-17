//! Enumeration order, skipping, the in-progress query, and the live tail.

use std::collections::HashSet;

use super::{AGENT, write_msg, write_response};
use crate::git_tree::Stream;
use crate::transcript::{AutoExpand, EntryKind, Tone, Transcript, build, rows};
use tempfile::tempdir;

#[test]
fn entries_are_ordered_by_filename_counter() {
    let dir = tempdir().unwrap();
    for n in ["003-c.md", "001-a.md", "002-b.md"] {
        write_msg(dir.path(), n, b"x");
    }
    let t = build(dir.path(), AGENT);
    let names: Vec<&str> = t.entries.iter().map(|e| e.name.as_str()).collect();
    assert_eq!(names, ["001-a.md", "002-b.md", "003-c.md"]);
}

#[test]
fn non_file_entries_are_skipped() {
    let dir = tempdir().unwrap();
    write_msg(dir.path(), "001-a.md", b"x");
    std::fs::create_dir_all(
        dir.path()
            .join("agents")
            .join(AGENT)
            .join("messages")
            .join("subdir"),
    )
    .unwrap();
    assert_eq!(build(dir.path(), AGENT).entries.len(), 1);
}

#[test]
fn absent_messages_dir_yields_empty_transcript() {
    let dir = tempdir().unwrap();
    assert!(build(dir.path(), AGENT).entries.is_empty());
}

#[test]
fn tool_in_progress_true_until_result_committed() {
    let dir = tempdir().unwrap();
    write_msg(
        dir.path(),
        "001-m.json",
        br#"[{"type":"tool_use","id":"t1","name":"N","input":{}}]"#,
    );
    assert!(build(dir.path(), AGENT).tool_in_progress("t1"));
    write_msg(
        dir.path(),
        "002-tool.json",
        br#"{"tool_use_id":"t1","content":"ok"}"#,
    );
    assert!(!build(dir.path(), AGENT).tool_in_progress("t1"));
}

/// Is any row pulsing (a tool call with no result)?
fn pulsing(t: &Transcript) -> bool {
    rows(
        t,
        super::rows::SPEAKER,
        AutoExpand::default(),
        &HashSet::new(),
    )
    .iter()
    .any(|r| r.tone == Tone::InFlight)
}

/// The operator's stuck `⚙ bash — running` (bl-47ec), byte-for-byte off disk:
/// lernie commits a **bare array** of canonical blocks, and the id is
/// OpenAI-shaped (`call_…`). The result must classify as a `ToolResult` — a
/// Raw-bucket entry names no id, so it retires nothing — and the call's row
/// must then carry neither the word nor the pulse.
#[test]
fn array_shaped_result_with_opaque_id_retires_the_pulse() {
    let dir = tempdir().unwrap();
    write_msg(
        dir.path(),
        "020-gpt-5.4.json",
        br#"[{"type":"tool_use","id":"call_QxWh5oDZm5GNM4nbnIFVb7Ou","name":"bash","input":{"command":"pwd"}}]"#,
    );
    assert!(pulsing(&build(dir.path(), AGENT)), "no result yet");

    write_msg(
        dir.path(),
        "021-tool.json",
        br#"[{"type":"tool_result","tool_use_id":"call_QxWh5oDZm5GNM4nbnIFVb7Ou","content":[{"type":"text","text":"/ops\n"}],"is_error":false}]"#,
    );
    let t = build(dir.path(), AGENT);
    // The listing opens at `020`, so it also carries the compaction mark for
    // the counter values below it (bl-7bd2) — the result is the last entry.
    assert!(matches!(
        &t.entries.last().unwrap().kind,
        EntryKind::ToolResult { tool_use_id, content, is_error: false }
            if tool_use_id == "call_QxWh5oDZm5GNM4nbnIFVb7Ou" && content == "/ops\n"
    ));
    assert!(!pulsing(&t), "a committed result retires the running badge");
    let call = rows(
        &t,
        super::rows::SPEAKER,
        AutoExpand::default(),
        &HashSet::new(),
    );
    // Ahead of it sits the mark for the counter values `020` opens above.
    assert!(
        call.iter().any(|r| r.prefix == "⚙ bash"),
        "got: {:?}",
        call.iter().map(|r| r.prefix.clone()).collect::<Vec<_>>()
    );
}

#[test]
fn build_reads_only_the_committed_messages() {
    let dir = tempdir().unwrap();
    write_msg(dir.path(), "001-a.md", b"hi");
    write_response(
        dir.path(),
        1,
        b"{\"type\":\"content_delta\",\"index\":0,\"delta\":{\"text_delta\":\"streaming\"}}\n",
    );
    // The open response file is on disk and `build` does not touch it: the
    // live tail is the caller's fold, on the caller's clock (§7.2).
    assert_eq!(build(dir.path(), AGENT).entries.len(), 1);
}

#[test]
fn with_live_appends_the_stream_as_a_trailing_entry() {
    let dir = tempdir().unwrap();
    write_msg(dir.path(), "001-a.md", b"hi");
    let t = build(dir.path(), AGENT).with_live(&Stream {
        text: Some("streaming".into()),
        thinking: Some("pondering".into()),
        last_delta: Some(crate::git_tree::Delta::Text),
    });
    assert_eq!(t.entries.len(), 2);
    assert!(matches!(
        &t.entries[1].kind,
        EntryKind::Streaming { thinking, text } if text == "streaming" && thinking == "pondering"
    ));
    // The Raw toggle shows what was said, reasoning first — the same order the
    // rows paint in and the same order the committed blocks will land in.
    assert_eq!(t.entries[1].raw, b"ponderingstreaming");
}

#[test]
fn a_stream_that_has_said_nothing_appends_nothing() {
    let dir = tempdir().unwrap();
    write_msg(dir.path(), "001-a.md", b"hi");
    // The model call is open but no delta has landed: the §11 live mark says
    // "waiting", and a blank live row would be a second, worse spelling of it.
    let t = build(dir.path(), AGENT).with_live(&Stream::default());
    assert_eq!(t.entries.len(), 1);
}
