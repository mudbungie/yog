//! The §11 table's spellings, held against the doc's own binding table
//! (bl-478d). [`super::super::spell`] derives them by sweeping [`keymap`], so
//! this test is the lockstep: the list below is DESIGN §11's `key`/`combo`
//! columns transcribed, and a binding added, moved or dropped without the doc
//! following fails here.

use super::super::spell::{bindings, spellings};
use super::super::{KeyAction, ZoomStep};

/// Every spelling §11 offers, in the sweep's order: the bare plane, then
/// Command, then Command+Shift. Letters and digits wear the doc's parentheses;
/// a key that is already a word stands alone.
const TABLE: &[&str] = &[
    "↑",
    "↓",
    "←",
    "→",
    "Enter",
    "Escape",
    "(1)",
    "(2)",
    "(3)",
    "(4)",
    "(5)",
    "(6)",
    "(a)",
    "(b)",
    "(c)",
    "(f)",
    "(g)",
    "(i)",
    "(m)",
    "(n)",
    "(r)",
    "(s)",
    "(w)",
    "(x)",
    "Ctrl+↑",
    "Ctrl+↓",
    "Ctrl+←",
    "Ctrl+→",
    "Ctrl++",
    "Ctrl+-",
    "Ctrl+0",
    "Ctrl+1",
    "Ctrl+2",
    "Ctrl+3",
    "Ctrl+4",
    "Ctrl+5",
    "Ctrl+6",
    "Ctrl+B",
    "Ctrl+F",
    "Ctrl+G",
    "Ctrl+I",
    "Ctrl+J",
    "Ctrl+N",
    "Ctrl+Shift++",
    "Ctrl+Shift+1",
    "Ctrl+Shift+2",
    "Ctrl+Shift+3",
    "Ctrl+Shift+4",
    "Ctrl+Shift+N",
];

#[test]
fn the_vocabulary_is_the_doc_table() {
    assert_eq!(spellings(), TABLE);
}

/// The pairing, spot-checked where §11's prose is most load-bearing: the two
/// meanings of `f` (the bare one mutates, the combo only looks), the walk's
/// continuation, and the zoom's combo-only plane.
#[test]
fn a_spelling_names_the_intent_the_table_binds() {
    let bound = bindings();
    for (press, action) in [
        ("(f)", KeyAction::Scan),
        ("Ctrl+F", KeyAction::Search),
        ("↓", KeyAction::ListNext),
        ("Ctrl+↓", KeyAction::ListNext),
        ("→", KeyAction::ExpandRow),
        ("Ctrl+←", KeyAction::CollapseRow),
        ("(x)", KeyAction::Stop),
        ("Ctrl+Shift+N", KeyAction::NewWorkspace),
        ("Ctrl+0", KeyAction::Zoom(ZoomStep::Reset)),
        ("Escape", KeyAction::Cancel),
    ] {
        assert!(
            bound.contains(&(press.to_owned(), action)),
            "{press} does not spell {action:?}"
        );
    }
}
