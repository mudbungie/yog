//! **The §3.2/§3.5 ball rows, selected out of the answered listing** (REMOTE
//! §9.7, bl-b4b5) — [`convs::visible`](super::convs::visible) and
//! [`convs::selection`](super::convs::selection)'s sibling at the other noun.
//!
//! Until this ball the §11 balls section asked `AppModel` three questions about
//! one fact: `ws_balls` (every bound ball of a workspace), `roster_ball_rows`
//! (that list minus the ones ▶ Continue already renders in full), and
//! `bound_ball` (one of them by id). All three folded the window's own
//! `join_rows` on the paint thread, which is the read a thin seat cannot make.
//!
//! One workspace-addressed question answers the list
//! ([`Query::WorkspaceBalls`](crate::boundary::Query::WorkspaceBalls)); the
//! other two were never questions at all, but **selections** out of it — the
//! same shape bl-48ae found for the selection's own facts. So the ask that pays
//! for the whole family is the section's own, and a menu, a ▶ Continue row and
//! the spend rows beside them cannot be reading three answers of three ages.

use super::BoundBall;

/// The **roster's own** rows: the answered listing minus the balls the ▶
/// Continue affordance already renders in full ([`crate::start::is_resume_eligible`]).
///
/// The section's rows partition the §3.5 states — ReadyStartable → ▶ Start,
/// Bound → ▶ Continue, Delivered → this list — so one ball is one row. It did
/// not before (bl-abbe): a bound ball drew the Continue row *and*, below the
/// new-ball form, a bare grey id with no title, no state and no verb. The
/// answered listing itself is unfiltered — the workspace pane's §3.2 strip
/// wants *every* ball the workspace bound, duplicate or not.
pub fn roster(rows: &[BoundBall]) -> Vec<BoundBall> {
    rows.iter()
        .filter(|b| !crate::start::is_resume_eligible(b.state))
        .cloned()
        .collect()
}

/// The ball `id` as this workspace has it bound — the object the ▶ Continue
/// row's §11 accelerator menu acts on (bl-abbe). A pointer-targeted menu may
/// not re-derive its object from the focus (the resumed ball's workspace need
/// not be the focused one), and the ball's own claimant is what its §8.2 verbs
/// stamp `--as` (§3.2). `None` when the listing carries no such ball.
pub fn bound(rows: &[BoundBall], id: &str) -> Option<BoundBall> {
    rows.iter().find(|b| b.id == id).cloned()
}

#[cfg(test)]
mod tests;
