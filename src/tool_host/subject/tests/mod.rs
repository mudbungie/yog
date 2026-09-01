//! The worktree lane (REMOTE §5.4, bl-77be): a granted, unqualified name
//! reaching the workspace's one consenting machine with the conversation's
//! working directory on the invocation — and the refusals, each naming the
//! way out.

use ::litany::cmd::{RoutedCall, ToolInjection as _};

use crate::test_support::FakeClock;
use crate::tool_host::tests::{budget, scripted};
use crate::tool_host::{Injection, loaded};
use serde_json::json;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use tempfile::TempDir;

/// The lane's injection: no front door (no engine act is exercised here) and
/// the scripted engine's brisk budget on both bounds.
fn injection(root: &Path) -> Injection {
    Injection::new(
        root.to_path_buf(),
        PathBuf::new(),
        budget(),
        budget(),
        FakeClock::new().arc(),
    )
}

/// One invocation, as litany's executor hands it over — a macro for the
/// borrow shape, exactly as the sibling test modules spell it.
macro_rules! call {
    ($name:expr, $input:expr, $stop:expr) => {
        RoutedCall {
            id: "toolu_1",
            name: $name,
            input: $input,
            workspace: Path::new("/w/home"),
            agent: "dulcet-mongoose",
            cwd: Path::new("/w/home/agents/dulcet-mongoose"),
            stop: $stop,
        }
    };
}

/// One advertised element as the engine's roster reply spells it.
fn advertised(name: &str, subject_cwd: bool) -> serde_json::Value {
    let mut tool = json!({"name": name, "description": "a tool",
                          "input_schema": {"type": "object"}});
    if subject_cwd {
        tool["subject_cwd"] = json!(true);
    }
    tool
}

/// A roster reply with one client advertising `tools`.
fn roster(client: &str, tools: &[serde_json::Value]) -> serde_json::Value {
    json!({"ok": true, "kind": "clients",
           "rows": [{"client": client, "present": true, "tools": tools}]})
}

/// **The lane, end to end through the injection**: nothing is loaded, the
/// workspace's one consenting machine advertises `bash`, and the bare granted
/// name routes there with the conversation's resolved working directory on
/// the invocation — the half REMOTE §5 said the thrall move owed. The far
/// machine's capture comes back verbatim.
#[test]
fn a_granted_worktree_name_routes_with_the_conversations_cwd() {
    let root = TempDir::new().expect("tmp");
    let (handle, seen) = scripted(
        root.path(),
        &[
            roster("laptop", &[advertised("bash", true)]),
            json!({"ok": true, "kind": "routed", "invocation": "inv-9"}),
            json!({"ok": true, "kind": "routed", "invocation": "inv-9",
                   "capture": {"stdout": "made\n", "stderr": "", "exit_code": 0}}),
        ],
    );
    let input = json!({"command": "printf made > out.txt"});
    let stop = AtomicBool::new(false);
    let capture = injection(root.path()).route(call!("bash", &input, &stop));
    handle.join().expect("engine");

    assert_eq!(capture.exit_code, 0);
    assert_eq!(capture.stdout, b"made\n");
    assert_eq!(
        seen.recv().expect("the roster read"),
        json!({"op": "clients", "workspace": "home"})
    );
    assert_eq!(
        seen.recv().expect("the queueing act"),
        json!({"op": "invoke", "client": "laptop", "tool": "bash",
               "input": {"command": "printf made > out.txt"},
               "cwd": "/w/home/agents/dulcet-mongoose"}),
        "the invocation carries the subject's location"
    );
    assert_eq!(
        seen.recv().expect("the poll"),
        json!({"op": "capture", "invocation": "inv-9"})
    );
}

/// The lane's last rung: what the engine performs on its own box, and what it
/// still refuses (bl-5710).
mod engine;
/// The sentences: unconsented, ambiguous, and the two transport failures.
mod refusals;

/// The site used by the lane is the call's own, so the loaded set stays
/// consulted first: a loaded qualified name never reaches the lane, and an
/// unqualified one always does — pinned here by the loaded document being
/// present while the bare name still routes by roster.
#[test]
fn a_loaded_set_does_not_shadow_the_lane_for_a_bare_name() {
    let root = TempDir::new().expect("tmp");
    loaded::add(
        root.path(),
        "home",
        "dulcet-mongoose",
        &[loaded::Entry {
            client: "laptop".to_owned(),
            tool: crate::tool_host::tests::tool("bash"),
        }],
    )
    .expect("loaded");
    let (handle, seen) = scripted(
        root.path(),
        &[
            roster("tower", &[advertised("bash", true)]),
            json!({"ok": true, "kind": "routed", "invocation": "inv-2"}),
            json!({"ok": true, "kind": "routed", "invocation": "inv-2",
                   "capture": {"stdout": "", "stderr": "", "exit_code": 0}}),
        ],
    );
    let input = json!({"command": "true"});
    let stop = AtomicBool::new(false);
    // `bash`, not `laptop_bash`: the bare spelling is the lane's.
    let capture = injection(root.path()).route(call!("bash", &input, &stop));
    handle.join().expect("engine");
    assert_eq!(capture.exit_code, 0);
    let _ = seen.recv().expect("the roster read");
    let queued = seen.recv().expect("the queueing act");
    assert_eq!(queued["client"], "tower", "routed by roster, not by loads");
}
