//! The §6 decision queue through the transport: **read** the queue, **answer**
//! one row, and the answer is the queue that remains. Its own file at §12's
//! cap, on the seam that every other beat above is about one consumption pass's
//! bookkeeping and this one is about a rung the two frontends share a disk over.

use super::*;
use crate::boundary::deposit;
use crate::boundary::dispatch::Deps;
use crate::boundary::tests::snapshot;
use crate::git_tree::AgentState;
use serde_json::json;
use std::path::Path;
use std::sync::Arc;
use tempfile::tempdir;

/// The V5 rung, end to end through the transport (STORIES S14-T7): **read** the queue, **answer**
/// one row, and the answer is the queue that remains — one gesture per
/// decision. The watermark lands in the `ui.json` the window reads (I0), so the
/// two frontends converge over one disk rather than over a protocol.
#[test]
fn the_decision_queue_reads_and_answers_over_the_one_ui_json() {
    let root = tempdir().unwrap();
    let mut waiting = crate::boundary::tests::agent("c-1", AgentState::Stopped, 9);
    waiting.notify_oid = Some("n".repeat(40));
    let d = Deps {
        snapshot: Arc::new(snapshot(
            Path::new("/names/alba"),
            "alba",
            vec![waiting],
            vec![],
        )),
        ..deps(root.path())
    };
    let home = tempdir().unwrap();
    let ui_path = home.path().join("ui.json");
    let mut ui = UiState::open(ui_path.clone());

    deposit::deposit(root.path(), "q-att", &json!({"op": "attention"})).unwrap();
    assert_eq!(consume(&d, &mut ui, "T1", 100), 1);
    let asked = deposit::read_reply(root.path(), "q-att").unwrap();
    assert_eq!(asked["kind"], "attention");
    assert_eq!(asked["rows"][0]["agent"], "c-1");
    // The §3.1 name, which is the token the answering deposit below spells
    // verbatim (bl-22ab): the read's answer and the act's address are one
    // vocabulary, so a seat copies rather than translates.
    assert_eq!(asked["rows"][0]["workspace"], "alba");
    assert_eq!(asked["rows"][0]["signals"], json!(["notify", "stopped"]));

    deposit::deposit(
        root.path(),
        "a-seen",
        &json!({"op": "seen", "workspace": "alba", "agent": "c-1"}),
    )
    .unwrap();
    assert_eq!(consume(&d, &mut ui, "T2", 100), 1);
    let answered = deposit::read_reply(root.path(), "a-seen").unwrap();
    assert_eq!(answered["ok"], true);
    // The receipt names its item (bl-5cfe): a `seen` answered by the remainder
    // alone is byte-identical to the `attention` read above once the queue
    // empties, so the act said nothing about itself.
    assert_eq!(answered["kind"], "acknowledged");
    assert_eq!(answered["workspace"], "alba");
    assert_eq!(answered["agent"], "c-1");
    assert_eq!(answered["rows"], json!([]), "the queue that remains");
    assert!(
        std::fs::read_to_string(&ui_path)
            .unwrap()
            .contains(&"n".repeat(40)),
        "the watermark is on the disk the window reads (I0)"
    );
}
