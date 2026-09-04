//! **The lane's last rung** (bl-5710, operator ruling 2026-08-31: *ship some
//! basic tools — a default install must be able to write a file*): with no
//! machine consenting, the three names the engine implements are performed at
//! the engine's own front door, and everything else still earns the sentence.

use ::litany::cmd::{RoutedCall, ToolInjection as _};

use super::super::{Lane, performs, verdict};
use crate::registry::roster::ClientRow;
use crate::registry::tools::Tool;
use crate::test_support::FakeClock;
use crate::tool_host::Injection;
use crate::tool_host::tests::{budget, front_door, scripted};
use serde_json::json;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use tempfile::TempDir;

/// One invocation whose working directory is a directory that exists — the
/// rung spawns a child there, so a fictional cwd would fail on the spawn
/// rather than on the act. A macro for the borrow shape, as the siblings.
macro_rules! act {
    ($name:expr, $cwd:expr, $input:expr, $stop:expr) => {
        RoutedCall {
            id: "toolu_1",
            name: $name,
            input: $input,
            workspace: Path::new("/w/home"),
            agent: "dulcet-mongoose",
            cwd: $cwd,
            stop: $stop,
        }
    };
}

/// One roster row, built directly so the selection can be proved pure.
fn row(client: &str, name: &str, subject_cwd: bool) -> ClientRow {
    ClientRow {
        client: client.to_owned(),
        present: true,
        tools: vec![Tool {
            name: name.to_owned(),
            description: "a tool".to_owned(),
            input_schema: json!({"type": "object"}),
            subject_cwd,
        }],
    }
}

/// **The audit of a partition yog derives rather than restates** (bl-e654).
/// `performs` is `litany::cmd::BUILTIN_TOOLS` minus the conversation-subject
/// engine acts, so nothing in production spells these names; the literals live
/// here, where a set that moves upstream reddens a test instead of quietly
/// changing which lane a name takes. Three members today, each a worktree name
/// the engine implements — and the engine grew two built-ins at litany 0.0.8
/// without growing this side, because `python` and `search_history` were
/// classified as engine acts in the same breath (bl-fe43, bl-81cc), which is
/// exactly the deliberate act this audit exists to force. What is deliberately
/// outside: the six built-ins whose subject is the conversation (they are
/// `engine_act`'s and never reach the lane), the compactor's injected pair (no
/// built-in at all), an operator-granted pool name with no implementation to
/// reach, and a name that differs only in case.
#[test]
fn the_partition_leaves_the_engine_exactly_the_three_worktree_names() {
    let performed: Vec<&str> = ::litany::cmd::BUILTIN_TOOLS
        .into_iter()
        .filter(|name| performs(name))
        .collect();
    assert_eq!(performed, ["apply_patch", "bash", "read_file"]);
    for elsewhere in [
        "cd",
        "dispatch",
        "message",
        "load_skill",
        "write_summary",
        "mark_for_deletion",
        "python",
        "search_history",
        "deploy",
        "Bash",
    ] {
        assert!(
            !performs(elsewhere),
            "{elsewhere} is not the engine's to perform on the lane"
        );
    }
    // The subtraction is total in the other direction too: every built-in the
    // engine ships is either an engine act or a performed worktree name, so no
    // built-in falls through to a refusal that would tell the operator to
    // enroll a machine for work the engine can already do.
    for name in ::litany::cmd::BUILTIN_TOOLS {
        assert!(
            performs(name) || crate::tool_host::engine_act::is(name),
            "{name} is a built-in on neither lane"
        );
    }
}

