//! **What a hold promises the operator** (bl-58bb): the lane's fourth subject,
//! beside what the tail promises ([`super`]), how the bytes are gathered
//! ([`super::reading`]) and what the window says ([`super::window`]).
//!
//! A conversation whose next act the capability control parked was reported by
//! this lane as at rest, and the stream ended — at the exact moment the
//! operator holding the read was the thing it was waiting for. The fact existed
//! and two other reads carried it; the live view was the one that dropped it.

use super::*;
use crate::control::hold::Held;

/// The park litany's seam writes when the control answers `hold`.
fn held() -> Held {
    Held {
        tool_use_id: "toolu_02".to_owned(),
        tool: "box2_service_status".to_owned(),
        reason: "box2_service_status {} classified opaque (its input carries no command line)"
            .to_owned(),
    }
}

/// Republish the workspace with `agent` at `state`, wearing `mark`.
fn publish(cell: &SnapshotCell, ws: &Path, state: AgentState, mark: Option<Held>) {
    let mut row = agent(AGENT, state, 100);
    row.held = mark;
    crate::state::publish_snapshot(
        cell,
        std::sync::Arc::new(snapshot(ws, "alba", vec![row], vec![])),
    );
}

/// **The beat this ball exists for.** The control parks the step's next call:
/// the branch stops driving, so every liveness reading says quiescent — and
/// this lane used to answer that by ending the stream, silently. It now names
/// the park, with the three fields the agent row already carries, and holds:
/// the step has not committed, and what it is waiting for is the operator
/// reading this.
#[test]
fn a_park_is_a_frame_and_never_the_end_of_the_stream() {
    let (dir, cell, mut follow) = flying();
    let file = response(dir.path(), 1);
    append(&file, &text_delta("Checking box2."));
    assert!(
        matches!(follow.poll(), Frame::Ready(..)),
        "the call streams"
    );

    publish(&cell, dir.path(), AgentState::Quiescent, Some(held()));
    let parked = ran(follow.poll()).expect("the park is a frame");
    assert_eq!(parked.len(), 1, "{parked:?}");
    assert_eq!(parked[0].tool_use, "toolu_02");
    assert_eq!(parked[0].name.as_deref(), Some("box2_service_status"));
    assert!(
        parked[0]
            .held
            .as_deref()
            .is_some_and(|why| why.contains("classified opaque")),
        "and why it was stopped: {:?}",
        parked[0].held
    );
    assert_eq!(parked[0].input, None, "nothing was dispatched");
    assert_eq!(parked[0].exit_code, None, "so nothing came back");

    assert!(
        matches!(follow.poll(), Frame::Waiting),
        "a hold is not rest: the stream stays open, and the park is said once"
    );
}

/// A read that arrives *after* the park is answered it on its first look — the
/// operator who opens the lane because the conversation went quiet is the one
/// this matters most to, and the read's watermark starts empty for exactly this
/// reason.
#[test]
fn a_read_opened_on_an_already_parked_conversation_is_answered_it() {
    let dir = tempfile::tempdir().expect("tmp");
    let cell = seated(dir.path(), AgentState::Quiescent);
    publish(&cell, dir.path(), AgentState::Quiescent, Some(held()));
    let mut follow = Follow::new(
        std::sync::Arc::clone(&cell),
        dir.path().to_path_buf(),
        AGENT.to_owned(),
    );
    append(&response(dir.path(), 1), "");

    let parked = ran(follow.poll()).expect("a frame on the first look");
    assert_eq!(parked.len(), 1, "{parked:?}");
    assert_eq!(parked[0].tool_use, "toolu_02");
}

/// **A park is not a third point on the posted→captured ladder.** When the
/// operator answers it the call runs, and its opening and closing arrive under
/// the same `tool_use` id — so a follower keyed on that id reads one call going
/// held → posted → complete, and never two calls.
#[test]
fn an_answered_park_runs_the_same_call_under_the_same_id() {
    let (dir, cell, mut follow) = flying();
    let file = response(dir.path(), 1);
    append(&file, "");
    let step = file.parent().expect("step dir").to_path_buf();
    publish(&cell, dir.path(), AgentState::Quiescent, Some(held()));
    assert_eq!(ran(follow.poll()).map(|t| t.len()), Some(1), "parked");

    // `/answer pass` lifts the mark and the seam dispatches the call.
    publish(&cell, dir.path(), AgentState::Live, None);
    let dir_02 = step.join("tools").join("toolu_02");
    std::fs::create_dir_all(&dir_02).expect("call dir");
    std::fs::write(
        dir_02.join("input.json"),
        r#"{"id":"toolu_02","name":"box2_service_status","input":{}}"#,
    )
    .expect("input");

    let opened = ran(follow.poll()).expect("the call it held now runs");
    assert_eq!(opened.len(), 1, "{opened:?}");
    assert_eq!(opened[0].tool_use, "toolu_02", "the same call");
    assert_eq!(opened[0].held, None, "no longer parked");
    assert!(opened[0].input.is_some(), "and dispatched: {opened:?}");
}

/// Rest is still rest: a quiescent conversation with nothing parked ends the
/// stream exactly as it always did. The hold is a distinct kind of stop, not a
/// new reading of the old one.
#[test]
fn a_quiescent_conversation_with_no_park_is_still_the_end_of_the_stream() {
    let (dir, cell, mut follow) = flying();
    append(&response(dir.path(), 1), &text_delta("done."));
    assert!(matches!(follow.poll(), Frame::Ready(..)));
    publish(&cell, dir.path(), AgentState::Quiescent, None);
    assert!(matches!(follow.poll(), Frame::Over));
}

/// The one-shot answer describes the same moment: an intake that cannot hold a
/// connection is handed the park too, or the two forms of one lane would
/// disagree about the one state an operator must not miss.
#[test]
fn the_one_shot_answer_carries_the_park() {
    let dir = tempfile::tempdir().expect("tmp");
    let cell = seated(dir.path(), AgentState::Quiescent);
    publish(&cell, dir.path(), AgentState::Quiescent, Some(held()));
    append(&response(dir.path(), 1), "");
    let snap = crate::state::latest_snapshot(&cell);
    let frame = super::super::once(&snap, dir.path(), AGENT);
    assert_eq!(frame.tools.len(), 1, "{:?}", frame.tools);
    assert_eq!(frame.tools[0].tool_use, "toolu_02");
    assert!(frame.tools[0].held.is_some());
}
