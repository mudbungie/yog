//! The **plane** half of the pure §11 keymap: what a press means as a function
//! of what holds the keyboard ([`Held`]), split from the binding table in
//! [`super`] at the seam the module itself is cut on — `bare`/`command`/
//! `command_shift` are the tables, `bare_plane` is the plane rule over them.

use super::super::*;
use super::combo;

/// The suppression decision, both halves: a text box swallows every bare key,
/// and no combo.
#[test]
fn text_focus_suppresses_bare_keys_only() {
    for key in [
        Key::Up,
        Key::Enter,
        Key::Escape,
        Key::Digit(1),
        Key::Char('r'),
    ] {
        assert!(
            keymap(key, Mods::Bare, Held::Nothing).is_some(),
            "{key:?} binds bare"
        );
        assert_eq!(
            keymap(key, Mods::Bare, Held::TextBox),
            None,
            "{key:?} yields to typing"
        );
    }
    assert_eq!(combo(Key::Char('n')), Some(KeyAction::NewConversation));
    assert_eq!(
        keymap(Key::Char('n'), Mods::Command, Held::Nothing),
        Some(KeyAction::NewConversation),
        "and the same combo works outside a text box"
    );
}

/// bl-d921: a modal owns the frame, so its Escape lands on the **first** press.
/// Under [`Held::TextBox`] the same press is swallowed and egui spends it
/// surrendering focus — that is the two-press dead end the modal used to sit
/// in, where the second Escape reached [`KeyAction::Cancel`] and cancelled a
/// *start goal* underneath instead of the dialog on top.
#[test]
fn a_modal_takes_escape_on_the_first_press_and_it_dismisses_the_modal() {
    assert_eq!(
        keymap(Key::Escape, Mods::Bare, Held::Modal),
        Some(KeyAction::DismissModal)
    );
    assert_eq!(
        keymap(Key::Escape, Mods::Bare, Held::TextBox),
        None,
        "under a plain text box Escape is egui's release gesture"
    );
    assert_ne!(
        keymap(Key::Escape, Mods::Bare, Held::Modal),
        Some(KeyAction::Cancel),
        "the modal on top is what Escape means, never the goal underneath"
    );
}

/// The rest of the bare plane is inert under a modal: nothing beneath it is
/// reachable, by pointer (the backdrop) or by key. The combo planes are
/// untouched — Ctrl+I must still reach the composer from inside the name form.
#[test]
fn a_modal_stops_every_other_bare_key_but_no_combo() {
    for key in [
        Key::Up,
        Key::Down,
        Key::Left,
        Key::Right,
        Key::Enter,
        Key::Digit(1),
        Key::Char('b'),
        Key::Char('x'),
        Key::Char('w'),
    ] {
        assert!(
            keymap(key, Mods::Bare, Held::Nothing).is_some(),
            "{key:?} binds bare with the keyboard free"
        );
        assert_eq!(
            keymap(key, Mods::Bare, Held::Modal),
            None,
            "{key:?} must not reach past a modal"
        );
    }
    assert_eq!(
        keymap(Key::Char('i'), Mods::Command, Held::Modal),
        Some(KeyAction::FocusComposer),
        "Ctrl+I survives a modal exactly as it survives a text box"
    );
    assert_eq!(
        keymap(Key::Char('n'), Mods::CommandShift, Held::Modal),
        Some(KeyAction::NewWorkspace)
    );
}
