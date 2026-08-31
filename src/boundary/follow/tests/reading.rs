//! How the bytes are gathered: partial writes, the resets, the step boundary
//! and the bound on a quiet hold.
//!
//! Split from [`super`] at §12's per-file budget; the fixtures are the
//! parent's.

use std::time::Duration;

use super::super::*;
use super::{AGENT, append, flying, response, said, seated, text_delta};
use crate::boundary::tests::{agent, snapshot};
use crate::git_tree::AgentState;

/// **A step dir with no response file yet is a hold, not an answer.** The
/// driver opens its step before the model says anything, so this is the
/// ordinary first look of every call — and reading it as an end would close
/// the stream a moment before it began.
#[test]
fn a_step_that_has_opened_no_response_file_is_waited_on() {
    let (dir, _cell, mut follow) = flying();
    let file = response(dir.path(), 1);
    assert!(!file.exists(), "the step dir is there and the file is not");
    assert!(matches!(follow.poll(), Frame::Waiting));
    append(&file, &text_delta("and then it speaks"));
    assert!(matches!(follow.poll(), Frame::Ready(_)));
}

/// A half-written line waits for its newline — partial-write tolerance at the
/// place the bytes are gathered, so no frame ever carries half a JSON object.
#[test]
fn a_half_written_line_waits_for_its_newline() {
    let (dir, _cell, mut follow) = flying();
    let file = response(dir.path(), 1);
    let whole = text_delta("atomic");
    let (head, tail) = whole.split_at(whole.len() / 2);
    append(&file, head);
    assert!(said(follow.poll()).is_none(), "nothing whole arrived");
    append(&file, tail);
    assert_eq!(
        said(follow.poll()).and_then(|s| s.text).as_deref(),
        Some("atomic")
    );
}

/// **Bytes moving is not the tail moving.** A `message_start` advances the
/// offset and says nothing an operator can see, so it earns no frame — which
/// is what keeps a quiet call from waking a face sixty times a second.
#[test]
fn an_event_the_operator_cannot_see_is_not_a_frame() {
    let (dir, _cell, mut follow) = flying();
    let file = response(dir.path(), 1);
    append(&file, "{\"type\":\"message_start\"}\n");
    assert!(matches!(follow.poll(), Frame::Waiting), "nothing to say");
    append(&file, &text_delta("now something"));
    assert!(matches!(follow.poll(), Frame::Ready(_)));
    assert!(
        matches!(follow.poll(), Frame::Waiting),
        "and an unchanged file is not news either"
    );
}

/// A file that shrank was truncated or replaced, so the read restarts from
/// zero rather than reading a suffix of bytes that are no longer there.
#[test]
fn a_file_that_shrank_is_read_again_from_the_start() {
    let (dir, _cell, mut follow) = flying();
    let file = response(dir.path(), 1);
    append(&file, &text_delta("the long first answer"));
    assert!(matches!(follow.poll(), Frame::Ready(_)));
    std::fs::write(&file, text_delta("short")).expect("truncate");
    assert_eq!(
        said(follow.poll()).and_then(|s| s.text).as_deref(),
        Some("the long first answershort"),
        "the accumulated text is now wrong by the bytes that vanished, and the \
         next derivation is what corrects it — the follower's own ruling, kept"
    );
}

/// **A response file with nothing in flight is not opened.** It is the last
/// step's settled answer, which the committed transcript already carries, so
/// opening it would paint the answer twice.
#[test]
fn a_settled_step_is_not_followed() {
    let dir = tempfile::tempdir().expect("tmp");
    let cell = seated(dir.path(), AgentState::Quiescent);
    append(&response(dir.path(), 1), &text_delta("already committed"));
    let mut follow = Follow::new(cell, dir.path().to_path_buf(), AGENT.to_owned());
    assert!(
        matches!(follow.poll(), Frame::Waiting),
        "a hold, not an answer — and never a frame"
    );
}

