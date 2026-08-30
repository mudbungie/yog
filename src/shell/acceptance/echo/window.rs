//! **The §3.4 start window, driven through the real window** (bl-56c6): Enter
//! to the detached driver's first write, the stretch in which the conversation
//! has only the §3.3 name it minted and that name resolves nowhere.
//!
//! Operator report: the outbox does not work while a conversation is being
//! instantiated by its first message. Every seat improvised in that stretch —
//! a second send bounced *"unknown conversation"*, the composer's queue was
//! empty for the first message, the centre carried the refusal in ichor, `Stop`
//! was offered on a driver no signal could reach, and a second Enter while the
//! pair of acts was still in flight minted the same name twice.
//!
//! None of it was reachable from a test, and that is why it stood: the world
//! these beats build is the one where the driver **has written nothing**, and
//! the substrate they run is a `litany` that records every verb it is handed —
//! so *"one conversation, one name, the follow-up delivered after the start"*
//! is read off what the substrate was actually asked to do, not inferred.

/// **The gap between a post and its receipt** — the two beats that can only be
/// driven from inside it, in their own file at §12's budget (bl-56c6).
mod gap;

use super::super::fixture::{World, seed_world};
use super::super::screen::Screen;
use super::harness::{MINTED_FIRST, SAID, quick, rows_named, say, shot, typed, world};
use crate::cli_outbound::Cli;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use tempfile::tempdir;

/// The second thing the operator says, during the window. Its own words, so a
/// beat can tell the two sends apart wherever they land.
pub(super) const AGAIN: &str = "and hurry";

/// The id the detached driver's branch turns out to have — the fact the whole
/// window is waiting for, and never something a seat could have guessed.
pub(super) const LANDED: &str = "c-2";

/// A `litany` that **writes down every verb it is handed** and otherwise
/// behaves like [`super::super::fixture::fake_litany`]. The claims these beats
/// make are about what did and did not reach the substrate — one `prompt` for
/// two Enters, one `message` and only after the start resolved — and no read of
/// the glass can make them.
fn recording_litany(dir: &Path, log: &Path) -> Cli {
    let path = dir.join("litany");
    std::fs::write(
        &path,
        format!(
            "#!/bin/sh\nprintf '%s\\n' \"$*\" >> '{}'\ncase \"$1\" in\n{}esac\nexit 0\n",
            log.display(),
            crate::test_support::authoring_new_arm()
        ),
    )
    .unwrap();
    let mut perms = std::fs::metadata(&path).unwrap().permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(&path, perms).unwrap();
    Cli::new(path)
}

/// Which verbs the substrate has been run with, in order — the first word of
/// each recorded line.
pub(super) fn verbs(log: &Path) -> Vec<String> {
    lines(log)
        .into_iter()
        .filter_map(|line| line.split_whitespace().next().map(str::to_owned))
        .collect()
}

/// Every line the substrate recorded, whole — read for the payload a `message`
/// carried, which is the claim about *which* held send went out.
pub(super) fn lines(log: &Path) -> Vec<String> {
    std::fs::read_to_string(log)
        .unwrap_or_default()
        .lines()
        .map(str::to_owned)
        .collect()
}

/// A drive standing where the operator stands the instant before Enter: the
/// workspace focused, nothing selected, the composer holding the keyboard, and
/// a substrate that records. Hands back the world, the screen and the log.
pub(super) fn armed(bin: &Path, log: &Path) -> (World, Screen) {
    let mut world = quick(world());
    seed_world(&world);
    let screen = Screen::with_litany(recording_litany(bin, log));
    assert!(screen.idle(&mut world), "the cursor starts in the composer");
    (world, screen)
}

/// What the driver finally writes: the branch, wearing the minted §3.3 name.
pub(super) fn driver_writes(world: &mut World) {
    world.add_root(LANDED, MINTED_FIRST);
    let ws = world.ws.clone();
    world.model.mark_dirty([ws]);
    world.converge();
}

/// Every §11 queue row's header line, in paint order — `✉ ` opens
/// [`crate::inboxview::header_line`] and nothing else on this window paints it,
/// so counting these counts the composer's queue.
fn queue_rows(out: &egui::FullOutput) -> Vec<String> {
    crate::paint_probe::painted_of(out)
        .into_iter()
        .filter(|(text, _)| text.starts_with('✉'))
        .map(|(text, _)| text)
        .collect()
}

