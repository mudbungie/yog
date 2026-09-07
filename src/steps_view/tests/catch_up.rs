//! The §7.2 **catch-up window** the engine waits out before it states a
//! no-response wound (bl-90bf/bl-18e8, judged on the engine by bl-776a).
//!
//! The wound's liveness half rides the last published snapshot and a driver
//! taking its flock announces nothing, so a freshly-started call reads as a
//! wound until the cache catches up. The window is spent here rather than named
//! in a reply: what crosses the §8.5 boundary is already judged, so no seat
//! re-implements `Cadence`'s arithmetic and none flashes the alarm the grace
//! exists to prevent.

use std::time::Duration;
use tempfile::tempdir;

use super::{AGENT, write_file};
use crate::app::Cadence;
use crate::git_tree::{AgentState, Framing};
use crate::steps_view::{Wound, build, latest_wound};

/// A call that started and produced nothing — `request.json` written, an empty
/// `response.json`, no `meta.json`. The request's mtime is *now*, so this is
/// the shape a live send wears for the whole of the window.
fn just_sent(ws: &std::path::Path) {
    write_file(ws, "001", "request.json", br#"{"model":"opus"}"#);
    write_file(ws, "001", "response.json", b"");
}

/// A clock far past any step this file writes.
const LATER: i64 = 4_000_000_000;

#[test]
fn a_call_younger_than_the_window_is_in_flight_not_a_wound() {
    let dir = tempdir().unwrap();
    let ws = dir.path();
    just_sent(ws);
    let now = i64::try_from(
        std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    )
    .unwrap();
    let grace = Cadence::default().wound_grace();
    assert!(
        !latest_wound(&build(ws, AGENT, AgentState::Stopped, now, grace)).wounded(),
        "the cached liveness has not had time to say a driver took the lock"
    );
    assert!(
        latest_wound(&build(ws, AGENT, AgentState::Stopped, LATER, grace)).wounded(),
        "past the window the honest wound is stated — delayed, never dropped"
    );
}

/// The anchor is the step's own call start, so a step with no `request.json`
/// carries no claim that it is young and the wound stands. Nothing on disk
/// saying a step is fresh must never read as a step that is.
#[test]
fn a_step_with_no_call_start_is_not_excused() {
    let dir = tempdir().unwrap();
    let ws = dir.path();
    write_file(ws, "001", "response.json", b"");
    assert!(
        latest_wound(&build(ws, AGENT, AgentState::Stopped, 0, Duration::MAX)).wounded(),
        "an unreadable stamp never hides a wound, however wide the window"
    );
}

/// Only the two **unanswered** classes wait: they are the ones whose truth
/// depends on the stale liveness half. A refusal settled on disk the instant it
/// was written, so holding it back would buy a second's silence for nothing.
#[test]
fn a_settled_refusal_never_waits() {
    let dir = tempdir().unwrap();
    let ws = dir.path();
    write_file(ws, "001", "request.json", br#"{"model":"opus"}"#);
    write_file(
        ws,
        "001",
        "response.json",
        b"{\"type\":\"error\",\"status\":401,\"message\":\"Unauthorized: check credentials\"}\n{\"type\":\"end\"}\n",
    );
    let wound = latest_wound(&build(ws, AGENT, AgentState::Stopped, 0, Duration::MAX));
    assert!(
        matches!(wound, Wound::Refused(_)),
        "stated at once, inside any window: {wound:?}"
    );
}

/// **THE BALL** (bl-ab53): the §4.4 framing waits on this same reading, and for
/// the same reason. A step being streamed has a tail with no terminal segment,
/// which is byte-for-byte what a signalled writer leaves — so a live
/// conversation read `killed` once per step, on the one word that makes an
/// interrupt legible.
#[test]
fn a_step_being_filled_reads_in_flight_and_a_cut_one_still_reads_killed() {
    let dir = tempdir().unwrap();
    let ws = dir.path();
    // Mid-stream: a request, tokens on the wire, no ending yet.
    write_file(ws, "001", "request.json", br#"{"model":"opus"}"#);
    write_file(
        ws,
        "001",
        "response.json",
        b"{\"type\":\"message_start\"}\n{\"type\":\"delta\"}\n",
    );
    let grace = Cadence::default().wound_grace();
    let framing = |state, now| build(ws, AGENT, state, now, grace).steps[0].framing;
    // A driver holding the lock is the answer whatever the clock says.
    assert_eq!(framing(AgentState::Live, LATER), Framing::InFlight);
    assert_eq!(framing(AgentState::InFlight, LATER), Framing::InFlight);
    // And with the lock free but the call younger than the window, the stale
    // half has not had time to say otherwise — the same seconds the wound waits.
    let now = i64::try_from(
        std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    )
    .unwrap();
    assert_eq!(framing(AgentState::Stopped, now), Framing::InFlight);
    // Past it, with nobody driving, the step was cut and says so. `killed`
    // keeps meaning the signal.
    assert_eq!(framing(AgentState::Stopped, LATER), Framing::Killed);
}

/// Only the **newest** step can be the one being filled: an earlier endingless
/// step is unambiguous and stays the place the conversation was cut, even under
/// a live driver — exactly the rule the wound is gated by one line above.
#[test]
fn only_the_newest_step_is_ever_read_as_in_flight() {
    let dir = tempdir().unwrap();
    let ws = dir.path();
    write_file(
        ws,
        "001",
        "response.json",
        b"{\"type\":\"message_start\"}\n",
    );
    write_file(ws, "002", "request.json", br#"{"model":"opus"}"#);
    write_file(
        ws,
        "002",
        "response.json",
        b"{\"type\":\"message_start\"}\n",
    );
    let view = build(
        ws,
        AGENT,
        AgentState::Live,
        LATER,
        Cadence::default().wound_grace(),
    );
    assert_eq!(view.steps[0].framing, Framing::Killed);
    assert_eq!(view.steps[1].framing, Framing::InFlight);
}

/// And a step that settled is untouched: only an endingless tail was ever
/// ambiguous, so a complete or failed step under a live driver keeps its own
/// word.
#[test]
fn a_settled_step_keeps_its_framing_under_a_live_driver() {
    let dir = tempdir().unwrap();
    let ws = dir.path();
    write_file(
        ws,
        "001",
        "response.json",
        b"{\"type\":\"finish\",\"reason\":\"stop\"}\n{\"type\":\"end\"}\n",
    );
    let view = build(
        ws,
        AGENT,
        AgentState::Live,
        LATER,
        Cadence::default().wound_grace(),
    );
    assert_eq!(view.steps[0].framing, Framing::Complete);
}
