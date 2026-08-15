//! The injection itself: what it declares, what it answers, and what it
//! declines (REMOTE §5, bl-c907).

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

/// A stand-in engine: answer the first deposit that lands with `reply`, hand
/// the request back over a channel, and stop. It is the deposit consumer's
/// contract and nothing else — claim, then reply — because that contract is
/// the whole of what the driver's ask depends on.
pub(super) fn engine(root: &Path, reply: &Value) -> (JoinHandle<()>, Receiver<Value>) {
    let root = root.to_path_buf();
    let reply = reply.clone();
    let (tx, rx) = std::sync::mpsc::channel();
    let handle = std::thread::spawn(move || {
        for _ in 0..4000 {
            if let Some((id, path)) = deposit::pending(&root).into_iter().next() {
                let request = std::fs::read(&path)
                    .ok()
                    .and_then(|b| serde_json::from_slice(&b).ok())
                    .unwrap_or(Value::Null);
                let _ = deposit::claim(&root, &id);
                let _ = deposit::write_reply(&root, &id, &reply);
                let _ = tx.send(request);
                return;
            }
            std::thread::sleep(Duration::from_millis(1));
        }
    });
    (handle, rx)
}

/// A budget that answers fast when an engine is there and gives up fast when
/// it is not.
pub(super) fn budget() -> ask::Budget {
    ask::Budget {
        waits: 4000,
        tick: Duration::from_millis(1),
    }
}

/// A budget with no patience at all — the "no engine" path, without the wait.
pub(super) fn impatient() -> ask::Budget {
    ask::Budget {
        waits: 1,
        tick: Duration::ZERO,
    }
}

pub(super) fn tool(name: &str) -> Tool {
    Tool {
        name: name.to_owned(),
        description: format!("what {name} does"),
        input_schema: json!({"type": "object"}),
    }
}

pub(super) fn site(root: &Path, budget: ask::Budget) -> Site {
    Site {
        state_root: root.to_path_buf(),
        workspace: "home".to_owned(),
        agent: "dulcet-mongoose".to_owned(),
        budget,
        clock: FakeClock::new().arc(),
    }
}

fn injection(root: &Path, driving: Option<(String, String)>) -> Injection {
    Injection::new(
        root.to_path_buf(),
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

/// **The residual, asserted** (REMOTE §9 step 7, bl-024b): a loaded name is
/// owned and routed, and the routing says — in band, non-zero — that the leg
/// carrying it to the host is not built. Nothing hangs, and nothing lies.
#[test]
fn a_loaded_remote_name_is_routed_and_refused_in_band() {
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

    let input = json!({"command": "ls"});
    let stop = AtomicBool::new(false);
    let capture = injection(root.path(), None)
        .route(call!("laptop_Bash", &input, &stop))
        .expect("owned");
    assert_eq!(capture.exit_code, 1);
    let said = String::from_utf8_lossy(&capture.stderr).into_owned();
    assert!(said.starts_with("laptop_Bash: "), "{said}");
    assert!(said.contains("bl-024b"), "{said}");
    assert!(
        said.contains("Nothing on \"laptop\" was contacted"),
        "{said}"
    );
}
