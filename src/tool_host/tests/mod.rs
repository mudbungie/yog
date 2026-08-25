//! The injection itself: what it declares, what it answers, and what it
//! declines (REMOTE §5, bl-c907).

/// **The substrate every beat in this family drives against** (REMOTE §5) — the
/// stand-in engines that hold up the deposit consumer's contract and nothing
/// else, the two budgets, and the `Site` an injection answers at. Its own file
/// at §12's per-file budget: four sibling test modules import it, so it is a
/// shared subject rather than this file's own scaffolding.
mod stand_in;

pub(super) use stand_in::{budget, engine, impatient, scripted, site, tool};

use super::*;
use crate::boundary::deposit;
use crate::registry::tools::Tool;
use crate::test_support::FakeClock;
use serde_json::{Value, json};
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::Receiver;
use std::thread::JoinHandle;
use std::time::Duration;
use tempfile::TempDir;

fn injection(root: &Path, driving: Option<(String, String)>) -> Injection {
    Injection::new(
        root.to_path_buf(),
        impatient(),
        impatient(),
        FakeClock::new().arc(),
        driving,
    )
}

/// One invocation, as lernie's executor hands it over. A macro rather than a
/// function because three independent borrows cannot be elided and a named
/// lifetime is banned (AGENTS.md rule 1).
macro_rules! call {
    ($name:expr, $input:expr, $stop:expr) => {
        RoutedCall {
            id: "toolu_1",
            name: $name,
            input: $input,
            workspace: Path::new("/w/home"),
            agent: "dulcet-mongoose",
            stop: $stop,
        }
    };
}

/// A clock reading a fixed wall-clock second, for the parseable-stamp arm.
struct At(i64);

impl Clock for At {
    fn now(&self) -> std::time::Instant {
        std::time::Instant::now()
    }
    fn stamp(&self) -> String {
        self.0.to_string()
    }
}

/// An observation is dated in the crate's one human spelling. A stamp that is
/// not seconds reads as the epoch rather than refusing — a wrong date on an
/// observation is worth less than the observation.
#[test]
fn an_answer_is_dated_at_the_instant_it_was_read() {
    let root = TempDir::new().expect("tmp");
    assert_eq!(
        site(root.path(), impatient()).observed(),
        "1970-01-01 00:00:00Z"
    );

    let mut s = site(root.path(), impatient());
    s.clock = Arc::new(At(1_785_630_266));
    assert_eq!(s.observed(), "2026-08-02 00:24:26Z");
}

/// The `clients` tool is in the prefix always, and it is the whole prefix when
/// the verb names no agent — which is what a conversation with no loads reads
/// as too, so there is one shape, not two.
#[test]
fn the_clients_tool_is_declared_on_every_request() {
    let root = TempDir::new().expect("tmp");
    let declared = injection(root.path(), None).tools();
    assert_eq!(declared.len(), 1);
    assert_eq!(declared[0].name, clients::NAME);
    assert_eq!(
        declared[0].description.as_deref(),
        Some(clients::DESCRIPTION)
    );
    assert_eq!(declared[0].input_schema, clients::schema());
}

/// A loaded tool is declared **individually named**, carrying the definition
/// frozen at the load act — never a multiplexer, and never an engine read.
#[test]
fn a_loaded_tool_is_declared_under_its_own_name() {
    let root = TempDir::new().expect("tmp");
    loaded::add(
        root.path(),
        "home",
        "dulcet-mongoose",
        &[loaded::Entry {
            client: "laptop".to_owned(),
            tool: tool("Bash"),
        }],
    )
    .expect("loaded");

    let driving = Some(("home".to_owned(), "dulcet-mongoose".to_owned()));
    let declared = injection(root.path(), driving).tools();
    assert_eq!(
        declared.iter().map(|t| t.name.clone()).collect::<Vec<_>>(),
        vec![clients::NAME.to_owned(), "laptop_Bash".to_owned()]
    );
    assert_eq!(declared[1].input_schema, tool("Bash").input_schema);
    assert_eq!(
        declared[1].description.as_deref(),
        Some("what Bash does"),
        "the description is the frozen one"
    );
}

/// A name the injection does not own is declined, so the executor resolves it
/// exactly as it would with no injection installed.
#[test]
fn an_unowned_name_falls_through_to_the_pool() {
    let root = TempDir::new().expect("tmp");
    let input = json!({});
    let stop = AtomicBool::new(false);
    assert!(
        injection(root.path(), None)
            .route(call!("Read", &input, &stop))
            .is_none()
    );
}

/// The `clients` tool is answered in the stdio vocabulary the executor already
/// speaks: the product on stdout at exit 0.
#[test]
fn a_clients_op_answers_as_a_zero_exit_capture() {
    let root = TempDir::new().expect("tmp");
    let (handle, seen) = engine(
        root.path(),
        &json!({"ok": true, "kind": "clients", "rows": []}),
    );
    let input = json!({"op": "list"});
    let stop = AtomicBool::new(false);
    let capture = Injection::new(
        root.path().to_path_buf(),
        budget(),
        budget(),
        FakeClock::new().arc(),
        None,
    )
    .route(call!(clients::NAME, &input, &stop))
    .expect("owned");
    handle.join().expect("engine");

    assert_eq!(capture.exit_code, 0);
    assert!(capture.stderr.is_empty());
    assert!(
        String::from_utf8_lossy(&capture.stdout).contains("workspace \"home\""),
        "the workspace comes off the call, not the process"
    );
    assert_eq!(
        seen.recv().expect("a request"),
        json!({"op": "clients", "workspace": "home"}),
        "the roster read is Query::Clients — no new verb"
    );
}

/// A refusal is a non-zero capture with the reason on stderr — an in-band
/// result the model steps on, never a harness fault and never a hang.
#[test]
fn a_refusal_is_an_in_band_non_zero_capture() {
    let root = TempDir::new().expect("tmp");
    let input = json!({"op": "nope"});
    let stop = AtomicBool::new(false);
    let capture = injection(root.path(), None)
        .route(call!(clients::NAME, &input, &stop))
        .expect("owned");
    assert_eq!(capture.exit_code, 1);
    assert!(capture.stdout.is_empty());
    let said = String::from_utf8_lossy(&capture.stderr).into_owned();
    assert!(said.starts_with("clients: "), "{said}");
    assert!(said.contains("unknown op"), "{said}");
}

/// The routing leg through the injection (bl-024b) — its own file at §12's
/// per-file budget, on the seam the module itself is cut on: the `clients`
/// tool is one subject and a loaded remote name is another.
mod routing;
