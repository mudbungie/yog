//! **The attention lane's own contract** (REMOTE §14.1, bl-09aa): what makes a
//! frame, what does not, and what the hold costs a seat that never hears one.
//!
//! Nothing here sleeps or waits on a thread. [`Attend::look`] is the mechanism
//! and [`Iterator::next`] is the patience around it, so every beat below is
//! driven look by look — the follow lane's own discipline, and the reason a
//! lane whose production bound is thirty seconds is tested in microseconds.

use super::*;
use crate::boundary::tests::{agent, snapshot};
use crate::git_tree::{Agent, AgentState};
use crate::state::new_snapshot_cell;
use crate::test_support::FakeClock;
use std::time::Duration;
use tempfile::TempDir;

/// The workspace every beat asks about — a **leaf**, because that is what a
/// registration names and what [`Snapshot::scoped`] filters on.
const WS: &str = "alba";

/// One conversation that is asking for the operator (§6 rule 1): an unseen
/// notify oid, which is the cheapest of the seven signals to state.
fn asking(id: &str) -> Agent {
    Agent {
        notify_oid: Some("n".repeat(40)),
        ..agent(id, AgentState::Quiescent, 0)
    }
}

/// The world holding `agents` in the one workspace, published.
fn published(dir: &TempDir, agents: Vec<Agent>) -> Arc<Snapshot> {
    Arc::new(snapshot(&dir.path().join(WS), WS, agents, vec![]))
}

/// A lane over a world with `agents` in it, scoped to `scope`, plus the cell
/// the worker republishes into and the clock the ages are read off.
fn lane(agents: Vec<Agent>, scope: &[&str]) -> (TempDir, SnapshotCell, FakeClock, Attend) {
    let dir = tempfile::tempdir().expect("tmp");
    let cell = new_snapshot_cell(published(&dir, agents));
    let clock = FakeClock::new();
    let attend = Attend::new(
        Arc::clone(&cell),
        scope.iter().map(|s| (*s).to_owned()).collect(),
        dir.path().join("ui.json"),
        clock.arc(),
    );
    (dir, cell, clock, attend)
}

/// The conversations one frame names, in frame order.
fn named(rows: &[QueueRow]) -> Vec<String> {
    rows.iter().map(|row| row.agent.clone()).collect()
}

/// **The first frame is the answer as of connect, and it is unconditional.**
/// Empty is the ordinary answer — nothing needs you — so a lane that opens on a
/// quiet world still says so, once, rather than leaving a seat unable to tell a
/// quiet world from a lane that never opened.
#[test]
fn the_first_frame_is_the_answer_at_connect_even_when_it_is_empty() {
    let (_dir, _cell, _clock, mut attend) = lane(vec![], &[WS]);
    assert_eq!(
        attend.look(),
        Some(vec![]),
        "the answer as of connect, empty and stated"
    );
    assert_eq!(
        attend.look(),
        None,
        "and the very same derivation owes nothing more"
    );
}

/// **A republish that changed nothing writes no frame, however much time
/// passed.** The ages in the rows moved — a whole minute of them — and the
/// answer did not, which is the distinction this lane's battery argument rests
/// on: a frame is a radio wake, and one that says only "it is later now" is the
/// 15 s poll §14.1 refuses to degrade into.
#[test]
fn a_republish_that_changed_nothing_writes_no_frame() {
    let (dir, cell, clock, mut attend) = lane(vec![asking("c-1")], &[WS]);
    let first = attend.look().expect("the answer at connect");
    assert_eq!(named(&first), ["c-1"], "one conversation is asking");
    assert_eq!(first[0].age_secs, 0);

    clock.advance(Duration::from_mins(1));
    crate::state::publish_snapshot(&cell, published(&dir, vec![asking("c-1")]));
    assert_eq!(
        attend.look(),
        None,
        "a fresh derivation of the same answer is not a change"
    );
}