/// **The ladder's order, proved pure over the roster.** A consenting machine
/// is an enrollment plus a `subject_cwd` key, so it wins even for a name the
/// engine implements; with nothing consenting, a performed name is the
/// engine's and a pool name is a sentence.
#[test]
fn a_consenting_machine_wins_and_the_engine_takes_what_is_left() {
    let mixed = vec![row("laptop", "bash", false), row("tower", "bash", true)];
    match verdict(&mixed, "bash") {
        Lane::Machine(picked) => {
            assert_eq!(picked.client, "tower");
            assert!(picked.tool.subject_cwd);
        }
        _ => panic!("one consenting machine is that machine's"),
    }
    // The same roster, and a name it advertises without consent anywhere:
    // the engine implements it, so the engine performs it.
    let unconsented = vec![row("laptop", "bash", false)];
    assert!(matches!(verdict(&unconsented, "bash"), Lane::Engine));
    assert!(matches!(verdict(&[], "apply_patch"), Lane::Engine));
    assert!(matches!(verdict(&[], "read_file"), Lane::Engine));
    // And a pool name the engine cannot perform keeps the operator's remedy —
    // the only one that puts the work where its subject is (bl-68e1). The
    // load the model could take unaided is named as what it is NOT, so a
    // refusal can never again be the step that sends a deliverable to a
    // directory nothing at the boundary reads.
    match verdict(&[], "deploy") {
        Lane::Refused(said) => {
            assert!(
                said.contains("this engine does not implement deploy"),
                "{said}"
            );
            assert!(
                said.contains("no machine of this workspace advertises it"),
                "{said}"
            );
            assert!(said.contains("\"subject_cwd\": true"), "{said}");
            assert!(
                said.contains("clients tool is not a way to do this work"),
                "{said}"
            );
            assert!(
                !said.contains("load what one advertises"),
                "the refusal must not steer to another machine's directory: {said}"
            );
        }
        _ => panic!("a name nothing implements and nothing advertises is refused"),
    }
}

/// **A default install writes a file**, end to end through the injection: no
/// client is registered, the roster comes back empty, and `apply_patch`
/// reaches the engine's own front door — the `tool` verb, the caller identity
/// on the child's environment, the `tool_use` input on its stdin, and the
/// child's product back untouched, at the conversation's own working
/// directory.
#[test]
fn an_empty_roster_performs_a_worktree_name_at_the_front_door() {
    let root = TempDir::new().expect("tmp");
    let door = front_door(
        root.path(),
        "printf '%s|%s|%s|%s|%s' \"$1\" \"$2\" \"$LITANY_CONV_BRANCH\" \"$(pwd)\" \"$(cat)\"",
    );
    let cwd = root.path().join("worktree");
    std::fs::create_dir(&cwd).expect("worktree");
    let (handle, seen) = scripted(
        root.path(),
        &[json!({"ok": true, "kind": "clients", "rows": []})],
    );
    let input = json!({"input": "*** Begin Patch\n*** End Patch\n"});
    let stop = AtomicBool::new(false);
    let capture = Injection::new(
        root.path().to_path_buf(),
        door,
        budget(),
        budget(),
        FakeClock::new().arc(),
    )
    .route(act!("apply_patch", &cwd, &input, &stop));
    handle.join().expect("engine");

    assert_eq!(capture.exit_code, 0);
    assert!(capture.stderr.is_empty());
    let said = String::from_utf8_lossy(&capture.stdout).into_owned();
    let want = format!(
        "tool|apply_patch|dulcet-mongoose|{}|{}",
        cwd.canonicalize().expect("cwd").display(),
        json!({"input": "*** Begin Patch\n*** End Patch\n"})
    );
    assert_eq!(said, want);
    assert_eq!(
        seen.recv().expect("the roster read"),
        json!({"op": "clients", "workspace": "home"}),
        "the roster is still asked first: a consenting machine must be able to win"
    );
}

/// A pool name the engine does not implement is unchanged by the rung: the
/// empty roster still earns the sentence that names the enrollment, and no
/// child is spawned (the front door here does not exist, so a spawn would
/// surface as a different sentence entirely).
#[test]
fn an_empty_roster_still_refuses_a_name_the_engine_cannot_perform() {
    let root = TempDir::new().expect("tmp");
    let (handle, _seen) = scripted(
        root.path(),
        &[json!({"ok": true, "kind": "clients", "rows": []})],
    );
    let input = json!({"target": "staging"});
    let stop = AtomicBool::new(false);
    let capture = Injection::new(
        root.path().to_path_buf(),
        PathBuf::new(),
        budget(),
        budget(),
        FakeClock::new().arc(),
    )
    .route(act!("deploy", root.path(), &input, &stop));
    handle.join().expect("engine");

    assert_eq!(capture.exit_code, 1);
    let said = String::from_utf8_lossy(&capture.stderr).into_owned();
    assert!(said.starts_with("deploy: "), "{said}");
    assert!(
        said.contains("no machine of this workspace advertises it"),
        "{said}"
    );
    assert!(said.contains("enrolls a thrall"), "{said}");
    assert!(
        said.contains("clients tool is not a way to do this work"),
        "{said}"
    );
}
