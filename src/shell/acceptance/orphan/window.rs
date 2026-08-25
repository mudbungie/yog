//! **bl-abba at the paint layer**: a tool window an executor died inside is
//! a *rendered* fact too, and the banner names the gesture that recovers it.
//!
//! The complaint this closes: an agent whose executor died mid-tool-window
//! leaves its assistant entry committed with `tool_use` blocks nobody
//! answered, no hold mark and its lock free — and yog painted **nothing**,
//! so the conversation read as an ordinary idle one that simply chose to
//! stop. The two banners beside it could not see this: the §7.3 wound wants
//! a step whose `meta.json` never landed (here the model call returned and
//! settled), and the mail banner wants a `.md` on the tail (here it is an
//! assistant `.json`). So the assertion is the operator's own check: render
//! the whole shell over a conversation in exactly that state, and read both
//! the class and the remedy out of the paint output.
//!
//! Same seat, same grace field, same banner as [`super`] — this file drives
//! the shape, not a second mechanism.

use std::sync::Arc;

use super::super::fixture::{World, world};
use super::super::painted;
use super::{DRIVER_WORDS, NEEDLE, dead_driver};
use crate::app::WoundGrace;
use crate::cli_outbound::Cli;
use crate::keymap::InspectorTab;
use crate::steps_view::ORPHANED_WINDOW;
use crate::test_support::FakeClock;

/// Lay the crashed window down: a third transcript entry that is an
/// assistant turn calling a tool, with no `tool_result` after it.
fn crashed_window(world: &World, driver_log: Option<&[u8]>) {
    let messages = world.ws.join("agents/c-1/messages");
    std::fs::write(
        messages.join("003-opus.json"),
        br#"[{"type":"tool_use","id":"toolu_9","name":"bash","input":{"command":"make check"}}]"#,
    )
    .unwrap();
    dead_driver(world, driver_log);
}

/// A world focused on the crashed conversation, with the grace gate on a
/// clock this test owns.
fn crashed_world(driver_log: Option<&[u8]>) -> (World, FakeClock) {
    let mut world = world();
    let ws = world.ws.clone();
    crashed_window(&world, driver_log);
    world.model.focus_agent(&ws, "c-1");
    world.model.select_tab(InspectorTab::Transcript);
    let clock = FakeClock::new();
    world.state.orphan_grace = WoundGrace::new(Arc::new(clock.handle()));
    world.converge();
    (world, clock)
}

#[test]
fn the_window_states_that_the_turn_died_mid_tool_call() {
    let (lernie, bl) = (Cli::new("lernie"), Cli::new("bl"));
    let (mut world, clock) = crashed_world(Some(DRIVER_WORDS.as_bytes()));

    // The same grace discipline: a driver between committing its assistant
    // entry and committing its first tool result wears this exact shape, so
    // the alarm is withheld until the window has elapsed.
    let early = painted(&mut world, &lernie, &bl);
    assert!(
        !early.contains(ORPHANED_WINDOW),
        "a live driver mid-window is graced, not accused:\n{early}"
    );

    clock.advance(world.model.cadence().wound_grace());
    let text = painted(&mut world, &lernie, &bl);
    assert!(
        text.contains(ORPHANED_WINDOW),
        "THE BALL: the class, in words, where nothing painted before:\n{text}"
    );
    assert!(
        text.contains("a message revives it"),
        "and the one gesture that recovers it:\n{text}"
    );
    assert!(
        text.contains(NEEDLE),
        "the dead driver's own words, on screen:\n{text}"
    );
}

/// An executor killed outright wrote nothing anywhere — the state is still
/// the fact, and the banner says outright that nothing on disk explains it.
#[test]
fn a_crashed_window_with_nothing_to_quote_still_banners_and_says_so() {
    let (lernie, bl) = (Cli::new("lernie"), Cli::new("bl"));
    let (mut world, clock) = crashed_world(None);
    let _ = painted(&mut world, &lernie, &bl);
    clock.advance(world.model.cadence().wound_grace());

    let text = painted(&mut world, &lernie, &bl);
    assert!(text.contains(ORPHANED_WINDOW), "still the class:\n{text}");
    assert!(
        text.contains("nothing on disk says why"),
        "the honest end of the trail:\n{text}"
    );
}
