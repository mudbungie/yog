//! **Inside the gap between an act's post and its receipt** (bl-56c6) — the
//! stretch [`Screen::unsettled`](super::super::screen::Screen::unsettled) is the
//! only way into, split off [`super`] at §12's per-file budget on the seam that
//! file's own doc draws: everything there is about *where a send goes* during
//! the §3.4 window, and this is about what the box and the mint do while the
//! engine has not answered yet.
//!
//! The §8.1 start is two posted acts and the composer is never disabled across
//! them, so both defects here live entirely in that gap: text typed into a box
//! whose send has not landed, and a second Enter arriving before the first one
//! has.

use super::super::super::screen::press;
use super::super::harness::{MINTED_FIRST, SAID, rows_named, shot};
use super::{AGAIN, LANDED, armed, driver_writes, lines, verbs};
use crate::actions::DraftKey;
use tempfile::tempdir;

/// **The draft is the operator's, through every spelling the conversation
/// wears.** Text typed while the fire is in flight survives its receipt, and
/// the buffer travels with the conversation as it goes from *no target* to the
/// minted name to the id its branch brought.
///
/// Driven from inside the gap between the post and the receipt
/// ([`Screen::unsettled`]) — the only place the defect exists, and the reason
/// no beat had ever reached it.
#[test]
fn a_draft_survives_the_receipt_and_every_re_keying_of_its_conversation() {
    let bin = tempdir().unwrap();
    let log = bin.path().join("verbs.log");
    let (mut world, screen) = armed(bin.path(), &log);

    // Enter, and then keep typing — the box is never disabled, so this is what
    // an operator who carries on actually does.
    screen.unsettled(&mut world, vec![egui::Event::Text(SAID.to_owned())]);
    screen.unsettled(
        &mut world,
        vec![press(egui::Key::Enter, egui::Modifiers::NONE)],
    );
    screen.unsettled(&mut world, vec![egui::Event::Text(AGAIN.to_owned())]);
    assert!(
        verbs(&log).is_empty(),
        "still inside the gap: the engine has not answered yet"
    );

    // The receipts land. What was sent leaves the box; what was typed after it
    // does not, and it is in the started conversation's own box now.
    shot(&screen, &mut world);
    assert_eq!(verbs(&log), ["prompt"]);
    let named = DraftKey::composer(Some(world.ws.clone()), Some(MINTED_FIRST.to_owned()));
    assert_eq!(
        world.state.actions.drafts.text(&named),
        AGAIN,
        "the fired words left and the rest stayed, in the box that is now the \
         started conversation's"
    );
    assert!(
        crate::paint_probe::painted_of(&shot(&screen, &mut world))
            .iter()
            .any(|(text, _)| text.contains(AGAIN)),
        "and it is on the glass, where they left it"
    );

    // The branch lands and the conversation swaps its name for an id.
    driver_writes(&mut world);
    shot(&screen, &mut world);
    assert_eq!(
        world.state.actions.drafts.text(&DraftKey::composer(
            Some(world.ws.clone()),
            Some(LANDED.to_owned())
        )),
        AGAIN,
        "one buffer, carried across the swap rather than stranded under the name"
    );
    assert_eq!(
        world.state.actions.drafts.text(&named),
        "",
        "and not left behind under the spelling nothing points at any more"
    );
}

/// **A second Enter while the fire itself is still in flight starts nothing.**
///
/// The §8.1 start is two acts and the §3.4 claim is taken on the second one's
/// receipt, so an Enter in between used to replace the hold: the first
/// `Prompt`'s aftermath never ran while its detached driver launched anyway,
/// and the replacement chained a second `Prompt` **with the same unspent §3.3
/// seed against the same occupied set** — one name, two roots, ambiguous
/// forever. Nothing is lost by refusing: the words stay in the box.
#[test]
fn a_double_enter_before_the_fire_lands_mints_one_name_and_not_two() {
    let bin = tempdir().unwrap();
    let log = bin.path().join("verbs.log");
    let (mut world, screen) = armed(bin.path(), &log);

    screen.unsettled(&mut world, vec![egui::Event::Text(SAID.to_owned())]);
    for _ in 0..2 {
        screen.unsettled(
            &mut world,
            vec![press(egui::Key::Enter, egui::Modifiers::NONE)],
        );
    }
    shot(&screen, &mut world);

    assert_eq!(
        verbs(&log),
        ["prompt"],
        "one fire, one mint: {:?}",
        lines(&log)
    );
    assert_eq!(
        rows_named(&world, MINTED_FIRST).len(),
        1,
        "and one row wearing the one name"
    );
    assert_eq!(
        world.model.focus().agent.as_deref(),
        Some(MINTED_FIRST),
        "the claim the first fire made is the one that stands"
    );
}
