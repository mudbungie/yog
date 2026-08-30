//! **One message is one queue row** (§7.2, bl-78d8), driven through the real
//! window — the half of [`super`]'s drive that watches the echo *give way*
//! rather than appear, split off at §12's budget on that seam.
//!
//! Operator report: a sent message shows as a visible duplicate in the §11
//! inbox-composer queue — the faded echo and the solid deposit, same words, side
//! by side, for about a second. §7.2 rules the opposite outright: *"brightening
//! is that same row at full strength, not a repaint into different hues."*
//!
//! Neither beat below could be written before this ball, and that gap **was**
//! the defect: no test had ever held an echo alive while its own deposit file
//! existed on disk. The shared fake `litany` writes nothing, so every earlier
//! beat watched the echo against an empty inbox — the one arrangement in which
//! appending unconditionally is indistinguishable from yielding.

use super::super::fixture::{World, fake_litany, seed_world, world};
use super::super::inbox_composer::{deposit, drain};
use super::super::screen::Screen;
use super::{SAID, converge_ws, faded_user, fills, quick, say, shot};
use crate::cli_outbound::Cli;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use tempfile::tempdir;

/// Every §11 queue row's header line, in paint order. `✉ ` opens
/// [`crate::inboxview::header_line`] and nothing else on this window paints it,
/// so counting these counts the queue — which is the claim both beats make, and
/// a claim no assertion over the row's *words* can make (the echo and the
/// deposit say the same words, which is exactly why they were hard to tell
/// apart on the glass).
fn queue_rows(out: &egui::FullOutput) -> Vec<String> {
    crate::paint_probe::painted_of(out)
        .into_iter()
        .filter(|(text, _)| text.starts_with('✉'))
        .map(|(text, _)| text)
        .collect()
}

/// How many painted galleys carry the operator's words.
fn said(out: &egui::FullOutput) -> usize {
    crate::paint_probe::painted_of(out)
        .into_iter()
        .filter(|(text, _)| text.contains(SAID))
        .count()
}

/// A `litany` whose **`message` arm writes the deposit**, the way the real one
/// does: the §8.2 verb is piped and run to completion, so the file is on disk
/// before the receipt that mints the echo. Every other verb exits 0.
fn depositing_litany(dir: &Path, ws: &Path) -> Cli {
    let path = dir.join("litany");
    let inbox = ws.join("inbox/c-1");
    std::fs::write(
        &path,
        format!(
            "#!/bin/sh\ncase \"$1\" in\nmessage) mkdir -p '{}' && \
             printf -- '---\\nfrom: user\\ndeposited_at: t1\\n---\\n%s\\n' \"$4\" \
             > '{}/user-002.md';;\nesac\nexit 0\n",
            inbox.display(),
            inbox.display()
        ),
    )
    .unwrap();
    let mut perms = std::fs::metadata(&path).unwrap().permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(&path, perms).unwrap();
    Cli::new(path)
}

/// The fixture world with c-1 selected on a tab that paints no message bodies,
/// so `✉` and [`SAID`] can only have come from the queue above the box.
fn queued() -> World {
    let mut world = quick(world());
    seed_world(&world);
    let ws = world.ws.clone();
    world.model.focus_agent(&ws, "c-1");
    world.model.select_tab(crate::keymap::InspectorTab::Steps);
    world
}

/// **The whole lifecycle, one row throughout.** Faded while the deposit is only
/// yog's word for it, solid the instant the listing carries the file, gone when
/// the delivery commit takes it — never two rows for the one message.
#[test]
fn one_message_is_one_queue_row_faded_then_solid_then_gone() {
    let bin = tempdir().unwrap();
    let mut world = queued();
    let screen = Screen::with_litany(fake_litany(bin.path()));
    let before = shot(&screen, &mut world);
    assert_eq!(
        queue_rows(&before),
        ["✉ user · t0"],
        "the fixture's one landed deposit, and nothing else"
    );

    say(&screen, &mut world);
    // This fake writes nothing, so the deposit really is still unwritten — the
    // pre-disk half, asserted rather than assumed.
    let inbox = world.ws.join("inbox/c-1");
    assert_eq!(
        std::fs::read_dir(&inbox).map_or(0, |d| d.flatten().count()),
        1,
        "no file yet: whatever the queue shows is yog's own word"
    );
    let pending = shot(&screen, &mut world);
    assert_eq!(queue_rows(&pending).len(), 2, "the echo took a seat");
    assert_eq!(said(&pending), 1, "and says what was typed, once");
    assert!(
        fills(&pending).contains(&faded_user()),
        "faded: not yet a statement (§11)"
    );

    // The substrate writes it. The listing the seat asks for grows by one, and
    // that — a count, never the text — is the echo's cue to give the seat up.
    deposit(&world, "user-002.md", "t1", SAID);
    converge_ws(&mut world);
    let landed = shot(&screen, &mut world);
    assert_eq!(
        queue_rows(&landed),
        ["✉ user · t0", "✉ user · t1"],
        "still two rows: the deposit took the echo's seat rather than a new one"
    );
    assert_eq!(
        said(&landed),
        1,
        "the operator's words are on the glass once"
    );
    assert!(
        !fills(&landed).contains(&faded_user()),
        "and at full strength: that same row, brightened (§7.2)"
    );

    // The delivery commit: the inbox drains and the message becomes transcript.
    drain(&world);
    let messages = world.ws.join("agents/c-1/messages");
    std::fs::write(messages.join("003-user.md"), SAID).unwrap();
    converge_ws(&mut world);
    let delivered = shot(&screen, &mut world);
    assert!(
        queue_rows(&delivered).is_empty(),
        "the queue is empty: {:?}",
        queue_rows(&delivered)
    );
}

/// **The ordering the operator actually hit.** The §8.2 verb is piped, so the
/// deposit is on disk *before* the receipt mints the echo — the arrangement no
/// beat had ever set up, and the one in which appending unconditionally paints
/// the duplicate. Here the fake substrate does the write, so nothing about the
/// timing is the test's invention.
#[test]
fn the_substrates_own_write_never_earns_a_second_row() {
    let bin = tempdir().unwrap();
    let mut world = queued();
    let ws = world.ws.clone();
    // The substrate's own write, seated the way every acceptance fake is: the
    // screen hands its binaries to the world on every frame it runs.
    let screen = Screen::with_litany(depositing_litany(bin.path(), &ws));
    shot(&screen, &mut world);

    say(&screen, &mut world);
    assert_eq!(
        std::fs::read_dir(ws.join("inbox/c-1")).map_or(0, |d| d.flatten().count()),
        2,
        "the piped verb wrote its deposit before the receipt came back"
    );
    converge_ws(&mut world);
    let out = shot(&screen, &mut world);
    assert_eq!(
        queue_rows(&out),
        ["✉ user · t0", "✉ user · t1"],
        "two deposits, two rows — the echo never earned a third"
    );
    assert_eq!(said(&out), 1, "and one row carries what was typed");
}
