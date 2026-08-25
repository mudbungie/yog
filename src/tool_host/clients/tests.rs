//! The `clients` tool: what it accepts, what it observes, and what it makes
//! callable (REMOTE §5, bl-c907).

/// **What it makes callable** — the third of the three subjects this file's own
/// doc names, in its own file at §12's per-file budget. `load` is the only op
/// that writes: it freezes definitions into the agent's durable set, so what it
/// refuses and what it leaves behind on a refusal are one subject, and the two
/// read-only ops share none of it.
mod load;

use super::*;
use crate::boundary::reply::{self, Reply};
use crate::registry::tools::Tool;
use crate::tool_host::tests::{budget, engine, impatient, site, tool};
use serde_json::json;
use std::path::Path;
use tempfile::TempDir;

fn quiet() -> AtomicBool {
    AtomicBool::new(false)
}

fn row(client: &str, present: bool, tools: Vec<Tool>) -> ClientRow {
    ClientRow {
        client: client.to_owned(),
        present,
        tools,
    }
}

/// Answer one op against a stand-in engine holding `rows`.
fn against(root: &Path, rows: Vec<ClientRow>, input: &Value) -> Result<String, String> {
    let (handle, _seen) = engine(root, &reply::encode(&Reply::Clients(rows)));
    let answered = answer(&site(root, budget()), input, &quiet());
    handle.join().expect("engine");
    answered
}

/// The schema declares the three ops and requires the one field that selects
/// among them.
#[test]
fn the_declared_schema_names_the_three_ops() {
    let s = schema();
    assert_eq!(
        s["properties"]["op"]["enum"],
        json!(["list", "get", "load"])
    );
    assert_eq!(s["required"], json!(["op"]));
    assert!(DESCRIPTION.contains("op=load"));
}

/// Every op the model can spell, read back.
#[test]
fn a_well_formed_invocation_reads_as_its_op() {
    assert_eq!(parse(&json!({"op": "list"})).expect("list"), Op::List);
    assert_eq!(
        parse(&json!({"op": "get", "client": "laptop"})).expect("get"),
        Op::Get("laptop".to_owned())
    );
    assert_eq!(
        parse(&json!({"op": "load", "client": "laptop", "tools": ["Bash"]})).expect("load"),
        Op::Load("laptop".to_owned(), vec!["Bash".to_owned()])
    );
}

/// Every malformed one declines naming the field, because the caller is a
/// model and a decline it can read is one it can correct next turn.
#[test]
fn a_malformed_invocation_names_the_field() {
    let cases = [
        (json!(7), "not a JSON object"),
        (json!({}), "\"op\""),
        (json!({"op": "wat"}), "unknown op"),
        (json!({"op": "get"}), "\"client\""),
        (json!({"op": "get", "client": ""}), "\"client\""),
        (json!({"op": "load", "client": "a"}), "\"tools\""),
        (
            json!({"op": "load", "client": "a", "tools": []}),
            "\"tools\"",
        ),
        (
            json!({"op": "load", "client": "a", "tools": ["Bash", 7]}),
            "\"tools\"",
        ),
    ];
    for (input, said) in cases {
        let e = parse(&input).expect_err("declined");
        assert!(e.contains(said), "{input} -> {e}");
    }
}

/// `list` is a dated observation of the roster: who is registered, who is
/// connected right now, and how much each offers.
#[test]
fn list_observes_the_roster_at_an_instant() {
    let root = TempDir::new().expect("tmp");
    let said = against(
        root.path(),
        vec![
            row("laptop", true, vec![tool("Bash")]),
            row("phone", false, vec![]),
        ],
        &json!({"op": "list"}),
    )
    .expect("listed");
    assert!(said.contains("observed 1970-01-01 00:00:00Z"), "{said}");
    assert!(said.contains("laptop — connected right now, advertising 1 tool\n"));
    assert!(said.contains("phone — not connected right now, advertising 0 tools\n"));
    assert!(said.contains("op=get with client=<name>"), "{said}");
}

/// A workspace with no clients says so, rather than rendering an empty list.
#[test]
fn list_says_so_when_nothing_is_registered() {
    let root = TempDir::new().expect("tmp");
    let said = against(root.path(), vec![], &json!({"op": "list"})).expect("listed");
    assert!(said.contains("(none is registered here)"), "{said}");
}

/// `get` is one client's detail with each advertised tool and the name it
/// would load as — the disambiguation §5.1 leaves to the load act, shown
/// before it is taken.
#[test]
fn get_shows_each_tool_and_the_name_it_loads_as() {
    let root = TempDir::new().expect("tmp");
    let said = against(
        root.path(),
        vec![row("laptop", true, vec![tool("Bash"), tool("Read")])],
        &json!({"op": "get", "client": "laptop"}),
    )
    .expect("got");
    assert!(
        said.contains("client \"laptop\" of workspace \"home\""),
        "{said}"
    );
    assert!(said.contains("It advertises 2 tools:"), "{said}");
    assert!(said.contains("Bash — what Bash does"), "{said}");
    assert!(said.contains("loads as: laptop_Bash"), "{said}");
    assert!(said.contains("op=load with client=\"laptop\""), "{said}");
}

/// A client that has advertised nothing reads as advertising nothing — the
/// posture of every client before it first connects as a tool host.
#[test]
fn get_on_a_client_that_advertises_nothing_says_so() {
    let root = TempDir::new().expect("tmp");
    let said = against(
        root.path(),
        vec![row("phone", false, vec![])],
        &json!({"op": "get", "client": "phone"}),
    )
    .expect("got");
    assert!(said.contains("not connected right now"), "{said}");
    assert!(said.contains("It advertises no tools."), "{said}");
}

/// An identity this workspace has not registered is **absent**, not forbidden
/// (REMOTE §4): the sentence says there is no such client here.
#[test]
fn an_unregistered_client_is_absent() {
    let root = TempDir::new().expect("tmp");
    let e = against(
        root.path(),
        vec![row("laptop", true, vec![])],
        &json!({"op": "get", "client": "desk"}),
    )
    .expect_err("absent");
    assert!(e.contains("no client \"desk\" is registered"), "{e}");
    assert!(e.contains("the 1 there are"), "{e}");
}

/// A malformed invocation is declined **before** the engine is asked: the
/// refusal belongs to the caller, not to a round trip.
#[test]
fn a_declined_invocation_never_reaches_the_engine() {
    let root = TempDir::new().expect("tmp");
    let e = answer(
        &site(root.path(), impatient()),
        &json!({"op": "wat"}),
        &quiet(),
    )
    .expect_err("declined");
    assert!(e.contains("unknown op"), "{e}");
    assert!(
        crate::boundary::deposit::pending(root.path()).is_empty(),
        "nothing was deposited"
    );
}

/// An engine that does not answer is the ask's own sentence, in band.
#[test]
fn an_unreachable_engine_is_said_in_band() {
    let root = TempDir::new().expect("tmp");
    let e = answer(
        &site(root.path(), impatient()),
        &json!({"op": "list"}),
        &quiet(),
    )
    .expect_err("no engine");
    assert!(e.contains("no engine answered"), "{e}");
}
