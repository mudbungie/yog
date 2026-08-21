//! Running one invocation locally: the tool contract, the four verdicts, and
//! the deadline that is a drop (REMOTE §5.2).

use super::*;
use crate::registry::tools::Tool;
use serde_json::json;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use tempfile::tempdir;

/// A tool executable, written the way an operator writes one: it reads the
/// invocation's JSON off stdin and answers on stdout.
fn script(dir: &Path, name: &str, body: &str) -> PathBuf {
    let path = dir.join(name);
    fs::write(&path, format!("#!/bin/sh\n{body}\n")).expect("script");
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).expect("chmod");
    path
}

fn local(command: Vec<String>, cwd: Option<PathBuf>) -> Local {
    Local {
        tool: Tool {
            name: "Bash".to_owned(),
            description: "run a command".to_owned(),
            input_schema: json!({"type": "object"}),
        },
        command,
        cwd,
    }
}

fn invocation(tool: &str) -> Invocation {
    Invocation {
        id: "inv-1".to_owned(),
        tool: tool.to_owned(),
        input: json!({"command": "ls"}),
    }
}

fn patient() -> Duration {
    Duration::from_secs(30)
}

/// **lernie's own tool contract, one for one** (its ARCH §3.3): the input JSON
/// on stdin, bytes on stdout, the exit code the verdict — and stderr beside
/// them, so a tool that warned and succeeded says both.
#[test]
fn the_input_arrives_on_stdin_and_the_three_facts_come_back() {
    let dir = tempdir().expect("tmp");
    let tool = script(dir.path(), "echo-tool", "cat; echo 'warned' >&2; exit 0");
    let set = vec![local(vec![tool.to_string_lossy().into_owned()], None)];
    let capture = execute(&set, &invocation("Bash"), patient());
    assert_eq!(capture.stdout, r#"{"command":"ls"}"#);
    assert_eq!(capture.stderr, "warned\n");
    assert_eq!(capture.exit_code, 0);
}

/// The argv is spawned **directly** — extra words are arguments, never a shell
/// line — and `cwd` is where it runs.
#[test]
fn the_argv_and_the_working_directory_are_the_operators() {
    let dir = tempdir().expect("tmp");
    let elsewhere = dir.path().join("elsewhere");
    fs::create_dir(&elsewhere).expect("cwd");
    let tool = script(dir.path(), "where-tool", "echo \"$1 $(pwd)\"");
    let set = vec![local(
        vec![tool.to_string_lossy().into_owned(), "arg one".to_owned()],
        Some(elsewhere.clone()),
    )];
    let capture = execute(&set, &invocation("Bash"), patient());
    assert_eq!(capture.exit_code, 0);
    assert!(capture.stdout.starts_with("arg one "), "{capture:?}");
    assert!(
        capture
            .stdout
            .contains(&elsewhere.to_string_lossy().into_owned()),
        "{capture:?}"
    );
}

/// A tool's own non-zero verdict is the capture's verdict — a tool that failed
/// is an answer, not a fault of the host's.
#[test]
fn a_tools_own_failure_is_its_verdict() {
    let dir = tempdir().expect("tmp");
    let tool = script(dir.path(), "fail-tool", "echo 'nope' >&2; exit 7");
    let set = vec![local(vec![tool.to_string_lossy().into_owned()], None)];
    let capture = execute(&set, &invocation("Bash"), patient());
    assert_eq!(capture.exit_code, 7);
    assert_eq!(capture.stderr, "nope\n");
}

/// **The deadline is the drop** — the SIGTERM-then-SIGKILL cascade the crate
/// already owns. What comes back is a capture, with the shell's own timeout
/// verdict and a sentence saying what happened, never a hang.
#[test]
fn a_tool_that_outruns_its_deadline_is_terminated_and_says_so() {
    let dir = tempdir().expect("tmp");
    let tool = script(dir.path(), "slow-tool", "echo 'starting'; sleep 30");
    let set = vec![local(vec![tool.to_string_lossy().into_owned()], None)];
    let capture = execute(&set, &invocation("Bash"), Duration::from_millis(120));
    assert_eq!(capture.exit_code, TIMED_OUT);
    assert!(capture.stderr.contains("terminated"), "{capture:?}");
    assert!(
        capture.stdout.contains("starting"),
        "what it did say is kept: {capture:?}"
    );
}

/// **REMOTE §5's staleness correction, answered at the end that knows**: a name
/// this machine no longer carries, and a command that cannot be spawned at all,
/// are both in-band captures the model reads as a tool that failed.
#[test]
fn a_name_this_machine_cannot_run_is_an_in_band_capture() {
    let dir = tempdir().expect("tmp");
    let set = vec![local(
        vec![
            dir.path()
                .join("no-such-tool")
                .to_string_lossy()
                .into_owned(),
        ],
        None,
    )];

    let gone = execute(&set, &invocation("Rm"), patient());
    assert_eq!(gone.exit_code, NO_SUCH_TOOL);
    assert!(gone.stdout.is_empty());
    assert!(gone.stderr.contains("\"Rm\""), "{gone:?}");

    let unspawnable = execute(&set, &invocation("Bash"), patient());
    assert_eq!(unspawnable.exit_code, NO_SUCH_TOOL);
    assert!(
        unspawnable.stderr.contains("no-such-tool"),
        "{unspawnable:?}"
    );
}

/// A tool whose bytes are not UTF-8 loses exactly what no `String` can name —
/// the transcode is here, once, and it never refuses.
#[test]
fn output_that_is_not_utf8_is_transcoded_rather_than_refused() {
    let dir = tempdir().expect("tmp");
    let tool = script(dir.path(), "binary-tool", "printf 'a\\377b'");
    let set = vec![local(vec![tool.to_string_lossy().into_owned()], None)];
    let capture = execute(&set, &invocation("Bash"), patient());
    assert_eq!(capture.exit_code, 0);
    assert_eq!(capture.stdout, "a\u{fffd}b");
}
