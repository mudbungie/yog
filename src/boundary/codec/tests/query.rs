//! The query roster's round trips — cut from the sibling table on the seam
//! production took (`codec/query.rs`): §4.8's taxonomy is a file boundary on
//! both sides.

use super::{p, rt};
use crate::boundary::{Gesture, Query, codec::decode};

#[test]
fn every_query_variant_round_trips() {
    rt(Gesture::Ask(Query::Workspaces));
    rt(Gesture::Ask(Query::Conversations {
        workspace: p("/ws"),
    }));
    rt(Gesture::Ask(Query::Balls));
    rt(Gesture::Ask(Query::Board));
    rt(Gesture::Ask(Query::Attention));
    rt(Gesture::Ask(Query::Ops { max: 32 }));
    rt(Gesture::Ask(Query::Search {
        text: "tekeli-li".into(),
    }));
    for file in [
        None,
        Some(crate::workdiff::WorkFile {
            ball: "bl-1".into(),
            path: "src/a.rs".into(),
        }),
    ] {
        rt(Gesture::Ask(Query::WorkDiff {
            workspace: p("/ws"),
            file,
        }));
    }
}

/// The work-diff's `file` is all-or-nothing: half of it is a patch read that
/// would open the wrong file, so the envelope refuses rather than guessing.
#[test]
fn a_half_named_work_file_is_refused() {
    let envelope = |file: serde_json::Value| serde_json::json!({ "op": "work-diff", "workspace": "/ws", "file": file });
    assert!(decode(&envelope(serde_json::json!({ "ball": "bl-1" }))).is_err());
    assert!(decode(&envelope(serde_json::json!({ "path": "a.rs" }))).is_err());
    assert_eq!(
        decode(&envelope(serde_json::json!("src/a.rs"))),
        Err("file: not a JSON object".to_owned())
    );
}
