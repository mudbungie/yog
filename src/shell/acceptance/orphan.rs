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

/// **The rising edge itself** (bl-18e8) — the beat the two above could not
/// be: they hand-place a world that is *already* orphaned and ask what the
/// window does to it, so the only latency they ever drive is the gate's own.
/// This one drives the healthy send: the driver takes the agent's lock and
/// delivers the mail under it (lernie §2.11), so disk reads "delivered mail
/// on the tail" the instant the deposit returns while the published snapshot
/// still says nobody is driving. Nothing is wrong; the cache is behind.
///
/// The catch-up is not a sweep tick. It is the whole §7.2 chain — the
/// coalescing window, then a derivation pass that may be queued behind a full
/// sweep of every workspace, then one `ASK_PERIOD` before the frame holds the
/// published answer — and every leg of it is read off the model's own cadence
/// here rather than spelled. Walk the clock along that chain and the alarm
/// must be silent at every step of it, which under the old sizing — cheap
/// sweep plus debounce, 2.1 s on the shipped rhythm — it was not: the operator
/// got a second of ichor red on a send that worked.
///
/// The coalescing window is turned off ([`super::inbox_composer::quick`]) for
/// the reason every other clock-driven drive turns it off: it is the *worker's*
/// wall clock, not this test's, and the suite holds no sleeps. The legs that
/// matter are the two the old window never covered.
#[test]
fn a_healthy_send_never_flashes_the_alarm_while_the_snapshot_catches_up() {
    let (lernie, bl) = (Cli::new("lernie"), Cli::new("bl"));
    let mut world = super::inbox_composer::quick(world());
    let ws = world.ws.clone();
    world.model.focus_agent(&ws, "c-1");
    world.model.select_tab(InspectorTab::Transcript);
    let clock = FakeClock::new();
    world.state.orphan_grace = WoundGrace::new(Arc::new(clock.handle()));
    world.converge();

    // The send, as the §5.1 #28 probe actually reads it: the driver's own fd on
    // the agent's inbox directory, held by this process, and the mail delivered
    // under that lock. The published snapshot has seen neither.
    let driver = std::fs::File::open(ws.join("inbox/c-1")).expect("the inbox dir");
    orphaned_mail(&world, None);

    let cadence = world.model.cadence();
    let chain = [
        cadence.debounce,
        // The pass bound at its widest — `Cadence::late_pass` of a full sweep,
        // which is the period a full sweep is budgeted (bl-4b28).
        cadence.full_sweep,
        crate::wire::asker::ASK_PERIOD,
    ];
    for leg in chain {
        let text = painted(&mut world, &lernie, &bl);
        assert!(
            !text.contains(ORPHANED_MAIL),
            "the send is healthy; the snapshot is merely behind:\n{text}"
        );
        clock.advance(leg);
    }

    // The snapshot catches up — the driver was holding the lock all along, so
    // the state clears and the window closes on nothing. A whole grace further
    // on, and the alarm has still never been on screen.
    super::inbox_composer::converge_ws(&mut world);
    // One round trip for the catch-up to reach the frame: a landed answer is
    // adopted on the refresh after the frame that kept its question standing,
    // so this paint is the one that carries it — and it is still inside the
    // window, so it could not have banner-ed either way.
    let _ = painted(&mut world, &lernie, &bl);
    clock.advance(cadence.wound_grace());
    let text = painted(&mut world, &lernie, &bl);
    assert!(
        !text.contains(ORPHANED_MAIL),
        "a state that healed leaves nothing behind:\n{text}"
    );
    drop(driver);
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
