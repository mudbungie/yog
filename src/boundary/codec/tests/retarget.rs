//! The §9.4 exit's envelope (bl-2d19) — its own file, on the same one-family
//! seam the rest of this directory is cut along, because the roster table
//! beside it is at the function-length cap.

use crate::boundary::codec::encode;
use crate::boundary::{Action, Gesture};
use serde_json::json;

pub(super) fn surface() -> Vec<Gesture> {
    vec![gesture()]
}

fn gesture() -> Gesture {
    Gesture::Act(Action::Retarget {
        workspace: "ws".into(),
        agent: "c-1".into(),
    })
}

/// Workspace and agent, and **no config field**: litany's default lineage is
/// the one yog's picker writes and the one the drift this answers is measured
/// against, so the envelope has no branch to carry and no seat has to guess one.
#[test]
fn the_retarget_exit_carries_no_config_field() {
    assert_eq!(
        encode(&gesture()),
        json!({ "op": "retarget", "workspace": "ws", "agent": "c-1" })
    );
}
