//! The consumer thread's tables (§8.5): the pass over the latest published
//! snapshot, and the one thing only a real thread can prove — spawn, consume,
//! stop. The fixtures the REMOTE §4 half builds its worlds from live here too.

/// The scoped intake (REMOTE §4, bl-8bbc): what a connection enumerates, what
/// an unregistered name earns, and the create that seats its own client. Its
/// own file at §12's cap — a real seam, because everything above is the
/// in-world intake and everything there is the wire's.
mod scope;
/// The REMOTE §5 half (bl-4e08): the intake threading its identity to the ACT
/// side, and the roster read that joins the three facts back.
mod tools;

use super::*;
use crate::boundary::deposit;
use crate::cli_outbound::Cli;
use crate::ui_state::SystemClock;
use serde_json::json;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tempfile::tempdir;

fn ctx(state_root: &std::path::Path) -> ConsumerCtx {
    over(
        state_root,
        crate::app::Snapshot::empty(Instant::now()),
        PathBuf::from("/data"),
        Cli::new("/no/such/lernie"),
    )
}

/// A context over a stated snapshot, world root and `lernie` — everything the
/// REMOTE §4 tests below vary.
fn over(
    state_root: &std::path::Path,
    snap: crate::app::Snapshot,
    yog_data_root: PathBuf,
    lernie: Cli,
) -> ConsumerCtx {
    ConsumerCtx {
        yog_binary: PathBuf::from("/no/such/yog"),
        world: crate::test_support::no_world(),
        lernie,
        bl: Cli::new("/no/such/bl"),
        state_root: state_root.to_path_buf(),
        home: PathBuf::from("/home/x"),
        yog_data_root,
        balls_state_root: PathBuf::from("/balls"),
        ui_path: state_root.join("ui.json"),
        cell: crate::state::new_snapshot_cell(std::sync::Arc::new(snap)),
        presence: crate::registry::presence::Presence::default(),
        mailbox: crate::registry::mailbox::Mailbox::default(),
        clock: Arc::new(SystemClock),
    }
}

/// A snapshot holding one named workspace per element of `names`, under `root`.
fn world_of(root: &std::path::Path, names: &[&str]) -> crate::app::Snapshot {
    let mut snap = crate::app::Snapshot::empty(Instant::now());
    snap.workspaces = names
        .iter()
        .map(|name| crate::binding::Workspace {
            path: crate::binding::workspace_path(root, name),
            kind: crate::binding::WorkspaceKind::Named {
                name: (*name).to_owned(),
            },
        })
        .collect();
    snap
}

/// The workspace names a `workspaces` reply lists.
fn listed(reply: &serde_json::Value) -> Vec<String> {
    reply["rows"]
        .as_array()
        .map(|rows| {
            rows.iter()
                .filter_map(|r| r["workspace"].as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

fn client(name: &str) -> crate::registry::Client {
    crate::registry::Client::parse(name).expect("a usable identity")
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
