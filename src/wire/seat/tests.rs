//! The terminal seat: what it sends, what it prints, and what it refuses.

use super::*;
use crate::test_support::wire::{material as fixture, mint};
use crate::wire::server::{Answerer, Listener};
use serde_json::json;
use std::sync::Arc;
use tempfile::TempDir;

/// An engine stand-in answering a fixed verdict — the seat's subject is the
/// transport and the exit, not what the boundary decided.
struct Verdict(bool);

impl Answerer for Verdict {
    fn answer(&self, request: Value) -> Vec<Value> {
        vec![json!({"ok": self.0, "kind": "echo", "asked": request})]
    }
}

/// A world whose material is minted and whose address is a live listener's.
fn engine(ok: bool) -> (TempDir, crate::xdg::Env, Listener) {
    let tmp = TempDir::new().expect("tmp");
    let world = crate::test_support::world_under(tmp.path());
    let dir = material::dir(&world);
    mint(&dir);
    let listener = Listener::bind(
        &fixture(&dir, Role::Server, crate::test_support::wire::EPHEMERAL),
        Arc::new(Verdict(ok)),
    )
    .expect("bind");
    std::fs::write(dir.join(material::ADDRESS), listener.address()).expect("address");
    (tmp, world, listener)
}

/// The whole seat: a gesture typed at a terminal reaches an engine over mTLS
/// and its verdict is this process's exit.
#[test]
fn a_gesture_reaches_the_engine_and_its_verdict_is_the_exit() {
    let (_tmp, world, _listener) = engine(true);
    assert_eq!(run(&world, &[r#"{"op":"workspaces"}"#.to_owned()]), 0);
}

/// A `/slash` line is the same envelope by another spelling — the reader is
/// literally `yog gesture`'s, so the two seats cannot drift.
#[test]
fn a_line_is_the_same_envelope() {
    let (_tmp, world, _listener) = engine(true);
    assert_eq!(
        run(&world, &["/workspaces".to_owned()]),
        0,
        "the line spelling reaches the same engine"
    );
}

/// A reply that is not ok exits 1 — the seat reports the boundary's verdict
/// and never substitutes its own.
#[test]
fn a_refusal_from_the_engine_exits_one() {
    let (_tmp, world, _listener) = engine(false);
    assert_eq!(run(&world, &[r#"{"op":"workspaces"}"#.to_owned()]), 1);
}

/// An engine that is not up is a transport failure, not a usage error.
#[test]
fn a_dead_engine_exits_one() {
    let (_tmp, world, listener) = engine(true);
    drop(listener);
    assert_eq!(run(&world, &[r#"{"op":"workspaces"}"#.to_owned()]), 1);
}

/// A machine with no wire refuses at the seat, and says how to be right —
/// bootstrap is out-of-channel, so the seat can only name the remedy.
#[test]
fn an_unprovisioned_machine_refuses_with_the_remedy() {
    let tmp = TempDir::new().expect("tmp");
    let world = crate::test_support::world_under(tmp.path());
    assert_eq!(
        run(&world, &[r#"{"op":"workspaces"}"#.to_owned()]),
        USAGE_EXIT
    );
}

/// Half a provisioning refuses the same way rather than degrading.
#[test]
fn a_half_provisioned_machine_refuses() {
    let (_tmp, world, _listener) = engine(true);
    std::fs::remove_file(material::dir(&world).join("client.key")).expect("rm");
    assert_eq!(
        run(&world, &[r#"{"op":"workspaces"}"#.to_owned()]),
        USAGE_EXIT
    );
}

/// Material that will not build a seat refuses before anything is dialled.
#[test]
fn unusable_material_refuses_before_dialling() {
    let (_tmp, world, _listener) = engine(true);
    std::fs::write(material::dir(&world).join("client.pem"), "").expect("write");
    assert_eq!(
        run(&world, &[r#"{"op":"workspaces"}"#.to_owned()]),
        USAGE_EXIT
    );
}

/// Bad argv never reaches the wire, and the usage line names *this* seat.
#[test]
fn bad_argv_never_reaches_the_wire() {
    let tmp = TempDir::new().expect("tmp");
    let world = crate::test_support::world_under(tmp.path());
    assert_eq!(run(&world, &[]), USAGE_EXIT);
    assert_eq!(
        run(&world, &["--nonesuch".to_owned(), "x".to_owned()]),
        USAGE_EXIT
    );
}

/// **Help is answered in place** (§8.5): asking what a verb does must not
/// depend on an engine being up — or, here, on this machine being provisioned
/// at all.
#[test]
fn help_is_answered_without_a_wire() {
    let tmp = TempDir::new().expect("tmp");
    let world = crate::test_support::world_under(tmp.path());
    assert_eq!(run(&world, &["--help".to_owned()]), 0);
    assert_eq!(run(&world, &["--help".to_owned(), "close".to_owned()]), 0);
}

/// An engine that terminated its stream without saying anything is not ok:
/// silence is never an answer.
#[test]
fn an_empty_stream_is_not_an_answer() {
    assert_eq!(report(&[]), 1);
    assert_eq!(report(&[json!({"ok": true})]), 0);
}
