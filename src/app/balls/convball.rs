//! The conversation↔ball join surface (§3.5): which balls a workspace has
//! bound.
//!
//! The **per-conversation** badge left with bl-296f: `conversation_ball` and the
//! `resolve_conv_ball` behind it were a second read of a fact
//! `Query::Conversations` has always answered on the row (`ConvRow::ball`), so
//! the §11 header folds it off the landed forest through `Selection::ball` and
//! the boundary's `answer::conv_ball` is the one derivation left.
//!
//! Split out of `app/balls.rs` at the cap. The parent owns the live `bl`
//! projection — the fetch cadence, the join rebuild, the ops tail and the
//! §8.2 verb hooks. This is what the *frontend asks of* that join, and its
//! test mirror `tests/convball.rs` already stood alone.

use super::AppModel;
use crate::projects::join;
use std::path::Path;

impl AppModel {
    /// **All** the bound balls a workspace renders (§3.5, §11 balls section):
    /// every join row whose workspace is `ws` and which carries a ball, each
    /// projected to its id + [`join::badge`]. A workspace with N bound balls
    /// shows all N (the wave-1 review fix — the old first-row `row_for` showed
    /// one arbitrary badge and let a Delivered row shadow a Bound one); an
    /// unassigned workspace yields an empty list (its UnassignedWorkspace row
    /// has no ball id).
    pub fn ws_balls(&self, ws: &Path) -> Vec<crate::nav::BoundBall> {
        self.snap
            .join_rows
            .iter()
            .filter(|r| r.workspace.as_deref() == Some(ws) && !r.ball_id.is_empty())
            .map(|r| crate::nav::BoundBall {
                id: r.ball_id.clone(),
                badge: join::badge(r.state, r.claimant.as_deref()),
                project: r.project.clone(),
                owner: join::owner_name(r),
                state: r.state,
            })
            .collect()
    }

    /// The **roster's own** ball rows for `ws` (§11 balls section, bl-abbe):
    /// [`Self::ws_balls`] minus the balls the ▶ Continue affordance already
    /// renders in full ([`crate::start::is_resume_eligible`]).
    ///
    /// The section's rows partition the §3.5 states — ReadyStartable → ▶ Start,
    /// Bound → ▶ Continue, Delivered → this list — so one ball is one row. It
    /// did not before: a bound ball drew the Continue row *and*, below the
    /// new-ball form, a bare grey id with no title, no state and no verb (the
    /// Bound badge is `None`, so the row rendered as nothing but its id).
    /// Deleting the duplicate rather than fattening it is the subtraction: the
    /// Continue row already carries `<id>: <title>`, and it now carries the
    /// row's verbs too (its §11 menu, seated on [`Self::bound_ball`]).
    ///
    /// [`Self::ws_balls`] itself is unchanged — the workspace pane's §3.2 strip
    /// wants *every* ball the workspace bound, duplicate or not.
    pub fn roster_ball_rows(&self, ws: &Path) -> Vec<crate::nav::BoundBall> {
        self.ws_balls(ws)
            .into_iter()
            .filter(|b| !crate::start::is_resume_eligible(b.state))
            .collect()
    }

    /// The ball `id` as `ws` has it bound — the object the ▶ Continue row's §11
    /// accelerator menu acts on (bl-abbe). A pointer-targeted menu may not
    /// re-derive its object from the focus (the resumed ball's workspace need
    /// not be the focused one), and the ball's own claimant is what its §8.2
    /// verbs stamp `--as` (§3.2). `None` when the workspace binds no such ball.
    pub fn bound_ball(&self, ws: &Path, id: &str) -> Option<crate::nav::BoundBall> {
        self.ws_balls(ws).into_iter().find(|b| b.id == id)
    }
}
