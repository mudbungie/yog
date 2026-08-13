//! The §8.5 line context: the seat's focus, read as what a slash command
//! elides — and the proof that a typed verb and a clicked one aim at the same
//! ball, because both read this one derivation.

use super::*;
use crate::boundary::line::parse;
use crate::boundary::{Action, Gesture};
use crate::start::BallSpec;

#[test]
fn the_focus_is_what_a_line_elides() {
    let w = world();
    let (_c, mut m) = model(&w);
    m.focus_workspace(&w.ws_cobalt);
    let ctx = m.line_context();
    assert_eq!(ctx.workspace.as_deref(), Some(w.ws_cobalt.as_path()));
    assert_eq!(ctx.project.as_deref(), Some(w.project.as_path()));
    // The §3.2 stamp is the focused ball's claimant, exactly as the ball row's
    // buttons stamp it.
    assert_eq!(ctx.name.as_deref(), Some("cobalt"));
    assert!(
        matches!(ctx.ball, Some(BallSpec::Existing { ref id, .. }) if id == "bl-work"),
        "the focused ball, whole: {:?}",
        ctx.ball
    );
    // Start-flow RAM is the shell's to fold in; the model holds none (§5.3).
    assert_eq!(ctx.prepared, None);

    // The parity claim, at the seat: `/close` with nothing typed is the Close
    // button's own action, parameter for parameter.
    assert_eq!(
        parse("/close", &ctx),
        Ok(Gesture::Act(Action::Close {
            project: w.project.clone(),
            id: "bl-work".to_owned(),
            name: "cobalt".to_owned(),
        }))
    );
}

/// A workspace holding no ball still stamps `bl` verbs with its own name —
/// that is what a ball being *acquired* is claimed as — and offers no ball,
/// project, or id for the verbs that need one.
#[test]
fn a_workspace_with_no_ball_stamps_its_own_name() {
    let w = world();
    let (_c, mut m) = model(&w);
    m.focus_workspace(&w.ws_spare);
    let ctx = m.line_context();
    assert_eq!(ctx.name.as_deref(), Some("spare"));
    assert_eq!(ctx.project, None);
    assert_eq!(ctx.ball, None);
    assert!(parse("/close", &ctx).is_err(), "no project, no close");
}
