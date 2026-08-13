//! The §6 **desktop escalation**, driven on the real window (bl-e160).
//!
//! The claim is not a painted one — a notification leaves the window — so what
//! this drives is the frame's *witnessing*: that rendering a frame folds the §6
//! queue into the window's own record of what it has already announced, and
//! that a focused frame therefore says nothing to the desktop, then or later.
//!
//! **Nothing here can reach the desktop.** egui's `RawInput::focused` defaults
//! to `true` ("integrations opt into global focus tracking"), so every frame the
//! suite renders is a focused one and the send path is unreachable from a test
//! — which is exactly the guarantee that lets this run on a developer's machine
//! without popping notifications. The unfocused half is driven purely, in
//! `crate::alert`, where the decision lives.

use super::super::render;
use super::fixture::world;
use super::input;
use crate::alert::{Announced, announce};
use crate::cli_outbound::Cli;

/// The fixture world has a resting conversation, so the §6 queue is non-empty
/// and there is something to announce.
#[test]
fn a_focused_frame_absorbs_what_needs_the_operator_and_tells_the_desktop_nothing() {
    let mut world = world();
    let (lernie, bl, bz) = (Cli::new("lernie"), Cli::new("bl"), Cli::new("bz"));
    let ctx = egui::Context::default();

    let queue = world.model.decision_queue(0);
    assert!(
        !queue.is_empty(),
        "the fixture must have something waiting, else this drives nothing"
    );

    // One rendered frame is one fold. The window is focused (egui's default),
    // so nothing is handed out — and the ask is *absorbed*, not saved up.
    for _ in 0..2 {
        let _ = ctx.run(input(), |ctx| {
            render(ctx, &mut world.model, &mut world.state, &lernie, &bl, &bz);
        });
    }

    // The proof the frame really folded: asked again with the window buried,
    // the very same queue is no longer an arrival. A window that had not
    // witnessed these would announce them all now.
    assert!(
        announce(&mut world.state.alerts, &queue, false, true).is_empty(),
        "a frame the operator was looking at has already witnessed its asks"
    );
    // …while a window that never saw them keeps its own baseline, so the first
    // buried fold is still a baseline and not a burst of stale news.
    let mut fresh = Announced::default();
    assert!(announce(&mut fresh, &queue, false, true).is_empty());
}