/// **The stream closes at the step boundary**, and the final bytes come out
/// first: a step that committed between two looks still wrote what it wrote.
/// The terminator is what makes the seat's swap to the committed entry a swap
/// rather than a duplication.
#[test]
fn the_stream_ends_when_the_call_does_and_the_last_bytes_come_out_first() {
    let (dir, cell, mut follow) = flying();
    let file = response(dir.path(), 1);
    append(&file, &text_delta("the model begins"));
    assert!(
        matches!(follow.poll(), Frame::Ready(_)),
        "the stream is open"
    );
    // The last characters land, and the driver finishes: the derivation
    // republishes with the call settled.
    append(&file, &text_delta(" and ends"));
    crate::state::publish_snapshot(
        &cell,
        std::sync::Arc::new(snapshot(
            dir.path(),
            "alba",
            vec![agent(AGENT, AgentState::Quiescent, 100)],
            vec![],
        )),
    );
    assert_eq!(
        said(follow.poll()).and_then(|s| s.text).as_deref(),
        Some("the model begins and ends"),
        "nothing written is dropped by the close"
    );
    assert!(matches!(follow.poll(), Frame::Over), "then the terminator");
}

/// A step advancing is **this stream ending**, not an accumulator to reset: a
/// response file belongs to one step, and the seat re-asks for the next one's.
#[test]
fn a_step_advancing_ends_the_stream() {
    let (dir, _cell, mut follow) = flying();
    append(&response(dir.path(), 1), &text_delta("step one"));
    assert!(matches!(follow.poll(), Frame::Ready(_)));
    append(&response(dir.path(), 2), &text_delta("step two"));
    assert!(matches!(follow.poll(), Frame::Over));
}

/// The tree going away ends it on the same terms — a conversation deleted
/// under a held read is a stream that stopped, said in the one word the
/// framing has for it.
#[test]
fn a_tree_that_went_away_ends_the_stream() {
    let (dir, _cell, mut follow) = flying();
    append(&response(dir.path(), 1), &text_delta("here"));
    assert!(matches!(follow.poll(), Frame::Ready(_)));
    std::fs::remove_dir_all(dir.path().join("steps")).expect("delete the steps");
    assert!(matches!(follow.poll(), Frame::Over));
}

/// **The hold is bounded, and a frame resets the bound.** A quiet read ends
/// after its stated waits so a peer that went away costs a thread for that long
/// and no longer; a read that is producing frames proves its peer with every
/// write, so its hold never expires.
#[test]
fn a_quiet_hold_expires_and_a_frame_resets_it() {
    let (dir, cell, _drop) = flying();
    let ws = dir.path().to_path_buf();
    let mut quiet = Follow::holding(
        std::sync::Arc::clone(&cell),
        ws.clone(),
        AGENT.to_owned(),
        2,
        Duration::ZERO,
    );
    assert!(
        quiet.next().is_none(),
        "two quiet looks and the hold is over"
    );

    let mut talking = Follow::holding(cell, ws.clone(), AGENT.to_owned(), 2, Duration::ZERO);
    let file = response(&ws, 1);
    append(&file, &text_delta("one"));
    assert!(talking.next().is_some());
    append(&file, &text_delta(" two"));
    assert!(
        talking.next().is_some(),
        "the quiet look before this one did not count against a hold a frame reset"
    );
}

/// **A stream that is over ends the hold, and says so by ending** — the
/// iterator's third arm. A step that advanced (or a tree that went away) is
/// not a quiet look to be waited out: there is nothing more to say about the
/// call this connection was following, so the frames stop rather than the
/// hold expiring on patience it never needed.
#[test]
fn a_stream_that_is_over_ends_the_hold_at_once() {
    let (dir, cell, _drop) = flying();
    let ws = dir.path().to_path_buf();
    let file = response(&ws, 1);
    let mut held = Follow::holding(
        cell,
        ws.clone(),
        AGENT.to_owned(),
        // A hold so patient that expiry cannot be what ends it: if the frames
        // stop, it is because the stream did.
        u32::MAX,
        Duration::ZERO,
    );
    append(&file, &text_delta("one"));
    assert!(held.next().is_some(), "the open step's bytes");

    // The step advances: a new response file, so the stream this hold opened
    // is over.
    let next_step = response(&ws, 2);
    append(&next_step, &text_delta("two"));
    assert!(
        held.next().is_none(),
        "the stream is over, and the hold ends with it"
    );
}