/// **The answer changing is the whole trigger.** A conversation that starts
/// asking is a frame; the same derivation asked twice is not.
#[test]
fn an_answer_that_changed_is_the_next_frame() {
    let (dir, cell, _clock, mut attend) = lane(vec![], &[WS]);
    assert_eq!(attend.look(), Some(vec![]), "quiet at connect");

    crate::state::publish_snapshot(&cell, published(&dir, vec![asking("c-1")]));
    let frame = attend.look().expect("the answer changed");
    assert_eq!(named(&frame), ["c-1"]);
    assert_eq!(
        frame[0].signals[0],
        crate::attention::AttentionKind::Notify,
        "and it carries why it fires, so the frame IS the notification"
    );

    crate::state::publish_snapshot(&cell, published(&dir, vec![asking("c-1"), asking("c-2")]));
    assert_eq!(
        named(&attend.look().expect("a second conversation is a change")),
        ["c-1", "c-2"]
    );
}

/// **Every frame is narrowed to the scope spent at connect** (REMOTE §4). A
/// certificate the operator has not seated is answered a lane whose frames are
/// empty — absence, on the terms every one-frame read already answers it, and
/// never a refusal.
#[test]
fn the_lane_answers_only_this_askers_scope() {
    let (_dir, _cell, _clock, mut unseated) = lane(vec![asking("c-1")], &[]);
    assert_eq!(
        unseated.look(),
        Some(vec![]),
        "a seat registered nowhere sees nothing, and is told so"
    );
    let (_dir, _cell, _clock, mut seated) = lane(vec![asking("c-1")], &[WS]);
    assert_eq!(named(&seated.look().expect("a frame")), ["c-1"]);
}

/// **The hold is bounded and the lane ends, so the seat re-asks** (REMOTE
/// §14.1). Driven on a stated bound rather than the production thirty seconds —
/// the follow lane's own test shape — and the two quiet looks between the frame
/// and the end are what prove the patience is the iterator's, not the caller's.
#[test]
fn the_hold_is_bounded_and_a_quiet_lane_ends() {
    let dir = tempfile::tempdir().expect("tmp");
    let cell = new_snapshot_cell(published(&dir, vec![asking("c-1")]));
    let mut attend = Attend::holding(
        cell,
        [WS.to_owned()].into_iter().collect(),
        dir.path().join("ui.json"),
        FakeClock::new().arc(),
        2,
        Duration::ZERO,
    );
    let Some(Reply::Attention(rows)) = attend.next() else {
        unreachable!("the answer at connect is a frame")
    };
    assert_eq!(named(&rows), ["c-1"]);
    assert_eq!(
        attend.next(),
        None,
        "a lane with nothing more to say waits out its patience and ends"
    );
}

/// **The `seen` watermarks are re-read per look, not carried** (§6). A seat
/// answers a row on its ordinary connection and the lane goes quiet: the
/// acknowledgement lands in `ui.json`, which is an input to this answer exactly
/// as the derived trees are, and the §7.2 worker's own watch on that document
/// is what makes the republish that discovers it — so the lane needs no watch
/// and no knowledge of the gesture that emptied it.
#[test]
fn an_acknowledgement_elsewhere_empties_the_lane() {
    let (dir, cell, _clock, mut attend) = lane(vec![asking("c-1")], &[WS]);
    assert_eq!(named(&attend.look().expect("a frame")), ["c-1"]);

    let mut ui = UiState::open(dir.path().join("ui.json"));
    crate::boundary::answer::queue::mark_seen(
        &crate::state::latest_snapshot(&cell),
        &mut ui,
        &dir.path().join(WS),
        "c-1",
    )
    .expect("the row is answerable");
    crate::state::publish_snapshot(&cell, published(&dir, vec![asking("c-1")]));
    assert_eq!(
        attend.look(),
        Some(vec![]),
        "answered, so the answer changed and the frame says so"
    );
}
