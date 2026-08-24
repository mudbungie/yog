//! **Which rows arrive open** (§11, §4.1) — the derived auto-state, the two
//! knobs that invert it and the per-row override that flips one of them. Split
//! off [`super`] at the cap on the seam that file's doc lists: what a row *is*
//! is read off the entry's bytes, and this is the one thing about a row that is
//! **policy** — config a knob sets, never code.

use super::{SPEAKER, default_rows, entry, model, tx};
use crate::transcript::{AutoExpand, Block, EntryKind, RowClass, Tone, rows};
use std::collections::HashSet;

#[test]
fn responses_auto_expand_and_everything_else_auto_contracts() {
    let t = tx(vec![
        model(vec![Block::Text("line one\nline two".into())]),
        entry(
            "003-tool.json",
            EntryKind::ToolResult {
                tool_use_id: "t".into(),
                content: "out one\nout two".into(),
                is_error: true,
            },
        ),
    ]);
    let got = default_rows(&t);
    assert_eq!(got[0].class, RowClass::Response);
    assert!(got[0].expanded, "a reply arrives expanded: {got:?}");
    assert_eq!(got[1].class, RowClass::Other);
    assert!(!got[1].expanded, "machinery arrives contracted: {got:?}");
    assert_eq!(got[1].tone, Tone::Bad, "an error result paints ichor");
}

#[test]
fn both_automatics_are_knobs() {
    let t = tx(vec![
        model(vec![Block::Text("a\nb".into())]),
        entry(
            "003-tool.json",
            EntryKind::ToolResult {
                tool_use_id: "t".into(),
                content: "c\nd".into(),
                is_error: false,
            },
        ),
    ]);
    // Inverted knobs invert both automatics — the policy is config, not code.
    let inverted = AutoExpand {
        responses: false,
        others: true,
    };
    let got = rows(&t, SPEAKER, inverted, &HashSet::new());
    assert!(!got[0].expanded, "responses knob off: {got:?}");
    assert!(got[1].expanded, "others knob on: {got:?}");
}

#[test]
fn an_override_flips_that_rows_auto_state_only() {
    let t = tx(vec![model(vec![
        Block::Text("a\nb".into()),
        Block::Thinking("c\nd".into()),
    ])]);
    let mut folds = HashSet::new();
    folds.insert("tx/002-opus.json#0".to_string());
    let got = rows(&t, SPEAKER, AutoExpand::default(), &folds);
    assert!(
        !got[0].expanded,
        "the override contracts the reply: {got:?}"
    );
    assert!(!got[1].expanded, "its neighbour keeps its auto-state");
    // The same override on a contracted row expands it (the flip is symmetric).
    let mut other = HashSet::new();
    other.insert("tx/002-opus.json#1".to_string());
    let got = rows(&t, SPEAKER, AutoExpand::default(), &other);
    assert!(got[0].expanded);
    assert!(
        got[1].expanded,
        "the override expands the thinking: {got:?}"
    );
}
