//! The driver's ask: the round trip through the inbox, and the four ways it
//! ends in a sentence rather than a wait (REMOTE §3, §5).

use super::*;
use crate::registry::tools::Tool;
use crate::tool_host::tests::{budget, engine, impatient};
use serde_json::json;
use tempfile::TempDir;

fn quiet() -> AtomicBool {
    AtomicBool::new(false)
}

/// The production budget is a real bound, not an absence of one.
#[test]
fn the_default_budget_is_a_bounded_wait() {
    let b = Budget::default();
    assert!(b.waits > 0 && b.tick > Duration::ZERO);
    assert_eq!(b.tick * b.waits, Duration::from_secs(10));
}

/// The happy path: deposit, the engine answers, the envelope comes back whole.
#[test]
fn a_deposit_earns_the_engine_s_own_envelope() {
    let root = TempDir::new().expect("tmp");
    let answer = json!({"ok": true, "kind": "clients", "rows": []});
    let (handle, seen) = engine(root.path(), &answer);
    let got = ask(
        root.path(),
        &json!({"op": "clients", "workspace": "home"}),
        budget(),
        &quiet(),
    )
    .expect("answered");
    handle.join().expect("engine");
    assert_eq!(got, answer);
    assert_eq!(seen.recv().expect("request")["op"], "clients");
}

/// No engine is a sentence, not a hang — the router's own deadline
/// (`docs/DESIGN_TOOL_INJECTION.md` §3.3).
#[test]
fn no_engine_gives_up_and_says_so() {
    let root = TempDir::new().expect("tmp");
    let e = ask(
        root.path(),
        &json!({"op": "clients"}),
        impatient(),
        &quiet(),
    )
    .expect_err("no consumer");
    assert!(e.contains("no engine answered"), "{e}");
}

/// A stop landing mid-wait ends the wait, so a torn-down drive is not held up
/// by an ask nobody will answer.
#[test]
fn a_stop_ends_the_wait() {
    let root = TempDir::new().expect("tmp");
    let stopped = AtomicBool::new(true);
    let e = ask(
        root.path(),
        &json!({"op": "clients"}),
        impatient(),
        &stopped,
    )
    .expect_err("stopped");
    assert!(e.contains("stopped while waiting"), "{e}");
}

/// A state root that cannot hold an inbox refuses at the mint — before any
/// deposit exists, so nothing is left behind.
#[test]
fn an_unusable_state_root_refuses_at_the_mint() {
    let root = TempDir::new().expect("tmp");
    let file = root.path().join("not-a-dir");
    std::fs::write(&file, b"x").expect("write");
    let e = ask(&file, &json!({"op": "clients"}), impatient(), &quiet()).expect_err("no inbox");
    assert!(e.starts_with("gesture id: "), "{e}");
}

/// An id whose inbox slot is already taken refuses at the deposit — the
/// create-only discipline, surfaced as this ask's own sentence.
#[test]
fn an_occupied_inbox_slot_refuses_at_the_deposit() {
    let root = TempDir::new().expect("tmp");
    let dir = crate::boundary::deposit::gestures_dir(root.path());
    std::fs::create_dir_all(dir.join("toolhost-0.json")).expect("squat the slot");
    let e = ask(
        root.path(),
        &json!({"op": "clients"}),
        impatient(),
        &quiet(),
    )
    .expect_err("occupied");
    assert!(e.starts_with("deposit: "), "{e}");
}

/// The roster read is `Query::Clients` and its answer decodes to rows — the
/// bl-4e08 surface, reached with no new verb.
#[test]
fn the_roster_is_the_landed_clients_query() {
    let root = TempDir::new().expect("tmp");
    let rows = vec![crate::registry::roster::ClientRow {
        client: "laptop".to_owned(),
        present: true,
        tools: vec![Tool {
            name: "Bash".to_owned(),
            description: "run a command".to_owned(),
            input_schema: json!({"type": "object"}),
        }],
    }];
    let answer = crate::boundary::reply::encode(&Reply::Clients(rows.clone()));
    let (handle, seen) = engine(root.path(), &answer);
    let got = roster(root.path(), "home", budget(), &quiet()).expect("rows");
    handle.join().expect("engine");
    assert_eq!(got, rows);
    assert_eq!(
        seen.recv().expect("request"),
        json!({"op": "clients", "workspace": "home"})
    );
}

/// Three ways the engine can fail to give a roster, each named rather than
/// swallowed: a refusal, an answer of another kind, and bytes no decoder reads.
#[test]
fn an_answer_that_is_not_a_roster_is_named() {
    let root = TempDir::new().expect("tmp");
    let cases = [
        (
            json!({"ok": false, "error": "unknown workspace \"x\""}),
            "unknown workspace",
        ),
        (
            crate::boundary::reply::encode(&Reply::Acked),
            "not a client roster",
        ),
        (json!({"ok": true, "kind": "no-such-kind"}), "undecodable"),
    ];
    for (answer, said) in cases {
        let (handle, _seen) = engine(root.path(), &answer);
        let e = roster(root.path(), "home", budget(), &quiet()).expect_err("not a roster");
        handle.join().expect("engine");
        assert!(e.contains(said), "{e}");
    }
}
