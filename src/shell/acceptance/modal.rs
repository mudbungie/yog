//! **A modal owns the frame** (§11, bl-d921), driven end to end through the
//! real window: Escape dismisses the §3.1 name form on the first press and
//! mints nothing, a click at a coordinate that demonstrably works reaches
//! nothing while the form is up, and Return submits the name the form is for.
//!
//! Both halves are one invariant with one predicate behind it
//! ([`super::super::modal::open`]) — the keyboard half through the pure
//! [`Held::Modal`](crate::keymap::Held::Modal) plane, the pointer half through
//! the backdrop's layer. The driver is [`super::screen`].

use super::fixture::{World, world};
use super::screen::{Screen, click, locate, press};
use crate::keymap::CenterTab;

/// Escape dismisses the form **on the first press**, drops the draft, and mints
/// nothing. Before bl-d921 no number of presses closed it: the form's own text
/// box swallowed the first, and the second reached `KeyAction::Cancel`, which
/// only ever spoke to the §8.1 pending start goal underneath.
#[test]
fn escape_dismisses_the_new_workspace_form_and_mints_nothing() {
    let mut world = world();
    let screen = Screen::new();
    screen.idle(&mut world);
    // The bare-key plane: `w` is a bare key, so the composer has to let go
    // first — the §11 `Escape` then key idiom, exactly as an operator does it.
    screen.release(&mut world);
    let before = spheres(&world);

    screen.frame(&mut world, vec![press(egui::Key::W, egui::Modifiers::NONE)]);
    assert!(world.state.new_ws.open, "bare `w` opens the §11 name form");
    // A draft to lose — dismissal is the operator saying no, and §5.3 keeps
    // unsubmitted input in RAM precisely so it can die with the surface.
    world.state.new_ws.typed = "ops".to_owned();

    assert!(
        screen.frame(
            &mut world,
            vec![press(egui::Key::Escape, egui::Modifiers::NONE)]
        ),
        "the dismissing frame hands the keyboard straight back to the composer: \
         the keys are lifted before any panel paints, so unlike the window's \
         own close button there is no frame in between"
    );
    assert!(!world.state.new_ws.open, "one Escape closes it");
    assert!(
        world.state.new_ws.typed.is_empty(),
        "and the draft dies with it"
    );
    assert_eq!(spheres(&world), before, "no sphere wall was raised");
    assert!(screen.idle(&mut world), "and the keyboard stays in the box");
}

/// The pointer half, at the exact seat the bug was reported against: the left
/// panel's `⚙ Config` entry, whose pane opened *under* the open form.
///
/// Asserted in **both directions**, because an assertion that a click did
/// nothing is worthless unless the same click demonstrably does something: the
/// coordinate focuses the Config tab and the strip brings the center home
/// again with no modal up, then the same coordinate reaches nothing at all
/// with the form open. Since bl-1ca2 the entry is a tab **focus**, so the way
/// back is another tab, never the same entry pressed twice.
#[test]
fn a_click_beneath_the_open_form_reaches_nothing() {
    let mut world = world();
    let screen = Screen::new();
    screen.idle(&mut world);
    screen.release(&mut world);
    let seat = locate(&screen.shapes(&mut world, Vec::new()), "⚙ Config")
        .expect("the Config entry reaches the paint layer");

    click(&screen, &mut world, seat);
    assert_eq!(
        world.state.center,
        CenterTab::Config,
        "the coordinate really is the Config entry"
    );
    let home = locate(&screen.shapes(&mut world, Vec::new()), "Conversation")
        .expect("the center strip's home tab reaches the paint layer");
    click(&screen, &mut world, home);
    assert_eq!(
        world.state.center,
        CenterTab::Conversation,
        "and the strip is the way back — a tab focus is left by focusing another"
    );

    screen.frame(&mut world, vec![press(egui::Key::W, egui::Modifiers::NONE)]);
    assert!(world.state.new_ws.open, "the form is up");
    click(&screen, &mut world, seat);
    assert_eq!(
        world.state.center,
        CenterTab::Conversation,
        "the click must not reach the panel beneath the modal"
    );
    assert!(
        world.state.new_ws.open,
        "and the form is still standing — the backdrop swallowed the click \
         rather than being hoisted above the dialog it sits under"
    );
}

/// Return submits a valid name, and does nothing at all with an invalid one
/// (bl-d921's third finding: the pointer on `Create workspace` was the only way
/// through). The defect was one line of ordering — the form's re-claim of the
/// keyboard ran *before* the `lost_focus()` read, handing focus back on the very
/// frame Enter surrendered it and making the read false forever.
///
/// The raise itself is read off the §8.1 planner's first mutating step, because
/// the driver deliberately names binaries that do not exist ([`Screen::new`]):
/// the point is that Enter reached the planner with the typed name, not what
/// `lernie` did next.
#[test]
fn return_submits_a_valid_name_and_a_refused_one_keeps_the_form() {
    let mut world = world();
    let screen = Screen::new();
    screen.idle(&mut world);
    screen.release(&mut world);

    // A name §3.1 refuses: Enter must not submit, and must not close the form
    // over the operator's half-typed word either.
    screen.frame(&mut world, vec![press(egui::Key::W, egui::Modifiers::NONE)]);
    world.state.new_ws.typed = "Not A Sphere".to_owned();
    screen.frame(
        &mut world,
        vec![press(egui::Key::Enter, egui::Modifiers::NONE)],
    );
    assert!(world.state.new_ws.open, "a refused Enter keeps the form up");
    assert_eq!(
        world.state.new_ws.typed, "Not A Sphere",
        "with what was typed still in it"
    );
    world.converge();
    assert!(
        world
            .model
            .last_failure(crate::opslog::Origin::Conversation)
            .is_none(),
        "and nothing was dispatched"
    );

    // The same key on a name that validates goes all the way through.
    world.state.new_ws.typed = "ops".to_owned();
    screen.frame(
        &mut world,
        vec![press(egui::Key::Enter, egui::Modifiers::NONE)],
    );
    assert!(
        !world.state.new_ws.open,
        "Enter submits and closes the form"
    );
    world.converge();
    let raise = world
        .model
        .last_failure(crate::opslog::Origin::Conversation)
        .expect("Enter reached the §8.1 planner");
    assert!(
        raise.argv.ends_with(" prime"),
        "the raise begins at the world seed: {}",
        raise.argv
    );
}

/// The sphere walls on disk — what "no workspace minted" is read against.
fn spheres(world: &World) -> Vec<std::ffi::OsString> {
    let root = world.ws.parent().expect("the workspaces root");
    let mut names: Vec<std::ffi::OsString> = std::fs::read_dir(root)
        .expect("the workspaces root is readable")
        .map(|e| e.expect("a directory entry").file_name())
        .collect();
    names.sort();
    names
}
