//! The injection itself: what it declares, what it answers, and what it
//! declines (REMOTE §5, bl-c907).

/// **The substrate every beat in this family drives against** (REMOTE §5) — the
/// stand-in engines that hold up the deposit consumer's contract and nothing
/// else, the two budgets, and the `Site` an injection answers at. Its own file
/// at §12's per-file budget: four sibling test modules import it, so it is a
/// shared subject rather than this file's own scaffolding.
mod stand_in;

pub(super) use stand_in::{budget, engine, front_door, impatient, scripted, site, tool};

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

fn injection(root: &Path) -> Injection {
    at(root, root.join("no-such-front-door"), impatient())
}

/// An injection with every knob named — the front door it re-enters for an
/// engine act, and the budget both its waits are bounded by.
fn at(root: &Path, driver_target: PathBuf, budget: ask::Budget) -> Injection {
    Injection::new(
        root.to_path_buf(),
        driver_target,
        budget,
        budget,
        FakeClock::new().arc(),
    )
}

/// One invocation, as litany's executor hands it over. A macro rather than a
/// function because three independent borrows cannot be elided and a named
/// lifetime is banned (AGENTS.md rule 1).
macro_rules! call {
    ($name:expr, $input:expr, $stop:expr) => {
        call!($name, $input, $stop, Path::new("/w/home"))
    };
    ($name:expr, $input:expr, $stop:expr, $workspace:expr) => {
        RoutedCall {
            id: "toolu_1",
            name: $name,
            input: $input,
            workspace: $workspace,
            agent: "dulcet-mongoose",
            cwd: $workspace,
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

/// The `clients` tool is in the prefix always, and it is the whole prefix for
/// an agent with no loads — a fresh conversation's shape, whatever verb fired
/// its driver (bl-fd24).
#[test]
fn the_clients_tool_is_declared_on_every_request() {
    let root = TempDir::new().expect("tmp");
    let declared = injection(root.path()).tools(Path::new("/w/home"), "dulcet-mongoose");
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

    let declared = injection(root.path()).tools(Path::new("/w/home"), "dulcet-mongoose");
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

/// **A name the injection does not own is a refusal it renders**, because the
/// router is total and nothing resolves a binary behind it. With nothing
/// loaded and no machine advertising the name, the worktree lane (bl-77be)
/// renders the loadless sentence with both remedies — the ship-inert posture
/// working — while the compactor's engine acts still go through, so nothing
/// about compaction depends on a machine being enrolled. `Read` is the name
/// under test precisely because the engine does not implement it: the three
/// that it does take the lane's last rung instead (bl-5710,
/// `subject::PERFORMED`), which is that ball's whole subject.
#[test]
fn nothing_loaded_refuses_an_ordinary_tool_in_band_and_still_compacts() {
    let root = TempDir::new().expect("tmp");
    let door = front_door(root.path(), "printf compacted");
    let input = json!({});
    let stop = AtomicBool::new(false);
    let live = at(root.path(), door, budget());

    let (engine, _seen) = scripted(
        root.path(),
        &[json!({"ok": true, "kind": "clients", "rows": []})],
    );
    let refused = live.route(call!("Read", &input, &stop));
    engine.join().expect("engine");
    assert_eq!(refused.exit_code, 1);
    assert!(refused.stdout.is_empty());
    let said = String::from_utf8_lossy(&refused.stderr).into_owned();
    assert!(
        said.starts_with("Read: no tool of that name is loaded"),
        "{said}"
    );
    assert!(
        said.contains("no machine of this workspace advertises"),
        "{said}"
    );
    assert!(said.contains("use the clients tool"), "{said}");

    for name in engine_act::NAMES {
        let acted = live.route(call!(name, &input, &stop, root.path()));
        assert_eq!(
            acted.exit_code, 0,
            "{name} is an engine act, not a tool call"
        );
        assert_eq!(acted.stdout, b"compacted");
    }
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
    let capture =
        at(root.path(), PathBuf::new(), budget()).route(call!(clients::NAME, &input, &stop));
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
    let capture = injection(root.path()).route(call!(clients::NAME, &input, &stop));
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
