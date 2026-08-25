//! The §3.8 fan's candidates on the work-diff surface (VISION §5 V3 item 3,
//! bl-c2bd): one [`Attempt`] row per fan candidate, each read at the ruled
//! range — the obligation's own `work/<id>` target against the candidate's
//! private `attempt/<handle>` source — plus the **derived acceptance mark**
//! (V3.2: *"The UI's mark is a rendered consequence of the target's history,
//! never a yog-owned winner field"*).
//!
//! Membership is [`crate::fan::cohort`]'s fold over the workspace's own trail,
//! and the obligation is the same fact the §8.6 writable root reads it from:
//! the workspace's **last** `bl claim` row names the project and the ball
//! ([`crate::control::root::claimed`]) — one rule, one home, consumed twice.
//! Nothing here stores anything: a candidate delivered a year ago and one
//! fanned a minute ago answer from the same two reads, refs and trail.

use std::path::Path;

use balls::delivery_path::{attempt_branch, work_branch};
use balls::layout::Xdg;

use crate::app::Snapshot;
use crate::control::root::claimed;
use crate::fan::{self, delivered_commit};
use crate::opslog::OpEntry;

use super::Attempt;
use super::read::diff_change;

/// Every fan candidate the workspace at `workspace` (named `name`) has bound,
/// as work-diff rows. A workspace that never fanned has none — the cohort fold
/// finding no fire row bound to an attempt path is the derivation stating "no
/// fan here", not an empty special case. The claim rows this listing rides
/// behind survive the ball's close only as long as the ball does; these rows
/// survive as long as the trail's fire rows and the retained refs do, which is
/// what "losers stay inspectable" means on this surface (§4.10 item 6).
pub(super) fn candidates(
    snap: &Snapshot,
    workspace: &Path,
    entries: &[OpEntry],
    xdg: &Xdg,
    name: &str,
) -> Vec<Attempt> {
    let Some((project, ball)) = claimed(entries, name) else {
        return Vec::new();
    };
    let named = snap.project_name(&project);
    let target = work_branch(&ball);
    fan::cohort::members(entries, xdg, &project, workspace)
        .into_iter()
        .map(|member| Attempt {
            project: named.clone(),
            ball_id: ball.clone(),
            delivered: delivered_commit(&project, &target, &member.handle),
            change: diff_change(&project, target.clone(), attempt_branch(&member.handle)),
            handle: Some(member.handle),
        })
        .collect()
}
