//! **The focus floor** (§11, bl-478d): the frame traverses its own controls
//! with Tab / Shift+Tab and presses the focused one with Space.
//!
//! This is what makes keyboard rule 2 — *"the pointer is never the only
//! path"* — true of **every** control rather than only the ones the binding
//! table names, so the everything-is-keyboard-operable ruling is answered by
//! one invariant instead of a binding per
//! fold, toggle, pin and pick. The table stays what it always was: an
//! accelerator layer over this floor.
//!
//! It is driven through the real window rather than asserted of egui in the
//! abstract, because the claim is about *yog's* frame: a `Sense::click()` seat
//! is focusable, nothing in `shell::keys` swallows Tab or Space, and no modal
//! stands in the way. A regression in any of those three is a regression here.

use super::fixture::world;
use super::screen::{Screen, press};

/// How far the walk may look for a control whose press is visible from the
/// model. Generous on purpose: what is under test is that traversal *reaches*,
/// never how soon it arrives at any particular seat.
const STOPS: usize = 40;

/// One Tab press.
fn tab() -> egui::Event {
    press(egui::Key::Tab, egui::Modifiers::NONE)
}

/// The traversal half: from the bare plane, Tab puts the frame's focus on a
/// control and a second Tab steps on to the next one. That is the whole
/// mechanism — a cursor over the controls that no binding table has to
/// enumerate.
#[test]
fn tab_walks_the_frame_control_by_control() {
    let mut world = world();
    let screen = Screen::new();
    screen.idle(&mut world);
    screen.release(&mut world);
    assert_eq!(screen.focused(), None, "the released plane holds nothing");
    screen.frame(&mut world, vec![tab()]);
    let first = screen.focused();
    assert!(first.is_some(), "Tab put the focus on a control");
    screen.frame(&mut world, vec![tab()]);
    assert_ne!(
        screen.focused(),
        first,
        "a second Tab stepped on to the next control"
    );
}

/// The press half: Space fires whatever the walk has reached. Driven onto the
/// balls fold — a control whose press is a durable §4.1 fact the model answers
/// for, so what is asserted is that a control was really *pressed*, not that a
/// widget lit up. The stop it sits at is nobody's business, so the drive walks
/// until it finds it rather than pinning an index a new control would shift.
#[test]
fn space_presses_the_control_the_walk_reached() {
    let pressed = (1..=STOPS).any(|stop| {
        let mut world = world();
        let screen = Screen::new();
        screen.idle(&mut world);
        screen.release(&mut world);
        let before = world.model.is_collapsed("balls");
        for _ in 0..stop {
            screen.frame(&mut world, vec![tab()]);
        }
        screen.frame(
            &mut world,
            vec![press(egui::Key::Space, egui::Modifiers::NONE)],
        );
        world.model.is_collapsed("balls") != before
    });
    assert!(
        pressed,
        "the keyboard alone never pressed the balls fold within {STOPS} stops — \
         the floor under §11's table is what makes every control operable"
    );
}
