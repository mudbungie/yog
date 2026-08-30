//! **The follow lane's own contract** (bl-73e7): freshness, and the one fold.
//!
//! The assertion that matters is freshness, and it is asserted the way the
//! §7.2 follower's own beats learned to (bl-70b8's catalogue of tests that
//! prove nothing): every beat below appends bytes to an open step file and
//! reads them off a frame with **nothing else having run** — the published
//! derivation is pinned by pointer and never republished, so a frame carrying
//! the new characters can only have come from this read.
//!
//! Nothing here sleeps. [`Follow::poll`] is the mechanism and
//! [`Iterator::next`] is the patience around it, so every beat is driven look
//! by look.
//!
//! How the bytes are *gathered* — the partial line, the truncation, the step
//! boundary, the bound on a quiet hold — is [`reading`], split off at §12's
//! per-file budget on the seam the §7.2 follower's own beats were cut along:
//! *what the tail promises the operator* here, *how the bytes are gathered*
//! there. The fixtures below serve both.

mod reading;

use std::path::Path;

use super::*;
use crate::app::Snapshot;
use crate::boundary::tests::{agent, snapshot};
use crate::git_tree::{AgentState, Delta};
use crate::state::{SnapshotCell, new_snapshot_cell};
use tempfile::TempDir;

pub(super) const AGENT: &str = "c-1";

/// One `content_delta` line of answer text, as brazen's `v=1` writes it.
pub(super) fn text_delta(fragment: &str) -> String {
    format!(
        "{{\"type\":\"content_delta\",\"index\":0,\"delta\":{{\"text_delta\":\"{fragment}\"}}}}\n"
    )
}

/// One `content_delta` line of reasoning.
pub(super) fn thinking_delta(fragment: &str) -> String {
    format!(
        "{{\"type\":\"content_delta\",\"index\":0,\"delta\":{{\"thinking_delta\":\"{fragment}\"}}}}\n"
    )
}

/// The open response file of `agent`'s step `seq` — the literal file litany's
/// harness appends stream events to.
pub(super) fn response(ws: &Path, seq: u32) -> std::path::PathBuf {
    let step = ws.join("steps").join(AGENT).join(format!("{seq:03}"));
    std::fs::create_dir_all(&step).expect("step dir");
    step.join("response.json")
}

/// Append to an open response file, exactly as the harness does mid-call.
pub(super) fn append(path: &Path, bytes: &str) {
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .expect("open response");
    file.write_all(bytes.as_bytes()).expect("append");
}

/// A published snapshot whose one agent wears `state`, and the cell holding it.
pub(super) fn seated(ws: &Path, state: AgentState) -> SnapshotCell {
    new_snapshot_cell(std::sync::Arc::new(snapshot(
        ws,
        "alba",
        vec![agent(AGENT, state, 100)],
        vec![],
    )))
}

/// A workspace, a cell that says the conversation is in flight, and a read of
/// its tail — the ordinary posture of a call the operator is watching.
pub(super) fn flying() -> (TempDir, SnapshotCell, Follow) {
    let dir = tempfile::tempdir().expect("tmp");
    let cell = seated(dir.path(), AgentState::InFlight);
    let follow = Follow::new(
        std::sync::Arc::clone(&cell),
        dir.path().to_path_buf(),
        AGENT.to_owned(),
    );
    (dir, cell, follow)
}

/// The stream a frame carries, or `None` for a look that produced none. Every
/// other reading of [`Frame`] is a beat of its own below.
pub(super) fn said(frame: Frame) -> Option<Stream> {
    match frame {
        Frame::Ready(stream) => Some(stream),
        _ => None,
    }
}

/// **The beat this ball exists for.** Bytes appended to the open step file are
/// on a frame with no derivation between — the published snapshot is the very
/// pointer it was, so nothing but this read can have carried them.
#[test]
pub(super) fn appended_bytes_are_on_a_frame_with_no_derivation_between() {
    let (dir, cell, mut follow) = flying();
    let file = response(dir.path(), 1);
    let pinned = crate::state::latest_snapshot(&cell);

    append(&file, &text_delta("the first "));
    assert_eq!(
        said(follow.poll()).and_then(|s| s.text).as_deref(),
        Some("the first "),
        "the first bytes are a frame"
    );

    append(&file, &text_delta("half."));
    assert_eq!(
        said(follow.poll()).and_then(|s| s.text).as_deref(),
        Some("the first half."),
        "every character that landed is on the next frame"
    );
    assert!(
        std::sync::Arc::ptr_eq(&pinned, &crate::state::latest_snapshot(&cell)),
        "and no derivation ran to put it there — this is the whole claim"
    );
}

/// **The one-moment invariant** (bl-6233): for the same bytes, the follow
/// lane's frame and the pull path's fold are the same value. The lane changes
/// the transport and the cadence; it never changes the fold, so two seats
/// watching one conversation cannot describe one moment differently.
#[test]
fn a_frame_says_exactly_what_the_pull_fold_says_of_the_same_bytes() {
    let (dir, _cell, mut follow) = flying();
    let file = response(dir.path(), 1);
    append(&file, &thinking_delta("first I "));
    append(&file, &text_delta("here goes"));
    let framed = said(follow.poll()).expect("a frame");

    // The derivation's own reading of the same file, published, then folded by
    // the one describer the pull read uses.
    let mut row = agent(AGENT, AgentState::InFlight, 100);
    row.stream = crate::git_tree::stream_from_disk(dir.path(), AGENT);
    let derived: Snapshot = snapshot(dir.path(), "alba", vec![row], vec![]);
    let pulled = crate::boundary::answer::inspector::live_tail(&derived, dir.path(), AGENT)
        .expect("in flight, so there is a tail");

    assert_eq!(framed, pulled, "one fold, two ways of reaching it");
    assert_eq!(framed.thinking.as_deref(), Some("first I "));
    assert_eq!(framed.last_delta, Some(Delta::Text));
}

/// Reasoning streams too, and the same one fold moves `last_delta` — so the
/// §11 live mark flips from thinking to inference off the lane rather than off
/// a sweep.
#[test]
fn reasoning_streams_and_carries_the_doing_split_with_it() {
    let (dir, _cell, mut follow) = flying();
    let file = response(dir.path(), 1);
    append(&file, &thinking_delta("first I "));
    let stream = said(follow.poll()).expect("a frame");
    assert_eq!(stream.thinking.as_deref(), Some("first I "));
    assert_eq!(stream.text, None, "no answer yet, and none invented");
    assert_eq!(stream.last_delta, Some(Delta::Thinking));

    append(&file, &text_delta("here goes"));
    let stream = said(follow.poll()).expect("a frame");
    assert_eq!(stream.text.as_deref(), Some("here goes"));
    assert_eq!(stream.last_delta, Some(Delta::Text));
}
