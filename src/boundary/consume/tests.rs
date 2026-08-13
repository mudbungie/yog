//! One consumption pass, tabled (§8.5): queries answered, refusals written,
//! a spawn-failing action logged and refused, and the reply-write failure's
//! own ops row — no error class dropped (INV-2).

use super::*;
use crate::boundary::deposit;
use crate::boundary::dispatch::Deps;
use crate::boundary::tests::snapshot;
use crate::cli_outbound::Cli;
use crate::git_tree::AgentState;
use serde_json::json;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tempfile::tempdir;

fn deps(state_root: &Path) -> Deps {
    Deps {
        lernie: Cli::new("/no/such/lernie"),
        bl: Cli::new("/no/such/bl"),
        state_root: state_root.to_path_buf(),
        yog_binary: PathBuf::from("/no/such/yog"),
        world: crate::test_support::no_world(),
        home: PathBuf::from("/home/x"),
        yog_data_root: PathBuf::from("/data"),
        balls_state_root: PathBuf::from("/balls"),
        snapshot: Arc::new(snapshot(
            Path::new("/names/alba"),
            "alba",
            vec![crate::boundary::tests::agent("c-1", AgentState::Live, 9)],
            vec![],
        )),
        mint_seed: 7,
    }
}

fn ui() -> UiState {
    UiState::open(PathBuf::from("/nonexistent/ui.json"))
}

#[test]
fn a_query_deposit_earns_its_reply_and_leaves_the_claimed_audit() {
    let root = tempdir().unwrap();
    let d = deps(root.path());
    deposit::deposit(root.path(), "q-1", &json!({"op": "balls"})).unwrap();
    assert_eq!(consume(&d, &mut ui(), "T1", 100), 1);
    let reply = deposit::read_reply(root.path(), "q-1").unwrap();
    assert_eq!(reply["ok"], true);
    assert_eq!(reply["kind"], "balls");
    assert!(
        deposit::gestures_dir(root.path())
            .join("claimed")
            .join("q-1.json")
            .is_file(),
        "the deposit stays, claimed — the audit's other half"
    );
    assert_eq!(consume(&d, &mut ui(), "T2", 100), 0, "nothing left");
}

/// A query can refuse too (§8.5, bl-0164 — the §9 config family's reads ask
/// the world, so they can fail exactly as their writes can): the deposit
/// still answers, naming why, rather than wedging the inbox.
///
/// The refusing read here is a **lineage** destination, whose browse stays the
/// §9.3 pane's own gesture (bl-ee0a) and which this single-destination query
/// therefore refuses outright. (It used to be the marks read on an unprimed
/// project; since bl-e47b that read is infallible, because an agent's branch
/// lives in its own space and needs no primed checkout to answer.)
#[test]
fn a_refusing_query_deposit_answers_with_a_refusal_not_a_wedge() {
    let root = tempdir().unwrap();
    let d = deps(root.path());
    deposit::deposit(
        root.path(),
        "q-refuse",
        &json!({"op": "config", "target": {"file": "branch", "workspace": "/ws",
                "lineage": "default", "path": "providers.yaml", "origin": "advance"}}),
    )
    .unwrap();
    assert_eq!(consume(&d, &mut ui(), "T1", 100), 1);
    let reply = deposit::read_reply(root.path(), "q-refuse").unwrap();
    assert_eq!(reply["ok"], false, "{reply}");
}

#[test]
fn a_conversations_query_renders_the_same_rows_the_frame_would() {
    let root = tempdir().unwrap();
    let d = deps(root.path());
    deposit::deposit(
        root.path(),
        "q-2",
        &json!({"op": "conversations", "workspace": "/names/alba"}),
    )
    .unwrap();
    consume(&d, &mut ui(), "T1", 100);
    let reply = deposit::read_reply(root.path(), "q-2").unwrap();
    assert_eq!(reply["rows"][0]["root_id"], "c-1");
    assert_eq!(reply["rows"][0]["state"], "live");
}

