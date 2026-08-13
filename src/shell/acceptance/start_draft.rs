//! **A pending start draft replaces the composer** (bl-6ad8), driven through
//! the real window.
//!
//! §11's rule is one box and one Enter (S0). A ball start legitimately opens the
//! §8.1 goal draft — that is the rung's prefill, and bl-9acf left exactly this
//! case standing — but the draft was painted as one more bottom panel *above* a
//! composer that stayed live beneath it: two goal boxes, each with its own
//! greyed name prediction, and no way to tell which one Enter would fire.
//!
//! So the draft **takes the composer's seat** rather than stacking on it, and
//! Cancel/Escape gives it back. The composer's own draft is untouched
//! throughout: it lives in the per-target map (`actions::Drafts`, bl-a69a),
//! which the start pane never reaches into, so restoring the box restores what
//! was in it.
//!
//! The pending draft is seated directly — it is precisely what
//! `start_pane::run_prepare` leaves after ▶ Start / `s` on a ready ball, and the
//! rung that reaches it is proven from the gesture end in [`super::raise`].

use super::fixture::world;
use super::screen::{Screen, press};
use crate::start::Prepared;

/// The ball rung's prefill, headline-first as §3.3 composes it.
const BALL_GOAL: &str = "Ball bl-1234: fix the gate\n\nthe gate is stuck";

#[test]
fn a_pending_start_draft_replaces_the_composer_and_cancel_restores_it() {
    let mut world = world();
    let screen = Screen::new();
    assert!(screen.idle(&mut world), "the cursor starts in the composer");
    screen.frame(
        &mut world,
        vec![egui::Event::Text("ship the goal".to_owned())],
    );

    let before = screen.text(&mut world);
    assert!(
        before.contains("New prompt"),
        "the composer is up:\n{before}"
    );
    assert!(
        before.contains("ship the goal"),
        "holding its own draft:\n{before}"
    );
    assert!(
        !before.contains("Start goal →"),
        "and nothing is pending yet:\n{before}"
    );

    // What ▶ Start on a ready ball leaves: a prefilled goal awaiting Send.
    let ws = world.ws.clone();
    world.state.start.pending = Some(Prepared {
        name: "ws".to_owned(),
        workspace: ws.clone(),
        cwd: ws,
        goal: BALL_GOAL.to_owned(),
        origin: crate::opslog::Origin::Balls,
    });

    let pending = screen.text(&mut world);
    assert!(
        pending.contains("Start goal →") && pending.contains("Send (detached prompt)"),
        "the start draft is up:\n{pending}"
    );
    assert!(
        !pending.contains("New prompt"),
        "and it REPLACED the composer — one box, one Enter:\n{pending}"
    );
    assert_eq!(
        pending.matches("will be named ").count(),
        1,
        "so exactly one name prediction is on screen:\n{pending}"
    );
    assert!(
        !pending.contains("ship the goal"),
        "the composer's draft is off screen, not a second live box:\n{pending}"
    );

    // Escape — `KeyAction::Cancel` (§11). It reaches the plane directly here
    // because the vanished composer took the keyboard with it and the goal box
    // does not grab one; an operator mid-type spends a first press surrendering
    // that focus, the §11 idiom the keymap's own tests pin.
    screen.frame(
        &mut world,
        vec![press(egui::Key::Escape, egui::Modifiers::NONE)],
    );
    assert!(
        world.state.start.pending.is_none(),
        "Escape cancels the pending start"
    );

    let after = screen.text(&mut world);
    assert!(
        after.contains("New prompt"),
        "and the composer comes back:\n{after}"
    );
    assert!(
        after.contains("ship the goal"),
        "with its own draft intact — the start pane never held it:\n{after}"
    );
    assert!(
        !after.contains("Start goal →"),
        "and the draft box is gone:\n{after}"
    );
}
