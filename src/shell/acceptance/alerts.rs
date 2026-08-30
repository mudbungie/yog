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
//!
//! **Both directions, since bl-f297.** The queue is `Query::Attention` over the
//! wire now (REMOTE §9.7), so the fold is on the *answer* and not on the frame:
//! a frame nobody answered is not a reading of the queue and must leave the
//! baseline exactly where it was. Each test below seeds the baseline with an
//! empty fold first, which is what makes the two outcomes distinguishable — with
//! no prior observation `announce` returns nothing whatever the frames did, and
//! the positive test alone would have passed against a seat that never asked.

use super::super::{ShellState, render};
use super::fixture::world;
use super::input;
use super::wire::wired;
use crate::AppModel;
use crate::alert::{Announced, announce};
use crate::boundary::Query;
use crate::boundary::reply::Reply;
use crate::cli_outbound::Cli;

/// The fixture world has a resting conversation, so the §6 queue is non-empty
/// and there is something to announce.
#[test]
fn a_focused_frame_absorbs_what_needs_the_operator_and_tells_the_desktop_nothing() {
    let mut world = world();
    let (litany, bl, bz) = (Cli::new("litany"), Cli::new("bl"), Cli::new("bz"));
    let ctx = egui::Context::default();

    let deps = world.model.boundary_deps(&litany, &bl);
    let Ok(Reply::Attention(queue)) = world.model.answer(&deps, &Query::Attention, 0) else {
        panic!("attention answers attention");
    };
    assert!(
        !queue.is_empty(),
        "the fixture must have something waiting, else this drives nothing"
    );

    // A window that has observed an empty world: the baseline exists and holds
    // nothing, so anything the frames go on to fold is a real advance.
    assert!(announce(&mut world.state.alerts, &[], false, true).is_empty());

    // The whole settle-then-render dance, because the fold is on the answer.
    // The window is focused (egui's default), so nothing is handed out — and
    // the ask is *absorbed*, not saved up.
    let mut paint = |model: &mut AppModel, state: &mut ShellState| {
        let _ = ctx.run(input(), |ctx| {
            render(ctx, model, state, &litany, &bl, &bz);
        });
        String::new()
    };
    let _ = wired(&mut world, &mut paint);

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

/// **An unanswered frame is not a reading of the queue** (REMOTE §9.7, bl-f297).
/// The same world, the same frames, with nothing answering: the baseline must
/// not move, so the very queue the window never heard about is still an arrival
/// afterwards. Without this the test above would pass against a seat that folded
/// an empty answer every frame — which is the failure mode the migration had to
/// design around, since that fold reads as everything having departed and then
/// as everything arriving at once.
#[test]
fn an_unanswered_frame_leaves_the_baseline_exactly_where_it_was() {
    let mut world = world();
    let (litany, bl, bz) = (Cli::new("litany"), Cli::new("bl"), Cli::new("bz"));
    let ctx = egui::Context::default();

    let deps = world.model.boundary_deps(&litany, &bl);
    let Ok(Reply::Attention(queue)) = world.model.answer(&deps, &Query::Attention, 0) else {
        panic!("attention answers attention");
    };
    assert!(!queue.is_empty(), "the fixture must have something waiting");

    assert!(announce(&mut world.state.alerts, &[], false, true).is_empty());
    for _ in 0..3 {
        let _ = ctx.run(input(), |ctx| {
            render(ctx, &mut world.model, &mut world.state, &litany, &bl, &bz);
        });
    }
    assert!(
        !announce(&mut world.state.alerts, &queue, false, true).is_empty(),
        "a frame the wire never answered witnessed nothing, so these still arrive"
    );
}
