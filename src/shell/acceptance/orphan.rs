//! **bl-ace6 at the paint layer**: a delivered message with no driver is a
//! *rendered* fact, and the reason is the last driver's own words.
//!
//! The complaint this closes: the 2nd..Nth message of a conversation whose
//! driver died at the boundary — an unpaired-tail decline, a crashed launch
//! — looked like a chat that simply stopped answering. The deposit
//! succeeded, `ops.jsonl` said `exit 0`, no step was created so the §7.3
//! wound had nothing to hang on, and the one copy of the cause sat in
//! `steps/<agent>/driver.log`, which nothing read. So the assertion here is
//! the operator's own check: render the whole shell over a conversation in
//! exactly that state and read the driver's sentence out of the paint
//! output.
//!
//! The banner rides the same bl-90bf grace discipline as the wound — a
//! frame inside the window says nothing (delivery happens under the
//! driver's lock, so a healthy send's mail-with-free-lock moment is only
//! the relaunch gap), and the frame after it carries the reason.

use std::sync::Arc;

use super::fixture::{World, world};
use super::painted;
use crate::app::WoundGrace;
use crate::cli_outbound::Cli;
use crate::keymap::InspectorTab;
use crate::steps_view::ORPHANED_MAIL;
use crate::test_support::FakeClock;

/// The shape lernie's own decline writes to `driver.log` — its `advance`
/// erroring out after the deposit delivered (lernie §6).
const DRIVER_WORDS: &str = "lernie: branch tip is an assistant entry with tool_use unmatched \
by committed tool results — declined (ARCH 6)";

/// The needle: a phrase from the file's bytes, in no shell vocabulary.
const NEEDLE: &str = "tool_use unmatched";

/// Lay the orphan down: a third transcript entry that is delivered mail
/// (newest, unanswered), and the dead driver's words beside the steps.
fn orphaned_mail(world: &World, driver_log: Option<&[u8]>) {
    let messages = world.ws.join("agents/c-1/messages");
    std::fs::write(messages.join("003-user.md"), "hello?").unwrap();
    if let Some(bytes) = driver_log {
        let steps = world.ws.join("steps/c-1");
        std::fs::create_dir_all(&steps).unwrap();
        std::fs::write(steps.join("driver.log"), bytes).unwrap();
    }
}

/// A world focused on the orphaned conversation, with the grace gate on a
/// clock this test owns.
fn orphaned_world(driver_log: Option<&[u8]>) -> (World, FakeClock) {
    let mut world = world();
    let ws = world.ws.clone();
    orphaned_mail(&world, driver_log);
    world.model.focus_agent(&ws, "c-1");
    world.model.select_tab(InspectorTab::Transcript);
    let clock = FakeClock::new();
    world.state.orphan_grace = WoundGrace::new(Arc::new(clock.handle()));
    world.converge();
    (world, clock)
}

#[test]
fn the_window_states_why_the_chat_stopped_answering() {
    let (lernie, bl) = (Cli::new("lernie"), Cli::new("bl"));
    let (mut world, clock) = orphaned_world(Some(DRIVER_WORDS.as_bytes()));

    // Inside the grace window the alarm is withheld — the healthy send's
    // relaunch gap must never flash it.
    let early = painted(&mut world, &lernie, &bl);
    assert!(
        !early.contains(ORPHANED_MAIL),
        "the orphan is graced before it is believed:\n{early}"
    );

    clock.advance(world.model.cadence().wound_grace());
    let text = painted(&mut world, &lernie, &bl);
    assert!(text.contains(ORPHANED_MAIL), "the class, in words:\n{text}");
    assert!(
        text.contains(NEEDLE),
        "THE BALL: the dead driver's own words, on screen:\n{text}"
    );
    assert!(
        text.contains("driver.log"),
        "and where the whole of it lives:\n{text}"
    );
}

/// A launch that died without ever writing a word still banners — the
/// state is the fact, and the banner says outright that nothing on disk
/// explains it.
#[test]
fn an_orphan_with_nothing_to_quote_still_banners_and_says_so() {
    let (lernie, bl) = (Cli::new("lernie"), Cli::new("bl"));
    let (mut world, clock) = orphaned_world(None);
    let _ = painted(&mut world, &lernie, &bl);
    clock.advance(world.model.cadence().wound_grace());

    let text = painted(&mut world, &lernie, &bl);
    assert!(text.contains(ORPHANED_MAIL), "still the class:\n{text}");
    assert!(
        text.contains("nothing on disk says why"),
        "the honest end of the trail:\n{text}"
    );
}
