//! **bl-55d8 at the paint layer**: a step whose `response.json` is empty and
//! whose `stderr.log` is not is a *rendered* failure (§7.3), and the reason is
//! the bytes — on the composed window, not on a view-model.
//!
//! The complaint this closes: the second message in a conversation looked as
//! though it always failed. The absence of a reply was the operator's whole
//! signal — while the reason sat in a file yog already reads past.
//! So the assertion here is the one an operator can check: render the whole
//! shell over a conversation in exactly that state, and read the adapter's own
//! sentence out of the paint output.
//!
//! The banner rides the bl-90bf grace window, so this drives the window too:
//! the frame **before** the window elapses must say nothing (an alarm that
//! flashes on a healthy send teaches the operator to distrust it), and the
//! frame after it must carry the reason.

use std::sync::Arc;
use std::time::Duration;

use super::fixture::{World, world};
use super::painted;
use crate::app::WoundGrace;
use crate::cli_outbound::Cli;
use crate::keymap::InspectorTab;
use crate::steps_view::NO_RESPONSE;
use crate::test_support::FakeClock;

/// The bl-55d8 falsifying run's `steps/<agent>/002/stderr.log`, verbatim.
const BZ_REFUSAL: &str = "bz: no workspace in this environment — providers, sign-ins and the \
model cache belong to a workspace, and there is nothing shared to fall back to. Run this inside \
a yog workspace, or focus one in yog.";

/// The needle: a phrase that appears nowhere in the shell's own vocabulary, so
/// finding it in the paint output can only mean the file's bytes reached it.
const NEEDLE: &str = "no workspace in this environment";

/// Lay the dead step down beside the fixture's answered one — the on-disk shape
/// the falsifying run left: `request.json` written, `response.json` at zero
/// bytes, **no** `meta.json` (lernie writes it only once the call returns), and
/// the adapter's refusal in `stderr.log`.
fn dead_step(world: &World, stderr: &[u8]) {
    let dir = world.ws.join("steps/c-1/002");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("request.json"), br#"{"model":"opus"}"#).unwrap();
    std::fs::write(dir.join("response.json"), b"").unwrap();
    std::fs::write(dir.join("stderr.log"), stderr).unwrap();
}

/// A world focused on the wounded conversation, with the grace gate on a clock
/// this test owns — the window is wall-clock time in the app, and a frame test
/// must not sleep through it.
fn wounded_world(stderr: &[u8]) -> (World, FakeClock) {
    let mut world = world();
    let ws = world.ws.clone();
    dead_step(&world, stderr);
    world.model.focus_agent(&ws, "c-1");
    world.model.select_tab(InspectorTab::Transcript);
    let clock = FakeClock::new();
    world.state.wound_grace = WoundGrace::new(Arc::new(clock.handle()));
    world.converge();
    (world, clock)
}

/// How far past the grace window this test moves the clock — the live cadence's
/// own catch-up bound, read off the model rather than spelled here (bl-3381).
fn past_the_window(world: &World) -> Duration {
    world.model.cadence().wound_grace()
}

#[test]
fn the_window_states_why_the_reply_never_came() {
    let (lernie, bl) = (Cli::new("lernie"), Cli::new("bl"));
    let (mut world, clock) = wounded_world(BZ_REFUSAL.as_bytes());

    // Inside the grace window the alarm is withheld — bl-90bf, unchanged.
    let early = painted(&mut world, &lernie, &bl);
    assert!(
        !early.contains(NEEDLE),
        "the wound is graced before it is believed:\n{early}"
    );

    clock.advance(past_the_window(&world));
    let text = painted(&mut world, &lernie, &bl);
    assert!(
        text.contains(NO_RESPONSE),
        "the §7.3 class, in words:\n{text}"
    );
    assert!(
        text.contains(NEEDLE),
        "THE BALL: the reason the operator never saw — the step's own \
         stderr.log, on screen:\n{text}"
    );
    assert!(
        text.contains("stderr.log"),
        "and where the whole of it lives:\n{text}"
    );
    // The retired pointer: for a turn continued by `lernie message` the driver
    // is lernie's, so no §8.1 sink exists and the trail holds nothing. A banner
    // must never send the operator somewhere empty.
    assert!(
        !text.contains("activity trail below"),
        "the dead pointer is gone:\n{text}"
    );
}

/// A driver killed outright leaves an empty `stderr.log` too. The banner still
/// fires — the wound is the state, the reason is what it carries — and it says
/// outright that nothing on disk explains it rather than inventing a cause or
/// pointing at a surface that has none.
#[test]
fn a_wound_with_nothing_to_quote_still_banners_and_says_so() {
    let (lernie, bl) = (Cli::new("lernie"), Cli::new("bl"));
    let (mut world, clock) = wounded_world(b"");
    // The window opens on the frame that first sees the wound, so it is a
    // rendered frame that starts the clock — then the wait, then the alarm.
    let _ = painted(&mut world, &lernie, &bl);
    clock.advance(past_the_window(&world));

    let text = painted(&mut world, &lernie, &bl);
    assert!(text.contains(NO_RESPONSE), "still the class:\n{text}");
    assert!(
        text.contains("nothing on disk says why"),
        "the honest end of the trail:\n{text}"
    );
}
