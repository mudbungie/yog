//! S1-T4 reply-streams (STORIES.md): the Codex bar's payoff — *the reply
//! streams into the focused view*. A fixture workspace with an **open**
//! `response.json` drives the public transcript view-model
//! ([`yog::transcript::build`] plus `Transcript::with_live`, the same pair the
//! shell's inspector paints), and the live tail appears as a trailing entry,
//! **visually distinct** from the committed messages: its own
//! [`EntryKind::Streaming`] variant (the renderer paints it in
//! `theme::SPECTRE`), folded from the growing `response.json` (§5.1 #10, §11
//! Transcript "live tail … appended, visually distinct").
//!
//! The two halves are separate calls on purpose (§7.2, bl-54f7): the committed
//! read is memoized per published snapshot and the tail moves at frame cadence,
//! so what puts them together is the caller, never `build`.
//!
//! Story-level integration proof: the streaming fold itself is unit-tested in
//! `src/git_tree/streaming.rs` and `src/transcript`, and the *freshness* of the
//! tail in `src/app/live/tests.rs`; this exercises the whole public path against
//! an on-disk fixture — no subprocess, so no fake substrate (Z7) is needed.

// The integration-test crate's fixture helpers are neither `#[test]` fns nor
// `#[cfg(test)]` mods, so `allow-unwrap-in-tests` does not reach them; scoped to
// this test binary and out of the src-only rules-audit (mirrors editor_roundtrip).
#![allow(clippy::unwrap_used)]

use std::path::Path;
use tempfile::tempdir;
use yog::git_tree::{Delta, Stream, stream_from_disk};
use yog::transcript::{self, EntryKind};

const AGENT: &str = "20260719T120000Z-reply";

/// Write one committed delivered message into `<ws>/agents/<AGENT>/messages/`.
fn write_msg(ws: &Path, name: &str, bytes: &[u8]) {
    let dir = ws.join("agents").join(AGENT).join("messages");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join(name), bytes).unwrap();
}

/// Write the latest step's still-open `response.json` — the JSONL the live-tail
/// fold reads (`content_delta` events accrete into the streaming text).
fn write_response(ws: &Path, seq: u32, body: &[u8]) {
    let dir = ws.join("steps").join(AGENT).join(format!("{seq:03}"));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("response.json"), body).unwrap();
}

#[test]
fn open_response_surfaces_a_visually_distinct_live_tail() {
    let dir = tempdir().unwrap();
    let ws = dir.path();
    // A returning conversation: one committed delivered message on disk.
    write_msg(ws, "001-operator.md", b"summarize the design");
    // The latest step is in flight — its response.json is still growing. The
    // model reasoned first and is now answering; both are display text.
    write_response(
        ws,
        1,
        b"{\"type\":\"content_delta\",\"index\":0,\"delta\":{\"thinking_delta\":\"weighing it\"}}\n\
          {\"type\":\"content_delta\",\"index\":0,\"delta\":{\"text_delta\":\"The design \"}}\n\
          {\"type\":\"content_delta\",\"index\":0,\"delta\":{\"text_delta\":\"streams.\"}}\n",
    );

    // The committed transcript is exactly the committed messages: `build` does
    // not read the open step file at all, whatever the agent's state.
    let settled = transcript::build(ws, AGENT);
    assert_eq!(settled.entries.len(), 1, "no live tail from `build` alone");
    assert!(matches!(
        settled.entries[0].kind,
        EntryKind::Delivered { .. }
    ));

    // What the open file says, off the one shared fold.
    let stream = stream_from_disk(ws, AGENT);
    assert_eq!(
        stream,
        Stream {
            text: Some("The design streams.".into()),
            thinking: Some("weighing it".into()),
            last_delta: Some(Delta::Text),
        }
    );

    // In flight: the folded live tail is appended as a trailing entry, distinct
    // from the committed message by its own Streaming kind.
    let live = settled.with_live(&stream);
    assert_eq!(live.entries.len(), 2, "committed message + live tail");
    assert!(matches!(live.entries[0].kind, EntryKind::Delivered { .. }));
    let tail = live.entries.last().unwrap();
    assert!(
        matches!(
            &tail.kind,
            EntryKind::Streaming { thinking, text }
                if text == "The design streams." && thinking == "weighing it"
        ),
        "expected a Streaming live tail folding the growing response, got {:?}",
        tail.kind
    );
    // The raw bytes surfaced under the Raw toggle are the folded live text,
    // reasoning first — the order the rows paint in.
    assert_eq!(tail.raw, b"weighing itThe design streams.");
    // Visually distinct: the tail is the only Streaming entry; committed
    // messages never wear that kind.
    let streaming = live
        .entries
        .iter()
        .filter(|e| matches!(e.kind, EntryKind::Streaming { .. }))
        .count();
    assert_eq!(streaming, 1);
}
