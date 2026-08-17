//! What the compactor deleted, and what it left in its place (bl-7bd2).
//!
//! Every test here drives [`build`] over a real directory, because the defect
//! was that `messages/` is a directory something else **writes to and deletes
//! from** — a hand-built `Vec<Entry>` cannot exhibit it. The painted assertion
//! goes through `crate::paint_probe`, the one walk.

use std::collections::HashSet;

use super::{AGENT, write_msg, write_summary};
use crate::transcript::{AutoExpand, Entry, EntryKind, Transcript, build, rows};
use tempfile::tempdir;

/// A workspace holding exactly these message files, read back as a transcript.
fn listing(names: &[&str]) -> (tempfile::TempDir, Transcript) {
    let dir = tempdir().unwrap();
    for n in names {
        write_msg(dir.path(), n, b"hi\n");
    }
    let t = build(dir.path(), AGENT);
    (dir, t)
}

/// Every compaction marker in a transcript, as `(name, first, last, summary)`.
fn marks(t: &Transcript) -> Vec<(String, usize, usize, String)> {
    t.entries
        .iter()
        .filter_map(|e| match &e.kind {
            EntryKind::Compacted {
                first,
                last,
                summary,
            } => Some((e.name.clone(), *first, *last, summary.clone())),
            _ => None,
        })
        .collect()
}

#[test]
fn a_first_entry_above_one_marks_the_head_that_is_gone() {
    // The sighted shape: the reply survives, the question that provoked it
    // does not. `001` never being on disk is the only evidence there is.
    let (_d, t) = listing(&["002-opus.json", "003-user.md"]);
    assert_eq!(marks(&t), vec![("«001»".to_owned(), 1, 1, String::new())]);
    // Seated *before* the surviving head, not appended anywhere convenient.
    assert_eq!(t.entries[0].name, "«001»");
    assert_eq!(t.entries[1].name, "002-opus.json");
}

#[test]
fn a_discontinuity_mid_sequence_marks_the_whole_span() {
    let (_d, t) = listing(&["001-user.md", "005-opus.json", "006-user.md"]);
    assert_eq!(
        marks(&t),
        vec![("«002–004»".to_owned(), 2, 4, String::new())]
    );
    assert_eq!(t.entries[1].name, "«002–004»");
}

#[test]
fn a_contiguous_listing_is_returned_untouched() {
    // The general path with no hole in it — no marker, and the entries are
    // the same objects in the same order they were read.
    let (_d, t) = listing(&["001-user.md", "002-opus.json", "003-tool.json"]);
    assert!(marks(&t).is_empty());
    let names: Vec<&str> = t.entries.iter().map(|e| e.name.as_str()).collect();
    assert_eq!(names, ["001-user.md", "002-opus.json", "003-tool.json"]);
}

#[test]
fn the_record_opens_from_the_first_mark_and_no_later_one() {
    // Two holes, two summaries, and NO on-disk fact pairing them. The whole
    // record rides the earliest mark in pass order; the later mark carries
    // the counter's finding alone. Nothing here asserts which summary
    // replaced which span, because nothing on disk says.
    let dir = tempdir().unwrap();
    for n in ["003-user.md", "007-opus.json"] {
        write_msg(dir.path(), n, b"hi\n");
    }
    write_summary(dir.path(), "001.md", "the first pass cut this");
    write_summary(dir.path(), "002.md", "the second pass cut that");
    assert_eq!(
        marks(&build(dir.path(), AGENT)),
        vec![
            (
                "«001–002»".to_owned(),
                1,
                2,
                "the first pass cut this\n\nthe second pass cut that".to_owned()
            ),
            ("«004–006»".to_owned(), 4, 6, String::new()),
        ]
    );
}

#[test]
fn a_gap_with_no_summary_at_all_still_marks_the_span() {
    // The marker never depends on a summary existing: a compaction whose
    // prose yog cannot read is still a compaction, and saying so is the
    // honest answer.
    let (_d, t) = listing(&["004-user.md"]);
    assert_eq!(
        marks(&t),
        vec![("«001–003»".to_owned(), 1, 3, String::new())]
    );
}

#[test]
fn the_summary_directory_yields_only_its_readable_md_files() {
    let dir = tempdir().unwrap();
    write_msg(dir.path(), "003-user.md", b"hi\n");
    write_summary(dir.path(), "001.md", "kept");
    write_summary(dir.path(), "002.txt", "not a summary");
    std::fs::create_dir_all(
        dir.path()
            .join("agents")
            .join(AGENT)
            .join("summary")
            .join("003.md"),
    )
    .unwrap();
    // Lossy, not dropped: mangled prose still says the record was rewritten.
    std::fs::write(
        dir.path()
            .join("agents")
            .join(AGENT)
            .join("summary")
            .join("004.md"),
        [b'o', b'k', 0xff],
    )
    .unwrap();
    assert_eq!(
        marks(&build(dir.path(), AGENT))[0].3,
        "kept\n\nok\u{fffd}",
        "only the .md FILES, in counter order"
    );
}

#[test]
fn an_entry_carrying_no_counter_neither_opens_nor_closes_a_hole() {
    // `002x.md` is the Raw bucket — no `NNN-<origin>` shape, so it says
    // nothing about which counter values are missing, and the gap either
    // side of it is still the one the surviving numbers prove.
    let (_d, t) = listing(&["001-user.md", "002x.md", "003-user.md"]);
    assert_eq!(marks(&t), vec![("«002»".to_owned(), 2, 2, String::new())]);
    assert_eq!(t.entries[2].name, "«002»");
    assert_eq!(t.entries[3].name, "003-user.md");
}

#[test]
fn a_counter_no_usize_can_hold_is_skipped_rather_than_believed() {
    let (_d, t) = listing(&["001-user.md", "003-user.md", "99999999999999999999-user.md"]);
    assert_eq!(marks(&t), vec![("«002»".to_owned(), 2, 2, String::new())]);
}

#[test]
fn a_wholly_compacted_conversation_has_no_counter_left_to_read() {
    // The derivation's honest floor, stated as a test so nobody mistakes the
    // silence for a bug: an empty directory bounds no hole.
    let dir = tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join("agents").join(AGENT).join("messages")).unwrap();
    assert!(build(dir.path(), AGENT).entries.is_empty());
}

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
    let painted = super::render::painted_with(
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
