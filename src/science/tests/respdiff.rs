//! S19-T3's pure half: the response diff (V3.3) is an honest line comparison —
//! equal inputs all-same, one-sided inputs one-sided, a replacement said
//! remove-then-add, and a response past the cap declared truncated rather than
//! silently clipped.

use crate::science::respdiff::{Diff, LINE_CAP, Row, lines};

#[test]
fn equal_responses_are_all_same_rows() {
    let diff = lines("a\nb", "a\nb");
    assert_eq!(
        diff,
        Diff {
            rows: vec![Row::Same("a".to_owned()), Row::Same("b".to_owned())],
            truncated: false,
        }
    );
}

#[test]
fn an_empty_side_answers_the_other_side_whole() {
    let left = lines("a\nb", "");
    assert_eq!(
        left.rows,
        vec![Row::Left("a".to_owned()), Row::Left("b".to_owned())]
    );
    let right = lines("", "a");
    assert_eq!(right.rows, vec![Row::Right("a".to_owned())]);
}

/// A replacement inside shared context reads remove-then-add, the order every
/// unified diff has taught.
#[test]
fn a_replacement_reads_remove_then_add() {
    let diff = lines("keep\nold\ntail", "keep\nnew\ntail");
    assert_eq!(
        diff.rows,
        vec![
            Row::Same("keep".to_owned()),
            Row::Left("old".to_owned()),
            Row::Right("new".to_owned()),
            Row::Same("tail".to_owned()),
        ]
    );
}

/// An insertion on the right and a removal on the left land on their own
/// sides — the table's tie-break exercised in both directions.
#[test]
fn one_sided_edits_land_on_their_own_sides() {
    let added = lines("a\nc", "a\nb\nc");
    assert_eq!(
        added.rows,
        vec![
            Row::Same("a".to_owned()),
            Row::Right("b".to_owned()),
            Row::Same("c".to_owned()),
        ]
    );
    let removed = lines("a\nb\nc", "a\nc");
    assert_eq!(
        removed.rows,
        vec![
            Row::Same("a".to_owned()),
            Row::Left("b".to_owned()),
            Row::Same("c".to_owned()),
        ]
    );
}

/// Past [`LINE_CAP`] lines the diff compares the head and says so — on either
/// side — instead of stalling a frame on an unbounded table.
#[test]
fn a_long_response_is_compared_on_its_head_and_says_so() {
    let long = "x\n".repeat(LINE_CAP + 5);
    let diff = lines(&long, "x");
    assert!(diff.truncated);
    assert_eq!(diff.rows.len(), LINE_CAP);
    let other = lines("x", &long);
    assert!(other.truncated);
    let short = lines("x", "x");
    assert!(!short.truncated);
}
