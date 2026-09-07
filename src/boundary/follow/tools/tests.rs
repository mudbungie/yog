//! What a read owes the seat, look by look (bl-5305): each transition once,
//! and never a restatement.

use super::{Window, whole};
use serde_json::json;
use std::path::{Path, PathBuf};

fn call(step: &Path, id: &str, name: &str) -> PathBuf {
    let dir = step.join("tools").join(id);
    std::fs::create_dir_all(&dir).expect("call dir");
    std::fs::write(
        dir.join("input.json"),
        serde_json::to_vec(&json!({"id": id, "name": name,
                                   "input": {"command": "apk add curl"}}))
        .expect("input"),
    )
    .expect("write input");
    dir
}

fn capture(dir: &Path, exit_code: i32) {
    std::fs::write(
        dir.join("output.json"),
        serde_json::to_vec(&json!({"stdout": "", "stderr": "", "exit_code": exit_code,
                                   "started_at": "t0", "ended_at": "t1"}))
        .expect("output"),
    )
    .expect("write output");
}

/// **The beat this ball exists for.** A call posted between two looks is on the
/// next one, and its capture is on the look after that — the tool window
/// opening and closing, live, which is what the lane carried nothing of.
#[test]
fn a_call_opens_on_one_look_and_closes_on_the_next() {
    let step = tempfile::tempdir().expect("tmp");
    let mut window = Window::default();
    assert!(window.look(step.path()).is_empty(), "nothing has run");

    let dir = call(step.path(), "toolu_01", "box2_Bash");
    let opened = window.look(step.path());
    assert_eq!(opened.len(), 1, "the call is on this look");
    assert_eq!(opened[0].name.as_deref(), Some("box2_Bash"));
    assert_eq!(opened[0].exit_code, None, "and it is still running");

    assert!(
        window.look(step.path()).is_empty(),
        "a look with nothing new says nothing — a frame is an append"
    );

    capture(&dir, 0);
    let closed = window.look(step.path());
    assert_eq!(closed.len(), 1, "the capture is on the next look");
    assert_eq!(closed[0].exit_code, Some(0));
    assert_eq!(closed[0].tool_use, "toolu_01", "on the same identity");

    assert!(
        window.look(step.path()).is_empty(),
        "and a closed call never speaks again"
    );
}

/// A look slower than the tool still says both halves, in order — the ordinary
/// case for anything that finishes in milliseconds, and the one where reporting
/// only the opening would leave a row an operator cannot act on.
#[test]
fn a_call_that_finished_between_two_looks_yields_both_halves_at_once() {
    let step = tempfile::tempdir().expect("tmp");
    let dir = call(step.path(), "toolu_01", "Read");
    capture(&dir, 2);
    let events = Window::default().look(step.path());
    assert_eq!(events.len(), 2, "opened and closed: {events:?}");
    assert_eq!(events[0].name.as_deref(), Some("Read"));
    assert_eq!(events[0].exit_code, None);
    assert_eq!(events[1].exit_code, Some(2));
}

/// Several calls in one step are each tracked on their own identity: one
/// finishing does not retire another's opening, and one still running does not
/// hold back another's capture.
#[test]
fn calls_are_tracked_one_watermark_each() {
    let step = tempfile::tempdir().expect("tmp");
    let mut window = Window::default();
    let first = call(step.path(), "toolu_01", "Bash");
    call(step.path(), "toolu_02", "Bash");
    assert_eq!(window.look(step.path()).len(), 2, "both opened");

    capture(&first, 0);
    let events = window.look(step.path());
    assert_eq!(events.len(), 1, "only the one that finished: {events:?}");
    assert_eq!(events[0].tool_use, "toolu_01");
}

/// The one-shot answer — the intake that cannot hold a connection — is the same
/// concatenation taken in one look, so a seat that absorbs frames in order and
/// a seat that pulled once hold the same list.
#[test]
fn the_one_shot_answer_is_the_held_read_s_frames_concatenated() {
    let step = tempfile::tempdir().expect("tmp");
    let dir = call(step.path(), "toolu_01", "Bash");
    let mut window = Window::default();
    let mut held = window.look(step.path());
    capture(&dir, 7);
    held.extend(window.look(step.path()));
    assert_eq!(
        held,
        whole(step.path(), None),
        "one rule, not two spellings"
    );
}
