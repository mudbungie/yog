//! Round-trip tables for the gesture codec (§8.5 deliverable 3): **every**
//! variant re-enters as itself. The other half of that deliverable — every
//! malformed envelope refusing with a reason, never a guessed default — is
//! [`refusals`], split out at §12's cap.
//!
//! **The values are [`surface`], not this file** (bl-32cb): one list per
//! family, walked by the round trips here *and* by the conformance corpus
//! ([`crate::boundary::corpus`]), so the fixtures every wire client replays and
//! the fixtures yog proves itself against are one set rather than two that
//! drift. A variant added tomorrow with no entry there leaves its own encode
//! arm unexecuted, which the coverage floor refuses — the same gate the reply
//! surface has always stood on.

use super::*;
use std::path::PathBuf;

mod balls;
mod control;
mod fan;
mod fleet;
mod fork;
mod query;
mod refusals;
mod retarget;
/// The §8.1 start family's own enum tables.
mod start;
pub(crate) mod surface;

pub(super) fn rt(gesture: Gesture) {
    let encoded = encode(&gesture);
    assert_eq!(decode(&encoded), Ok(gesture.clone()), "via {encoded}");
}

pub(super) fn p(s: &str) -> PathBuf {
    PathBuf::from(s)
}

/// The whole surface, in one walk: every action and every query re-enters as
/// itself.
#[test]
fn every_gesture_variant_round_trips() {
    for gesture in surface::gestures() {
        rt(gesture);
    }
}

#[test]
fn a_stop_without_the_children_field_defaults_to_false() {
    let decoded = decode(&serde_json::json!({
        "op": "stop", "workspace": "ws", "agent": "c-1"
    }));
    assert_eq!(
        decoded,
        Ok(Gesture::Act(Action::Stop {
            workspace: "ws".into(),
            agent: "c-1".into(),
            children: false,
        }))
    );
}
