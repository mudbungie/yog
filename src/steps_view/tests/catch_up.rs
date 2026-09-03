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
use crate::git_tree::AgentState;
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
