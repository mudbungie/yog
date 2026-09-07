//! **The follow lane's answer, taken once** (REMOTE §3, §5.5; bl-73e7,
//! bl-5305) — what an intake that cannot hold a connection gets.
//!
//! It is the general path with one frame and never a degraded reading of it, so
//! it answers the same two liveness questions [`Follow::poll`](super::Follow)
//! asks look by look: the prose only while a model call streams, the tool
//! window for as long as a driver holds the lease — which is the span the
//! step's tools run in. Answering the window off the narrower question would
//! drop it precisely during the tool phase, which is the defect this ball
//! closed on the held lane, said again on this one.
//!
//! **A held conversation is followed, not called at rest** (bl-58bb). The
//! capability control's park is the lane's third window transition, and the
//! liveness question that opens the window admits it beside the lease — a
//! branch waiting on the operator has not finished its step, and the operator
//! it is waiting on is the one holding this read.
//!
//! Its own file beside the held read rather than a third of the chokepoint's
//! [`inspector`](crate::boundary::answer::inspector): what one look at this
//! lane is worth is this lane's rule, and it has to agree with the held read's
//! by sitting next to it.

use std::path::Path;

use crate::app::Snapshot;
use crate::boundary::answer::inspector;
use crate::boundary::reply::FollowFrame;
use crate::git_tree::{AgentState, latest_response_path};

/// One frame, answered from the published derivation and the workspace's own
/// bytes. A conversation nobody is working has neither half, which is an empty
/// frame — the general path with no input, not a case of its own.
pub(crate) fn once(snap: &Snapshot, ws: &Path, agent: &str) -> FollowFrame {
    let parked = inspector::held_of(snap, ws, agent);
    let live = matches!(
        inspector::state_of(snap, ws, agent),
        AgentState::Live | AgentState::InFlight
    ) || parked.is_some();
    FollowFrame {
        stream: inspector::live_tail(snap, ws, agent).unwrap_or_default(),
        tools: live
            .then(|| latest_response_path(ws, agent))
            .flatten()
            .as_deref()
            .and_then(Path::parent)
            .map(|step| super::tools::whole(step, parked.as_ref()))
            .unwrap_or_default(),
    }
}
