//! **The Command plane** — §11's combo column, which no text box suppresses.
//! Split from the bare table at §12's budget on the seam the §11 rule already
//! draws: a bare key is a gesture at the selection and yields to a text box, a
//! combo is a gesture at the seat and does not — so the whole safety property
//! that lets combos survive focus is asked here, over the whole alphabet.

use super::super::*;
use super::{bare_press, combo};

/// The §11 combo column, read back: each pairing and the reason its letter
/// either matches its bare key or deliberately does not.
#[test]
fn the_command_plane_pairs_the_safe_gestures() {
    let want = [
        ('i', KeyAction::FocusComposer),
        ('n', KeyAction::NewConversation),
        ('b', KeyAction::ToggleBalls),
        ('g', KeyAction::ToggleGrouping),
        // `a` is select-all's, so the bottom panel takes the bottom-panel combo.
        ('j', KeyAction::ToggleActivity),
    ];
    for (c, action) in want {
        assert_eq!(
            combo(Key::Char(c)),
            Some(action),
            "Ctrl+{c} binds {action:?}"
        );
    }
    for n in 1..=6 {
        let tab = InspectorTab::from_digit(n);
        assert_eq!(combo(Key::Digit(n)), tab.map(KeyAction::Tab));
    }
    assert_eq!(combo(Key::Digit(7)), None, "Ctrl+7 selects no tab");
}
/// The Command+Shift plane carries the two "others" and nothing else: the
/// workspace raise (which Ctrl+W would have letter-matched and must not) and
/// the §11 center-tab strip, whose unshifted digits belong to the inspector.
#[test]
fn command_shift_carries_the_workspace_raise_and_the_center_tabs() {
    assert_eq!(
        keymap(Key::Char('n'), Mods::CommandShift, Held::TextBox),
        Some(KeyAction::NewWorkspace)
    );
    for key in [Key::Char('i'), Key::Char('w'), Key::Enter] {
        assert_eq!(
            keymap(key, Mods::CommandShift, Held::Nothing),
            None,
            "{key:?}"
        );
    }
}

/// §11 rule 3: no combo fires a verb at the current selection, and no combo
/// lands on a key the focused text box owns. This is the safety property that
/// lets combos survive text focus — assert it over the whole alphabet, so a
/// later hand cannot quietly add `Ctrl+S` = Start.
#[test]
fn no_combo_fires_a_selection_verb_or_steals_a_text_box_key() {
    let selection_verbs = [
        KeyAction::StartHead,
        KeyAction::Stop,
        KeyAction::Scan,
        KeyAction::CloseBall,
        KeyAction::ReleaseBall,
        KeyAction::Fire,
        KeyAction::Cancel,
    ];
    for c in 'a'..='z' {
        for mods in [Mods::Command, Mods::CommandShift] {
            let action = keymap(Key::Char(c), mods, Held::Nothing);
            for verb in selection_verbs {
                assert_ne!(action, Some(verb), "{mods:?}+{c} must not fire {verb:?}");
            }
        }
    }
    // The text box's own combos: select-all, undo/redo, and the clipboard three
    // (which egui-winit never even delivers as a key).
    for c in ['a', 'z', 'y', 'c', 'x', 'v'] {
        assert_eq!(
            combo(Key::Char(c)),
            None,
            "Ctrl+{c} belongs to the text box"
        );
    }
    // The three hostile conventions, each unbound on purpose (§11). Ctrl+F is
    // no longer among them: the find reflex has a surface to land on (§8.5),
    // and it is a *query*, so rule 3 is satisfied rather than dodged.
    for c in ['r', 's', 'w'] {
        assert_eq!(
            combo(Key::Char(c)),
            None,
            "Ctrl+{c} binds nothing by design"
        );
    }
    assert_eq!(combo(Key::Char('f')), Some(KeyAction::Search));
    assert_eq!(
        bare_press(Key::Char('f')),
        Some(KeyAction::Scan),
        "the bare plane keeps the mutation the letter always had"
    );
}
/// Text size (§4.1 `zoom`) is the browser convention exactly, and combo-only:
/// bare `+`/`-` are characters an operator is typing, never a gesture.
#[test]
fn zoom_is_the_browser_combo_and_nothing_bare() {
    assert_eq!(combo(Key::Plus), Some(KeyAction::Zoom(ZoomStep::In)));
    assert_eq!(combo(Key::Minus), Some(KeyAction::Zoom(ZoomStep::Out)));
    assert_eq!(combo(Key::Digit(0)), Some(KeyAction::Zoom(ZoomStep::Reset)));
    // `+` is Shift+`=` on most layouts, so the shifted plane means zoom in too.
    assert_eq!(
        keymap(Key::Plus, Mods::CommandShift, Held::Nothing),
        Some(KeyAction::Zoom(ZoomStep::In))
    );
    assert_eq!(
        keymap(Key::Minus, Mods::CommandShift, Held::Nothing),
        None,
        "Ctrl+Shift+- binds nothing"
    );
    for key in [Key::Plus, Key::Minus] {
        assert_eq!(bare_press(key), None, "{key:?} bare is typing, not zoom");
    }
    // The tab digits are untouched by the reset taking 0.
    assert_eq!(
        combo(Key::Digit(1)),
        Some(KeyAction::Tab(InspectorTab::Transcript))
    );
}
