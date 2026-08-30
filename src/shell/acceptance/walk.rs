//! **The roster walk under the amended focus discipline** (§11, bl-c21f),
//! driven through the real window: a selection lands the keyboard in the
//! composer whatever plane it rode, and the walk keeps going on the combo plane
//! that text focus does not suppress.
//!
//! Asserted on egui's own `wants_keyboard_input()` — the predicate the
//! suppression rule reads — *and* on the model's selection, because "the
//! composer took the keyboard" and "the step actually moved" are two facts and a
//! drive that checked only the first would pass on a walk that never walked.
//! Split from [`super::focus`], which is the pointer/launch half, at §12's
//! budget. The driver is [`super::screen`].

use super::fixture::{World, world};
use super::screen::{Screen, press};

/// The fixture's second **visible** list row — a second root conversation.
///
/// It was a descent child of `c-1` until bl-fa82: the walk is the visible list
/// now, and a child of a collapsed row is one row, not two, so a fixture whose
/// only second agent was folded away gave the walk nowhere to walk to (the ↓
/// wrapped onto the row it started on and both beats below read as green
/// keyboard plumbing over a walk that never moved). Two roots is the shape that
/// asserts what these beats are about — the *plane*, not the descent — while
/// the unfold's own walk is `acceptance::unfold`'s.
const SECOND: &str = "c-2";

/// A world whose list holds two rows, with the composer already holding the
/// keyboard from launch.
fn walked_world() -> (World, Screen) {
    let mut world = world();
    // Zero the watcher debounce so the root below derives on the very next
    // pass instead of on a wall clock the test would have to sleep against.
    std::fs::write(
        world.model.state_root().join("cadence.yaml"),
        "cadence:\n  watcher:\n    debounce_ms: 0\n",
    )
    .unwrap();
    world.model.after_litany_verb();
    world.converge();
    world.add_root(SECOND, "second-root");
    let ws = world.ws.clone();
    world.model.mark_dirty([ws]);
    world.converge();
    let screen = Screen::new();
    screen.idle(&mut world);
    (world, screen)
}

/// The selected agent id, or `"-"` for a workspace with none selected.
fn selected(world: &World) -> String {
    world
        .model
        .focused_agent()
        .map_or_else(|| "-".to_owned(), |a| a.agent_id.clone())
}

/// The ruling, in both directions: *"When you select an agent, focus to the chat
/// prompt."* ↓ and ↑ are selections, so each ends with the cursor in the box —
/// the rule this replaced left the keyboard on the bare plane instead.
#[test]
fn a_roster_step_lands_the_composer_in_both_directions() {
    for arrow in [egui::Key::ArrowDown, egui::Key::ArrowUp] {
        let (mut world, screen) = walked_world();
        screen.release(&mut world);
        assert_eq!(
            selected(&world),
            "-",
            "the walk starts with nothing selected"
        );
        assert!(
            screen.frame(&mut world, vec![press(arrow, egui::Modifiers::NONE)]),
            "{arrow:?} selected an agent, so the composer took the keyboard"
        );
        assert_ne!(selected(&world), "-", "and the step really did select one");
    }
}

/// The cost the ruling accepts, and its answer. A bare ↓ surrenders the plane it
/// was pressed on, so a second bare ↓ reaches nothing — and Ctrl+↓, which no
/// text box suppresses, is what carries the walk on from inside the box.
#[test]
fn the_walk_continues_on_the_combo_plane() {
    let (mut world, screen) = walked_world();
    screen.release(&mut world);
    screen.frame(
        &mut world,
        vec![press(egui::Key::ArrowDown, egui::Modifiers::NONE)],
    );
    let first = selected(&world);
    assert!(
        screen.frame(
            &mut world,
            vec![press(egui::Key::ArrowDown, egui::Modifiers::NONE)]
        ),
        "the box still holds the keyboard"
    );
    assert_eq!(
        selected(&world),
        first,
        "a second bare ↓ is suppressed — the step spent its own plane"
    );
    assert!(
        screen.frame(
            &mut world,
            vec![press(egui::Key::ArrowDown, egui::Modifiers::COMMAND)]
        ),
        "and Ctrl+↓ leaves the keyboard where the next message gets typed"
    );
    assert_ne!(selected(&world), first, "having stepped the walk on");
}

/// Escape is still the one door back to the bare plane — the more so now that
/// every selection lands the composer. It releases after a keyboard selection,
/// and the plane it releases to is live: a bare ↓ there selects again.
#[test]
fn escape_still_releases_a_keyboard_selection_and_the_bare_plane_is_live() {
    let (mut world, screen) = walked_world();
    screen.release(&mut world);
    screen.frame(
        &mut world,
        vec![press(egui::Key::ArrowDown, egui::Modifiers::NONE)],
    );
    let first = selected(&world);
    screen.release(&mut world);
    assert!(
        !screen.idle(&mut world),
        "and the release holds — nothing re-grabs the box a frame later"
    );
    assert!(
        screen.frame(
            &mut world,
            vec![press(egui::Key::ArrowDown, egui::Modifiers::NONE)]
        ),
        "the bare plane is live again, and its step lands the composer as before"
    );
    assert_ne!(selected(&world), first, "the bare ↓ stepped the walk");
}
