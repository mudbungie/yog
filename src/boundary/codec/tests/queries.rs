//! The **query** half of the round-trip table (§8.5): every populating read
//! re-enters as itself, and the two envelopes that must refuse rather than
//! guess a target. Split from the action half at §12's per-file budget, on the
//! same seam the codec itself took ([`query`](super::super::query)).

use super::super::{decode, encode};
use crate::boundary::{Gesture, Query};
use std::path::PathBuf;

fn rt(gesture: Gesture) {
    let encoded = encode(&gesture);
    assert_eq!(decode(&encoded), Ok(gesture.clone()), "via {encoded}");
}

fn p(s: &str) -> PathBuf {
    PathBuf::from(s)
}

#[test]
fn every_query_variant_round_trips() {
    rt(Gesture::Ask(Query::Workspaces));
    rt(Gesture::Ask(Query::Conversations {
        workspace: p("/ws"),
    }));
    rt(Gesture::Ask(Query::Balls));
    rt(Gesture::Ask(Query::Board));
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
