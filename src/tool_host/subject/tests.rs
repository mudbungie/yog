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

/// **Advertised without consent is a refusal naming the key** — the box must
/// opt in before it executes at a path the conversation names, and the
/// sentence carries both ways out: the model's (load the host-bound
/// instance) and the operator's (the config edit, by key, file and box).
#[test]
fn an_unconsenting_advertiser_is_refused_naming_the_remedy() {
    let root = TempDir::new().expect("tmp");
    let (handle, _seen) = scripted(
        root.path(),
        &[roster("laptop", &[advertised("bash", false)])],
    );
    let input = json!({"command": "ls"});
    let stop = AtomicBool::new(false);
    let capture = injection(root.path()).route(call!("bash", &input, &stop));
    handle.join().expect("engine");

    assert_eq!(capture.exit_code, 1);
    let said = String::from_utf8_lossy(&capture.stderr).into_owned();
    assert!(said.contains("laptop advertises bash"), "{said}");
    assert!(
        said.contains("no machine of this workspace consents"),
        "{said}"
    );
    assert!(said.contains("\"subject_cwd\": true"), "{said}");
    assert!(said.contains("clients tool"), "{said}");
}

/// **Two consenting machines is a config ambiguity, refused naming both** —
/// one adjudication decision must stand for exactly one execution on one
/// machine (REMOTE §5, no broadcast).
#[test]
fn two_consenting_machines_are_an_ambiguity_refused_naming_them() {
    let root = TempDir::new().expect("tmp");
    let (handle, _seen) = scripted(
        root.path(),
        &[json!({"ok": true, "kind": "clients", "rows": [
            {"client": "laptop", "present": true,
             "tools": [advertised("bash", true)]},
            {"client": "tower", "present": false,
             "tools": [advertised("bash", true)]},
        ]})],
    );
    let input = json!({"command": "ls"});
    let stop = AtomicBool::new(false);
    let capture = injection(root.path()).route(call!("bash", &input, &stop));
    handle.join().expect("engine");

    assert_eq!(capture.exit_code, 1);
    let said = String::from_utf8_lossy(&capture.stderr).into_owned();
    assert!(said.contains("2 machines consent"), "{said}");
    assert!(said.contains("laptop, tower"), "{said}");
    assert!(said.contains("exactly one entry"), "{said}");
}

/// A consenting machine that never answers the routing leg is the transport
/// sentence every other ask renders — in band, non-zero, never a hang. The
/// scripted engine answers the roster and then goes silent, which is the
/// invoke ask running out its own bound.
#[test]
fn a_lane_whose_invoke_is_never_answered_refuses_in_band() {
    let root = TempDir::new().expect("tmp");
    let (handle, _seen) = scripted(
        root.path(),
        &[roster("laptop", &[advertised("bash", true)])],
    );
    let input = json!({"command": "ls"});
    let stop = AtomicBool::new(false);
    let capture = injection(root.path()).route(call!("bash", &input, &stop));
    handle.join().expect("engine");

    assert_eq!(capture.exit_code, 1);
    let said = String::from_utf8_lossy(&capture.stderr).into_owned();
    assert!(said.starts_with("bash: "), "{said}");
    assert!(said.contains("no engine answered"), "{said}");
}

/// **The lane's first ask can fail too**, and it is refused in band on the same
/// terms the routing leg is: a lane whose *roster* read never lands has no set
/// to select from, so the transport's own sentence is what the model reads —
/// never a hang, and never a silent empty roster read as "nothing advertises
/// it", which would name the wrong remedy.
#[test]
fn a_lane_whose_roster_is_never_answered_refuses_in_band() {
    let root = TempDir::new().expect("tmp");
    let input = json!({"command": "ls"});
    let stop = AtomicBool::new(false);
    let site = crate::tool_host::Site {
        state_root: root.path().to_path_buf(),
        workspace: "home".to_owned(),
        agent: "dulcet-mongoose".to_owned(),
        budget: crate::tool_host::tests::impatient(),
        patience: crate::tool_host::tests::impatient(),
        clock: FakeClock::new().arc(),
    };
    let capture = super::answer(&site, &call!("bash", &input, &stop));

    assert_eq!(capture.exit_code, 1);
    let said = String::from_utf8_lossy(&capture.stderr).into_owned();
    assert!(said.starts_with("bash: "), "{said}");
    assert!(said.contains("no engine answered"), "{said}");
    assert!(
        !said.contains("advertises"),
        "a failed roster read is a transport fact, not a selection one: {said}"
    );
}

/// The selection is pure over the roster, so the one-name-two-entries shapes
/// are provable without an engine: a duplicate name inside one client's set
/// cannot reach here (the advertisement refuses it), but one consenting and
/// one not across clients selects the consenting one alone.
#[test]
fn one_consenting_machine_wins_over_an_unconsenting_advertiser() {
    let rows = vec![
        crate::registry::roster::ClientRow {
            client: "laptop".to_owned(),
            present: true,
            tools: vec![crate::registry::tools::Tool {
                name: "bash".to_owned(),
                description: "a tool".to_owned(),
                input_schema: json!({"type": "object"}),
                subject_cwd: false,
            }],
        },
        crate::registry::roster::ClientRow {
            client: "tower".to_owned(),
            present: true,
            tools: vec![crate::registry::tools::Tool {
                name: "bash".to_owned(),
                description: "a tool".to_owned(),
                input_schema: json!({"type": "object"}),
                subject_cwd: true,
            }],
        },
    ];
    let picked = super::verdict(&rows, "bash").expect("one consenting machine");
    assert_eq!(picked.client, "tower");
    assert!(picked.tool.subject_cwd);
    // And a name nothing advertises earns the loadless sentence with both
    // remedies, provable from the same pure function.
    let said = super::verdict(&rows, "read_file").expect_err("nothing advertises it");
    assert!(said.contains("no machine of this workspace advertises read_file"));
    assert!(said.contains("clients tool"), "{said}");
    assert!(said.contains("\"subject_cwd\": true"), "{said}");
}

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
