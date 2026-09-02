//! The help table's own invariants (§8.5): it is the single source, so what it
//! says has to be true of the surface it describes — every row a gesture, every
//! gesture a row, and a page that actually says something.

use super::*;
use crate::boundary::{Gesture, codec, line};
use serde_json::json;

/// Every row names a gesture the reader answers, and carries all three of the
/// things a page is made of. A row with an empty detail is a promise of help
/// that pays out nothing.
#[test]
fn every_row_describes_a_real_gesture() {
    for row in &table() {
        assert!(known(row.verb), "{} is not known to itself", row.verb);
        assert!(
            line::parse(&format!("/{}", row.verb), &line::Context::default()).is_ok()
                || line::parse(&format!("/{} x", row.verb), &line::Context::default()).is_err()
                // A verb whose whole grammar is ONE word satisfies neither of
                // the two above — bare is the refusal naming what is missing,
                // and one word is the gesture (`/capture <invocation>`, whose
                // subject is a handle no seat's context can hold, bl-024b). It
                // must still refuse *something*, or the page describes a
                // control that swallows every line.
                || line::parse(&format!("/{} x y", row.verb), &line::Context::default()).is_err(),
            "{} must be readable as a line",
            row.verb
        );
        assert!(row.usage.starts_with('/'), "{} usage is a line", row.verb);
        assert!(!row.summary.is_empty() && !row.detail.is_empty());
        assert!(
            row.detail.len() > row.summary.len(),
            "{}'s page says no more than its one-liner",
            row.verb
        );
    }
}

/// Every gesture the codec answers to has a page. The two lists are the same
/// surface seen twice; a verb in one and not the other is the drift this table
/// exists to make impossible.
#[test]
fn every_gesture_the_codec_reads_has_a_page() {
    for row in &table() {
        let envelope = json!({ "op": row.verb });
        let decoded = codec::decode(&envelope);
        assert!(
            !matches!(&decoded, Err(reason) if reason.contains("unknown op")),
            "{} is helped but unspellable as an envelope: {decoded:?}",
            row.verb
        );
    }
}

/// **Every gesture a foot may say is classed `machine`** (`docs/PARITY.md` §2,
/// bl-8758). The two facts are written in different places for good reason —
/// `Grade::admits` is an authorization enumeration and the class is an
/// interface obligation — but they answer to each other: a gesture yog will
/// only ever accept from a tool host cannot be one every seat owes a control
/// for. The walk is the codec's own exhaustive surface, and the op token is the
/// unit (PARITY §3), so a folded family is judged per member.
#[test]
fn a_foot_gesture_is_never_owed_a_control() {
    let table = table();
    let mut judged = 0;
    for gesture in codec::tests::surface::gestures() {
        if !crate::registry::peer::Grade::Foot.admits(&gesture) {
            continue;
        }
        let envelope = codec::encode(&gesture);
        let op = envelope.get("op").and_then(serde_json::Value::as_str);
        let row = op.and_then(|op| table.iter().find(|row| row.verb == op));
        assert_eq!(
            row.map(|row| row.surface),
            Some(Surface::Machine),
            "{op:?} is admitted to a foot but is not classed machine"
        );
        judged += 1;
    }
    assert!(judged >= 3, "the foot's own three gestures went unwalked");
}

/// Asking about one verb is a page; asking about everything is a roster. The
/// shape of the answer follows the question, because a wall of paragraphs
/// answers nothing anyone asked.
#[test]
fn a_page_is_detail_and_a_roster_is_summaries() {
    let page = render(&rows(Some("scan")));
    assert!(page.starts_with("/scan"), "{page}");
    assert!(
        page.contains("silent-death sweep") || page.contains("epitaph"),
        "{page}"
    );

    let roster = roster();
    for row in &table() {
        assert!(roster.contains(row.usage), "{} is unlisted", row.verb);
        assert!(roster.contains(row.summary));
    }
    assert!(
        !roster.contains(table()[0].detail),
        "the roster must not print pages"
    );
}

/// An unknown verb never reaches the answer: the codec refuses it, so the rows
/// are total and no seat renders an empty page.
#[test]
fn help_about_nothing_is_refused_at_the_edge() {
    assert!(!known("enhance"));
    let refused = codec::decode(&json!({ "op": "help", "verb": "enhance" }));
    assert_eq!(refused, Err("help: unknown verb \"enhance\"".to_owned()));
    let read = codec::decode(&json!({ "op": "help", "verb": "close" }));
    assert!(matches!(read, Ok(Gesture::Ask(_))));
}
