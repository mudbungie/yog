//! The consumer thread's tables (§8.5): the pass over the latest published
//! snapshot, and the one thing only a real thread can prove — spawn, consume,
//! stop.

use super::*;
use crate::boundary::deposit;
use crate::cli_outbound::Cli;
use crate::ui_state::SystemClock;
use serde_json::json;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tempfile::tempdir;

fn ctx(state_root: &std::path::Path) -> ConsumerCtx {
    let snap = std::sync::Arc::new(crate::app::Snapshot::empty(Instant::now()));
    ConsumerCtx {
        yog_binary: PathBuf::from("/no/such/yog"),
        world: crate::test_support::no_world(),
        lernie: Cli::new("/no/such/lernie"),
        bl: Cli::new("/no/such/bl"),
        state_root: state_root.to_path_buf(),
        home: PathBuf::from("/home/x"),
        yog_data_root: PathBuf::from("/data"),
        balls_state_root: PathBuf::from("/balls"),
        ui_path: state_root.join("ui.json"),
        cell: crate::state::new_snapshot_cell(snap),
        clock: Arc::new(SystemClock),
    }
}

#[test]
fn an_empty_inbox_costs_one_listing_and_nothing_else() {
    let root = tempdir().unwrap();
    assert_eq!(ctx(root.path()).pass(), 0);
}

#[test]
fn a_pass_answers_from_the_latest_published_snapshot() {
    let root = tempdir().unwrap();
    deposit::deposit(root.path(), "q-1", &json!({"op": "workspaces"})).unwrap();
    assert_eq!(ctx(root.path()).pass(), 1);
    let reply = deposit::read_reply(root.path(), "q-1").unwrap();
    assert_eq!(reply["ok"], true);
    assert_eq!(reply["kind"], "workspaces");
    assert_eq!(
        reply["rows"].as_array().unwrap().len(),
        0,
        "the empty snapshot"
    );
}

/// The wire's intake is this same context (REMOTE §3, bl-b6fa): an envelope
/// handed straight in is answered exactly as a deposited one is, which is what
/// makes the listener a second intake rather than a second implementation.
#[test]
fn one_envelope_is_answered_where_a_deposit_is() {
    let root = tempdir().unwrap();
    let reply = ctx(root.path()).answer(&json!({"op": "workspaces"}));
    assert_eq!(reply["ok"], true);
    assert_eq!(reply["kind"], "workspaces");
    // And a torn envelope refuses in-band rather than wedging the caller.
    let refusal = ctx(root.path()).answer(&json!({"op": "enhance"}));
    assert_eq!(refusal["ok"], false);
}

#[test]
fn the_thread_consumes_a_deposit_and_stops_on_drop() {
    let root = tempdir().unwrap();
    deposit::deposit(root.path(), "q-t", &json!({"op": "balls"})).unwrap();
    let consumer = Consumer::spawn(Arc::new(ctx(root.path())));
    let deadline = Instant::now() + Duration::from_secs(10);
    while deposit::read_reply(root.path(), "q-t").is_none() {
        assert!(Instant::now() < deadline, "the consumer never answered");
        std::thread::sleep(Duration::from_millis(10));
    }
    drop(consumer); // joins cleanly — the Drop is the shutdown
}
