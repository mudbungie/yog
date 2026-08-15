//! **One workspace's bound balls, with their figures** (REMOTE §9.7, bl-b4b5)
//! — [`Query::WorkspaceBalls`](crate::boundary::Query::WorkspaceBalls)'s
//! derivation, cut off the chokepoint at §12's budget like the four beside it.
//!
//! It is the §3.2/§3.5 family taken as **one question at one altitude** rather
//! than as the eight accessors the window used to ask.
//! [`Query::Balls`](crate::boundary::Query::Balls) answers the binding facts of
//! the whole world; this answers *what one workspace holds*, which is what
//! every §11 balls surface actually paints — and it is addressed by the §3.1
//! name, so a seat holding a focus can ask it without the engine-side join
//! bl-7407 refused.
//!
//! **The spend rides the row.** `AppModel::ball_spend` was a second read, per
//! ball, per frame, of the same `Snapshot::bills` fold this one walks — and a
//! figure that arrived from a different derivation than the row it sits beside
//! is the disagreement bl-296f's activity chip had. One walk, one answer.

use std::path::Path;

use crate::app::Snapshot;
use crate::nav::BoundBall;
use crate::projects::join;
use crate::ui_state::UiState;

/// Every ball `ws` has bound (§3.5, §11 balls section), in the join's own
/// order: each projected to its id, its [`join::badge`], the §5.1 #1 project
/// name its `bl` verbs run in, the claimant they stamp `--as`, the §3.5 state
/// the enablement predicates read, and its priced figure.
///
/// A workspace with N bound balls answers all N — the wave-1 review fix; the
/// old first-row projection showed one arbitrary badge and let a Delivered row
/// shadow a Bound one. An unassigned workspace answers nothing: its
/// UnassignedWorkspace row names no ball, which is the absence of one and not
/// a row about one.
pub fn ws_balls(snap: &Snapshot, ui: &UiState, ws: &Path) -> Vec<BoundBall> {
    let name = snap.ws_name(ws);
    let prices = ui.prices();
    let bills = snap.bills.get(ws).cloned().unwrap_or_default();
    snap.join_rows
        .iter()
        .filter(|r| r.workspace.as_deref() == Some(name.as_str()) && !r.ball_id.is_empty())
        .map(|r| BoundBall {
            spend: crate::spend::of_ball(
                &bills,
                &crate::board::stamped_roots(&snap.trees, ws, &r.ball_id),
                &prices,
            ),
            id: r.ball_id.clone(),
            badge: join::badge(r.state, r.claimant.as_deref()),
            project: r.project.clone(),
            owner: join::owner_name(r),
            state: r.state,
        })
        .collect()
}
