//! Table tests for the pure §11 keymap: every key maps to its intent, every
//! digit to its tab (or nothing), every combo to its pair, and the suppression
//! rule (bare keys yield to a text box, combos do not) holds on both planes.
//!
//! The plane rule itself — what each of [`Held`]'s three answers does to the
//! bare plane — is [`planes`], its own file at the module's own seam; the §11
//! center-tab strip and the plane it rides are [`center`]'s.

mod center;
mod planes;
mod spell;

use super::*;

/// A bare press with no text box holding the keyboard — the §11 table read
/// straight.
fn bare_press(key: Key) -> Option<KeyAction> {
    keymap(key, Mods::Bare, Held::Nothing)
}

/// A Command-plane press (⌘ on macOS, Ctrl elsewhere) while typing — the case
/// combos exist for.
fn combo(key: Key) -> Option<KeyAction> {
    keymap(key, Mods::Command, Held::TextBox)
}

#[test]
fn vertical_arrows_step_the_visible_list() {
    assert_eq!(bare_press(Key::Up), Some(KeyAction::ListPrev));
    assert_eq!(bare_press(Key::Down), Some(KeyAction::ListNext));
}

/// The unfold's own axis (bl-fa82): the horizontal arrows fold the row the
/// selection already names, and they mean the same thing on the combo plane —
/// the pairing exists for the same reason the walk's does.
#[test]
fn horizontal_arrows_unfold_the_selected_row_on_both_planes() {
    assert_eq!(bare_press(Key::Right), Some(KeyAction::ExpandRow));
    assert_eq!(bare_press(Key::Left), Some(KeyAction::CollapseRow));
    assert_eq!(combo(Key::Right), Some(KeyAction::ExpandRow));
    assert_eq!(combo(Key::Left), Some(KeyAction::CollapseRow));
    assert_eq!(
        keymap(Key::Right, Mods::Bare, Held::TextBox),
        None,
        "the bare arrow stays out of the caret's way — that is why the combo exists"
    );
}

/// The walk's continuation (bl-c21f): a selection now lands the composer
/// whatever plane it rode, so the bare step spends its own plane — and the
/// combo, which no text box suppresses, is what carries the second step. The
/// two planes mean the same thing, which is the point.
#[test]
fn ctrl_arrows_continue_the_walk_from_inside_the_box() {
    assert_eq!(combo(Key::Up), Some(KeyAction::ListPrev));
    assert_eq!(combo(Key::Down), Some(KeyAction::ListNext));
    assert_eq!(
        keymap(Key::Up, Mods::Bare, Held::TextBox),
        None,
        "the bare arrow still yields to typing — that is why the combo exists"
    );
    for key in [Key::Up, Key::Down] {
        assert_eq!(
            keymap(key, Mods::CommandShift, Held::Nothing),
            None,
            "{key:?} rides the plain Command plane only"
        );
    }
}

#[test]
fn enter_fires_the_pending_goal_and_escape_cancels_it() {
    assert_eq!(bare_press(Key::Enter), Some(KeyAction::Fire));
    assert_eq!(bare_press(Key::Escape), Some(KeyAction::Cancel));
}

/// The §11 letter table: one key per gesture, each mnemonic and collision-free
/// (the pairs below are the doc's table, read back).
#[test]
fn letters_fire_the_altitude_gestures() {
    let want = [
        ('i', KeyAction::FocusComposer),
        ('n', KeyAction::NewConversation),
        ('w', KeyAction::NewWorkspace),
        ('s', KeyAction::StartHead),
        ('x', KeyAction::Stop),
        ('f', KeyAction::Scan),
        ('c', KeyAction::CloseBall),
        ('r', KeyAction::ReleaseBall),
        ('b', KeyAction::ToggleBalls),
        ('m', KeyAction::ToggleModelPicker),
        ('g', KeyAction::ToggleGrouping),
        ('a', KeyAction::ToggleActivity),
    ];
    for (c, action) in want {
        assert_eq!(
            bare_press(Key::Char(c)),
            Some(action),
            "`{c}` binds {action:?}"
        );
    }
    // No two letters share an intent, and no letter shadows the arrows/digits.
    let actions: Vec<KeyAction> = want.iter().map(|(_, a)| *a).collect();
    for (i, a) in actions.iter().enumerate() {
        assert_eq!(
            actions.iter().filter(|b| *b == a).count(),
            1,
            "intent {a:?} is bound once (entry {i})"
        );
    }
}

#[test]
fn unbound_letters_map_to_nothing() {
    for c in ['q', 'z', 'e', 'v', 'k', 'j', 'I', 'N', 'M'] {
        assert_eq!(bare_press(Key::Char(c)), None, "`{c}` binds nothing");
    }
}

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

#[test]
fn digits_one_through_six_select_the_inspector_tabs() {
    let want = [
        (1, InspectorTab::Transcript),
        (2, InspectorTab::Steps),
        (3, InspectorTab::Inbox),
        (4, InspectorTab::Files),
        (5, InspectorTab::Config),
        (6, InspectorTab::Work),
    ];
    for (n, tab) in want {
        assert_eq!(bare_press(Key::Digit(n)), Some(KeyAction::Tab(tab)));
        assert_eq!(InspectorTab::from_digit(n), Some(tab));
    }
}

#[test]
fn digits_outside_one_through_six_are_unbound() {
    assert_eq!(InspectorTab::from_digit(0), None);
    for n in 7..=9 {
        assert_eq!(bare_press(Key::Digit(n)), None, "digit {n} binds no tab");
    }
    assert_eq!(bare_press(Key::Digit(0)), None, "bare 0 is typing");
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

#[test]
fn all_lists_the_six_tabs_in_order_and_labels_each() {
    let all = InspectorTab::all();
    assert_eq!(all.len(), 6);
    assert_eq!(all[0], InspectorTab::default(), "Transcript is the default");
    let labels: Vec<&str> = all.iter().map(|t| t.label()).collect();
    assert_eq!(
        labels,
        vec!["Transcript", "Steps", "Inbox", "Files", "Config", "Work"]
    );
    // The pin is a conversation-repo commit, so it reaches every tab but the
    // one whose subject is the project repo.
    let pinnable: Vec<&str> = all
        .iter()
        .filter(|t| t.pinnable())
        .map(|t| t.label())
        .collect();
    assert_eq!(
        pinnable,
        vec!["Transcript", "Steps", "Inbox", "Files", "Config"]
    );
    // The digit map and `all` agree: `all[i]` is digit `i + 1`.
    for (i, tab) in all.into_iter().enumerate() {
        assert_eq!(InspectorTab::from_digit(i as u8 + 1), Some(tab));
    }
}
