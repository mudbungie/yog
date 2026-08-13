//! The §11 **center tabs** (bl-1ca2), read back off the table: the shifted
//! digit plane they ride, that the unshifted plane still names the altitude-2
//! inspector, and the strip's own list.
//!
//! Its own file at §12's per-file budget, on the same seam [`super::planes`]
//! was split along: the parent tests the whole table, this tests one strip.

use super::super::*;

/// A press on the plane the center tabs ride, while the composer holds the
/// keyboard — which is the state the window actually rests in (the
/// focus ruling), so a binding that only worked with the box released would be
/// no binding at all.
fn shifted(key: Key) -> Option<KeyAction> {
    keymap(key, Mods::CommandShift, Held::TextBox)
}

/// Command+Shift+1–4, each to its tab — and the unshifted digit unchanged, so
/// the two strips never collide.
#[test]
fn command_shift_digits_focus_the_center_tabs() {
    let want = [
        (1, CenterTab::Conversation),
        (2, CenterTab::Config),
        (3, CenterTab::Login),
        (4, CenterTab::Search),
    ];
    for (n, tab) in want {
        assert_eq!(
            shifted(Key::Digit(n)),
            Some(KeyAction::Center(tab)),
            "Ctrl+Shift+{n} focuses {tab:?} from inside the composer"
        );
        assert_eq!(CenterTab::from_digit(n), Some(tab));
        assert_eq!(
            keymap(Key::Digit(n), Mods::Command, Held::Nothing),
            InspectorTab::from_digit(n).map(KeyAction::Tab),
            "the unshifted digit still names the altitude-2 tab"
        );
    }
    for n in [0, 5, 6, 9] {
        assert_eq!(
            CenterTab::from_digit(n),
            None,
            "digit {n} names no center tab"
        );
        assert_eq!(
            shifted(Key::Digit(n)),
            None,
            "Ctrl+Shift+{n} focuses nothing"
        );
    }
    // The bare digit plane is the inspector's, untouched.
    assert_eq!(
        keymap(Key::Digit(2), Mods::Bare, Held::Nothing),
        Some(KeyAction::Tab(InspectorTab::Steps))
    );
}

/// The strip's own list, read back: order, default, labels, and that every tab
/// says what it is (§11 discoverability — a tab cannot ship mute).
#[test]
fn the_center_strip_lists_four_tabs_in_order_and_states_each() {
    let all = CenterTab::all();
    assert_eq!(all.len(), 4);
    assert_eq!(
        all[0],
        CenterTab::default(),
        "the conversation is where the center rests"
    );
    let labels: Vec<&str> = all.iter().map(|t| t.label()).collect();
    assert_eq!(labels, vec!["Conversation", "Config", "Login", "Search"]);
    for tab in all {
        assert!(
            tab.hint().len() > 20,
            "{tab:?} says nothing useful on hover"
        );
        // The one hover every seat that FOCUSES a tab wears (bl-91f1): what the
        // tab is, then the combo that presses it. Both halves derived, so a
        // renamed tab or a re-ordered strip carries its own hover with it.
        let hover = tab.focus_hover();
        assert!(hover.starts_with(tab.hint()), "{tab:?} hides what it shows");
        assert!(
            hover.contains(&format!("Ctrl+Shift+{}", tab.digit())),
            "{tab:?} hides its own combo: {hover}"
        );
    }
    // The digit map and `all` agree: `all[i]` is digit `i + 1`.
    for (i, tab) in all.into_iter().enumerate() {
        assert_eq!(CenterTab::from_digit(i as u8 + 1), Some(tab));
    }
}