/// **Two Enters in the window are one conversation and two messages** — DESIGN
/// §3.4's always-the-second ruling, held through the one stretch in which the
/// second had nowhere to go.
///
/// The send is HELD by yog rather than fired at a name that resolves nowhere,
/// so it reaches the substrate exactly once, addressed by the id the branch
/// brought, and after the start rather than instead of it.
#[test]
fn a_second_enter_in_the_window_is_held_and_delivered_when_the_start_resolves() {
    let bin = tempdir().unwrap();
    let log = bin.path().join("verbs.log");
    let (mut world, screen) = armed(bin.path(), &log);

    say(&screen, &mut world);
    assert_eq!(
        world.model.focus().agent.as_deref(),
        Some(MINTED_FIRST),
        "the start selected what it started, by the minted name"
    );
    assert_eq!(verbs(&log), ["prompt"], "one fire: {:?}", lines(&log));

    // The second Enter, mid-window. It aims at the minted name, which no
    // enumeration answers to — the address that bounced for the whole window.
    typed(&screen, &mut world, AGAIN);
    assert_eq!(
        verbs(&log),
        ["prompt"],
        "nothing was fired at a name that resolves nowhere: {:?}",
        lines(&log)
    );
    let pending = rows_named(&world, MINTED_FIRST);
    assert_eq!(pending.len(), 1, "still exactly one conversation");
    assert_eq!(
        pending[0].root_id, MINTED_FIRST,
        "and one name — the mint was not spent twice"
    );
    let waiting = shot(&screen, &mut world);
    assert_eq!(
        queue_rows(&waiting).len(),
        2,
        "both sends are in the queue above the box, in the operator's own words"
    );

    // The driver writes its branch: the conversation has an id an act can
    // address, and everything held goes out at once.
    driver_writes(&mut world);
    shot(&screen, &mut world);
    assert_eq!(
        verbs(&log),
        ["prompt", "message"],
        "the held send left exactly once, after the start: {:?}",
        lines(&log)
    );
    let sent = lines(&log);
    assert!(
        sent[1].contains(AGAIN) && sent[1].contains(LANDED),
        "addressed by the id its branch brought, carrying what was typed: {sent:?}"
    );
    assert!(
        !sent[1].contains(MINTED_FIRST),
        "and never by the minted name, which is the whole of the fix: {sent:?}"
    );
}

/// **The composer's queue carries the first message from the fire onward**
/// (§7.2's *"the seat the operator named"*).
///
/// It carried nothing for the whole window: the seat asked `Query::Inbox` about
/// the minted name, the refusal was discarded, and the start's own echo was
/// declined here on a premise bl-2e8f had already retired.
#[test]
fn the_queue_carries_the_started_conversations_first_message_at_once() {
    let bin = tempdir().unwrap();
    let log = bin.path().join("verbs.log");
    let (mut world, screen) = armed(bin.path(), &log);
    world.model.select_tab(crate::keymap::InspectorTab::Steps);
    assert!(
        queue_rows(&shot(&screen, &mut world)).is_empty(),
        "nothing is queued before the operator says anything"
    );

    say(&screen, &mut world);
    let out = shot(&screen, &mut world);
    assert_eq!(
        queue_rows(&out).len(),
        1,
        "one row, for the one thing that has been said"
    );
    assert!(
        crate::paint_probe::painted_of(&out)
            .iter()
            .any(|(text, _)| text.contains(SAID)),
        "and it says what was typed"
    );
    assert!(
        super::harness::fills(&out).contains(&super::harness::faded_user()),
        "faded: yog's own word for it, not yet a statement (§11)"
    );
}

/// **A healthy window is not a fault, and must not read as one.** No refusal
/// sentence reaches the glass while the driver is merely still starting, and
/// §8.2's `Stop` is not offered on a conversation whose driver yog has never
/// observed — the row's own state says so, so no seat has to.
#[test]
fn a_healthy_window_paints_no_refusal_and_offers_no_stop() {
    let bin = tempdir().unwrap();
    let log = bin.path().join("verbs.log");
    let (mut world, screen) = armed(bin.path(), &log);
    say(&screen, &mut world);

    for tab in [
        crate::keymap::InspectorTab::Transcript,
        crate::keymap::InspectorTab::Steps,
        crate::keymap::InspectorTab::Inbox,
        crate::keymap::InspectorTab::Files,
        crate::keymap::InspectorTab::Config,
    ] {
        world.model.select_tab(tab);
        let out = shot(&screen, &mut world);
        let said: Vec<String> = crate::paint_probe::painted_of(&out)
            .into_iter()
            .map(|(text, _)| text)
            .filter(|text| text.contains("unknown conversation") || text.contains("ambiguous"))
            .collect();
        assert!(
            said.is_empty(),
            "{tab:?} told the operator their own new conversation was unknown: {said:?}"
        );
    }

    let row = rows_named(&world, MINTED_FIRST);
    assert!(
        !row[0].stoppable,
        "no driver has been observed, so there is none to offer to kill"
    );
    assert!(
        row[0].uncertain,
        "and yog says so: the state is a framing-only reading, flagged"
    );
}
