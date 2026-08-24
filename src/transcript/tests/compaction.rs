//! What the compactor deleted, and what it left in its place (bl-7bd2).
//!
//! Every test here drives [`build`] over a real directory, because the defect
//! was that `messages/` is a directory something else **writes to and deletes
//! from** — a hand-built `Vec<Entry>` cannot exhibit it. What the marker then
//! becomes on the glass is [`row`], split off at the cap on that same seam: a
//! hole is found by reading a directory, and said by projecting a row.

mod row;

use super::{AGENT, write_msg, write_summary};
use crate::transcript::{EntryKind, Transcript, build};
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
