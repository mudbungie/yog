//! Table tests for the pure §11 keymap: every key maps to its intent, every
//! digit to its tab (or nothing), every combo to its pair, and the suppression
//! rule (bare keys yield to a text box, combos do not) holds on both planes.
//!
//! The plane rule itself — what each of [`Held`]'s three answers does to the
//! bare plane — is [`planes`], its own file at the module's own seam; the §11
//! center-tab strip and the plane it rides are [`center`]'s; the Command
//! column itself — every pairing, and the rule that no combo fires a selection
//! verb or steals a text box's key — is [`combos`]'.

mod center;
mod combos;
mod planes;
mod spell;

use super::*;

/// A bare press with no text box holding the keyboard — the §11 table read
/// straight.
pub(super) fn bare_press(key: Key) -> Option<KeyAction> {
    keymap(key, Mods::Bare, Held::Nothing)
}

/// A Command-plane press (⌘ on macOS, Ctrl elsewhere) while typing — the case
/// combos exist for.
pub(super) fn combo(key: Key) -> Option<KeyAction> {
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
