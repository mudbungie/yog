//! **The marker as a row** (bl-7bd2) — what the hole this enumeration found
//! says on the glass, split off [`super`] at the cap on the seam that file's own
//! doc draws: finding a hole needs a real directory, saying one needs only the
//! entry the finding produced, so these drive the projection over hand-built
//! markers and never touch disk.
//!
//! The painted assertion goes through `crate::paint_probe`, the one walk.

use std::collections::HashSet;

use crate::transcript::{AutoExpand, Entry, EntryKind, Transcript, rows};

/// A marker as the projection sees it, with the record it carries.
fn mark_entry(first: usize, last: usize, summary: &str) -> Entry {
    Entry {
        name: format!("«{first:03}–{last:03}»"),
        raw: summary.as_bytes().to_vec(),
        kind: EntryKind::Compacted {
            first,
            last,
            summary: summary.to_owned(),
        },
    }
}

/// The row projection of `entries` under the given machinery knob.
fn projected(entries: Vec<Entry>, others: bool) -> Vec<crate::transcript::Row> {
    rows(
        &Transcript { entries },
        "agent",
        AutoExpand {
            responses: true,
            others,
        },
        &HashSet::new(),
    )
}

#[test]
fn the_marker_states_how_many_and_which_and_wears_no_role() {
    let row = projected(vec![mark_entry(1, 12, "gist")], false).remove(0);
    assert_eq!(row.prefix, "✂ 12 entries compacted away — 001–012");
    assert_eq!(row.preview, "gist");
    assert_eq!(row.class, crate::transcript::RowClass::Other);
    assert_eq!(row.tone, crate::transcript::Tone::Weak);
    assert!(row.role.is_none(), "nobody is speaking on a cut mark");
    assert!(
        row.hover.contains(
            "nothing \
             on disk says which summary replaced which span"
        ),
        "the hover states the derivation's own limit: {}",
        row.hover
    );
}

#[test]
fn one_missing_entry_reads_as_one_entry() {
    let row = projected(vec![mark_entry(4, 4, "")], false).remove(0);
    assert_eq!(row.prefix, "✂ 1 entry compacted away — 004");
    assert_eq!(row.preview, "(no summary on this mark)");
}

#[test]
fn a_turn_rollup_never_swallows_the_mark() {
    // A marker is a turn BOUNDARY: were it a step, the collapsed aggregate
    // would hide behind a fold the one row saying the record was rewritten —
    // which is the silence this ball closed.
    let model = |text: &str| Entry {
        name: format!("00x-{text}.json"),
        raw: Vec::new(),
        kind: EntryKind::Model {
            model_id: "opus".to_owned(),
            blocks: vec![
                crate::transcript::Block::Thinking("hmm".to_owned()),
                crate::transcript::Block::Text(text.to_owned()),
            ],
            usage: crate::transcript::Usage::new(),
        },
    };
    let out = projected(
        vec![model("before"), mark_entry(1, 2, "gist"), model("after")],
        false,
    );
    assert!(
        out.iter().any(|r| r.prefix.starts_with('✂')),
        "the mark survived the rollup: {:?}",
        out.iter().map(|r| r.prefix.clone()).collect::<Vec<_>>()
    );
}

#[test]
fn the_mark_and_its_record_reach_the_glass() {
    // Through the paint layer, not the row struct: a galley reports the
    // string that went IN, so only the painted glyphs witness what an
    // operator can actually read.
    let t = Transcript {
        entries: vec![mark_entry(1, 2, "the operator asked about the gate")],
    };
    let painted = super::super::render::painted_with(
        &t,
        false,
        AutoExpand {
            responses: true,
            others: true,
        },
        &mut HashSet::new(),
    );
    assert!(
        painted.contains("✂ 2 entries compacted away — 001–002"),
        "got:\n{painted}"
    );
    assert!(painted.contains("the operator asked about the gate"));
}