#[test]
fn a_mangled_deposit_answers_with_a_refusal_not_a_wedge() {
    let root = tempdir().unwrap();
    let d = deps(root.path());
    deposit::deposit(root.path(), "g-raw", &json!({"op": "warp"})).unwrap();
    std::fs::write(
        deposit::gestures_dir(root.path()).join("g-torn.json"),
        b"not json",
    )
    .unwrap();
    assert_eq!(consume(&d, &mut ui(), "T1", 100), 2);
    let unknown = deposit::read_reply(root.path(), "g-raw").unwrap();
    assert_eq!(unknown["ok"], false);
    assert!(unknown["error"].as_str().unwrap().contains("unknown op"));
    let torn = deposit::read_reply(root.path(), "g-torn").unwrap();
    assert_eq!(torn["ok"], false);
    assert!(torn["error"].as_str().unwrap().contains("not JSON"));
}

#[test]
fn a_spawn_failing_action_is_refused_and_its_ops_row_stands() {
    let root = tempdir().unwrap();
    let d = deps(root.path());
    deposit::deposit(
        root.path(),
        "a-1",
        &json!({"op": "scan", "workspace": "/names/alba"}),
    )
    .unwrap();
    assert_eq!(consume(&d, &mut ui(), "T1", 100), 1);
    let reply = deposit::read_reply(root.path(), "a-1").unwrap();
    assert_eq!(reply["ok"], false, "the spawn never launched");
    let ops = crate::opslog::tail(root.path(), 8);
    assert_eq!(ops.len(), 1, "the synthetic failure line is durable (§4.2)");
    assert_eq!(ops[0].ts, "T1");
}

#[test]
fn the_trail_verbs_land_through_the_boundary() {
    let root = tempdir().unwrap();
    let d = deps(root.path());
    deposit::deposit(root.path(), "a-ack", &json!({"op": "ack"})).unwrap();
    assert_eq!(consume(&d, &mut ui(), "T1", 100), 1);
    assert_eq!(
        deposit::read_reply(root.path(), "a-ack").unwrap()["kind"],
        "acked"
    );
    deposit::deposit(root.path(), "a-clear", &json!({"op": "clear-trail"})).unwrap();
    assert_eq!(consume(&d, &mut ui(), "T2", 100), 1);
    assert_eq!(
        deposit::read_reply(root.path(), "a-clear").unwrap()["kind"],
        "trail-cleared"
    );
    let ops = crate::opslog::tail(root.path(), 8);
    assert_eq!(
        ops.len(),
        1,
        "the clear truncated; its own row opens the trail"
    );
    assert_eq!(ops[0].ts, "T2");
}

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
    assert_eq!(asked["rows"][0]["workspace"], "/names/alba");
    assert_eq!(asked["rows"][0]["signals"], json!(["notify", "stopped"]));

    deposit::deposit(
        root.path(),
        "a-seen",
        &json!({"op": "seen", "workspace": "/names/alba", "agent": "c-1"}),
    )
    .unwrap();
    assert_eq!(consume(&d, &mut ui, "T2", 100), 1);
    let answered = deposit::read_reply(root.path(), "a-seen").unwrap();
    assert_eq!(answered["ok"], true);
    assert_eq!(answered["kind"], "attention");
    assert_eq!(answered["rows"], json!([]), "the queue that remains");
    assert!(
        std::fs::read_to_string(&ui_path)
            .unwrap()
            .contains(&"n".repeat(40)),
        "the watermark is on the disk the window reads (I0)"
    );
}

#[test]
fn an_unwritable_reply_leaves_its_own_step_failure_row() {
    let root = tempdir().unwrap();
    let d = deps(root.path());
    deposit::deposit(root.path(), "q-9", &json!({"op": "balls"})).unwrap();
    // A regular file where `replies/` must be a directory: the write fails.
    std::fs::write(deposit::gestures_dir(root.path()).join("replies"), b"x").unwrap();
    assert_eq!(consume(&d, &mut ui(), "T1", 100), 1);
    let ops = crate::opslog::tail(root.path(), 8);
    assert!(
        ops.iter()
            .any(|e| e.argv.get(1).is_some_and(|s| s == "gesture-reply")),
        "{ops:?}"
    );
}
