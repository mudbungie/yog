//! The routed leg, driven against a stand-in server over a real `gestures/`
//! inbox: how a name is read, what crosses, and the two ways a call fails.

use super::*;
use crate::tool_host::tests::{budget, scripted};
use serde_json::{Value, json};
use std::path::Path;
use std::time::Duration;
use tempfile::TempDir;

fn names(of: &[&str]) -> Vec<String> {
    of.iter().map(|n| (*n).to_owned()).collect()
}

fn far(root: &Path, waits: u32) -> Far {
    Far {
        state_root: root.to_path_buf(),
        client: "laptop".to_owned(),
        argv: names(&["--json", "--provider", "sub"]),
        budget: budget(),
        patience: Budget {
            waits,
            tick: Duration::from_millis(1),
        },
        events: true,
    }
}

/// Run the leg against `replies`; answer `(exit, stdout, stderr, requests)`.
fn drive(far: &Far, replies: &[Value]) -> (i32, String, String, Vec<Value>) {
    let (handle, seen) = scripted(&far.state_root, replies);
    let (mut out, mut err) = (Vec::new(), Vec::new());
    let exit = far.run(&mut "the request".as_bytes(), &mut out, &mut err);
    handle.join().expect("server");
    (
        exit,
        String::from_utf8(out).expect("utf8"),
        String::from_utf8(err).expect("utf8"),
        seen.iter().collect(),
    )
}

fn queued() -> Value {
    json!({"ok": true, "kind": "routed", "invocation": "inv-1"})
}

fn partial(stdout: &str, stderr: &str) -> Value {
    json!({"ok": true, "kind": "routed", "invocation": "inv-1",
           "partial": {"stdout": stdout, "stderr": stderr}})
}

/// The two lines a call nothing was relayed for answers, decoded with brazen's
/// own types — the reader litany uses.
fn refusal(stdout: &str) -> brazen::CanonicalError {
    let events: Vec<brazen::Event> = stdout
        .lines()
        .map(|line| serde_json::from_str(line).expect("canonical event"))
        .collect();
    match events.as_slice() {
        [brazen::Event::Error(error), brazen::Event::End] => error.clone(),
        other => panic!("not one error and the terminator: {other:?}"),
    }
}

/// How a name is read, and which calls the seam routes.
mod name;

/// The whole turn: the command line and the request cross as the tool's
/// input, every poll states what is already held, and each piece is on stdout
/// in the order it arrived — the capture's text being only the remainder.
#[test]
fn the_far_stream_is_relayed_in_the_pieces_it_arrived_in() {
    let root = TempDir::new().expect("tmp");
    let (exit, out, err, asked) = drive(
        &far(root.path(), 1000),
        &[
            queued(),
            queued(),
            partial("a\n", ""),
            partial("b\n", "e"),
            json!({"ok": true, "kind": "routed", "invocation": "inv-1",
                   "capture": {"stdout": "c\n", "stderr": "f", "exit_code": 0}}),
        ],
    );
    assert_eq!((exit, out.as_str(), err.as_str()), (0, "a\nb\nc\n", "ef"));
    assert_eq!(
        asked.first(),
        Some(&json!({"op": "invoke", "client": "laptop", "tool": "bz",
            "input": {"args": ["--json", "--provider", "sub"], "stdin": "the request"}}))
    );
    let since: Vec<&Value> = asked.iter().skip(1).map(|ask| &ask["since"]).collect();
    assert_eq!(
        since,
        [
            &json!({"stdout": 0, "stderr": 0}),
            &json!({"stdout": 0, "stderr": 0}),
            &json!({"stdout": 2, "stderr": 0}),
            &json!({"stdout": 4, "stderr": 1}),
        ]
    );
}

/// Nobody took it inside the patience: one non-retryable error naming the
/// client and that it has never connected, then the terminator.
#[test]
fn a_call_nobody_took_is_one_error_event_and_the_terminator() {
    let root = TempDir::new().expect("tmp");
    let (exit, out, err, asked) = drive(&far(root.path(), 0), &[queued(), queued()]);
    let error = refusal(&out);
    assert_eq!(error.kind, brazen::ErrorKind::Other("far_call".to_owned()));
    assert!(!error.retryable());
    assert!(error.message.contains("\"laptop\""), "{}", error.message);
    assert!(
        error.message.contains("never connected"),
        "{}",
        error.message
    );
    assert_eq!((exit, err.as_str(), asked.len()), (70, "", 2));
}

/// The same failure names when the machine last spoke, off the roster's file.
#[test]
fn the_error_says_when_the_machine_last_spoke() {
    let root = TempDir::new().expect("tmp");
    let laptop = Client::parse("laptop").expect("client");
    seen::mark(root.path(), &laptop, 86_400);
    let (_, out, _, _) = drive(&far(root.path(), 0), &[queued(), queued()]);
    let message = refusal(&out).message;
    assert!(
        message.contains("last connected at 1970-01-02"),
        "{message}"
    );
}

/// **Busy means wait** (REMOTE §3.3, bl-d9ad): a machine that spoke a moment
/// ago is working, and a call queued behind its job is waited for however
/// long the patience is — here, no time at all.
#[test]
fn a_call_queued_behind_a_working_machine_outlives_the_patience() {
    let root = TempDir::new().expect("tmp");
    let laptop = Client::parse("laptop").expect("client");
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("after the epoch")
        .as_secs();
    seen::mark(
        root.path(),
        &laptop,
        i64::try_from(now).expect("this century"),
    );
    let (exit, out, _, _) = drive(
        &far(root.path(), 0),
        &[
            queued(),
            queued(),
            queued(),
            json!({"ok": true, "kind": "routed", "invocation": "inv-1",
                   "capture": {"stdout": "ran\n", "stderr": "", "exit_code": 0}}),
        ],
    );
    assert_eq!((exit, out.as_str()), (0, "ran\n"));
}

/// A machine that has posted is running the call: silence after that is the
/// server's to judge, and the patience is not spent on it.
#[test]
fn a_taken_call_outlives_the_patience() {
    let root = TempDir::new().expect("tmp");
    let (exit, out, _, _) = drive(
        &far(root.path(), 1),
        &[
            queued(),
            partial("", ""),
            partial("", ""),
            partial("", ""),
            json!({"ok": true, "kind": "routed", "invocation": "inv-1",
                   "capture": {"stdout": "late\n", "stderr": "", "exit_code": 3}}),
        ],
    );
    assert_eq!((exit, out.as_str()), (3, "late\n"));
}

/// A refused ask has relayed nothing, so it is the same in-band answer.
#[test]
fn a_refused_ask_is_the_error_event_too() {
    let root = TempDir::new().expect("tmp");
    let (exit, out, _, _) = drive(
        &far(root.path(), 9),
        &[json!({"ok": false, "error": "client \"laptop\" advertises no tool \"bz\" right now"})],
    );
    assert!(refusal(&out).message.contains("advertises no tool"));
    assert_eq!(exit, 70);
}

/// After a byte has gone out nothing is added to the stream: no terminator,
/// the reason on stderr, a non-zero exit — the half stream litany reads.
#[test]
fn a_failure_after_bytes_were_relayed_adds_nothing_to_the_stream() {
    let root = TempDir::new().expect("tmp");
    let (exit, out, err, _) = drive(
        &far(root.path(), 9),
        &[
            queued(),
            partial("a\n", ""),
            json!({"ok": false, "error": "no invocation \"inv-1\" is in flight"}),
        ],
    );
    assert_eq!((exit, out.as_str()), (1, "a\n"));
    assert!(err.contains("yog bz: no invocation"), "{err}");
}

/// A listing or a sign-in has no canonical stream to answer in: the same
/// sentence goes to stderr and stdout stays empty.
#[test]
fn a_call_that_is_not_a_model_call_fails_on_stderr() {
    let root = TempDir::new().expect("tmp");
    let far = Far {
        events: false,
        ..far(root.path(), 0)
    };
    let (exit, out, err, _) = drive(&far, &[queued(), queued()]);
    assert_eq!((exit, out.as_str()), (1, ""));
    assert!(
        err.contains("yog bz: client \"laptop\" did not take"),
        "{err}"
    );
}

/// A request that cannot be read.
struct Broken;

impl Read for Broken {
    fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
        Err(std::io::Error::other("closed"))
    }
}

#[test]
fn a_reader_that_went_away_ends_the_call() {
    let root = TempDir::new().expect("tmp");
    let far = far(root.path(), 9);
    let (handle, _seen) = scripted(root.path(), &[queued(), partial("a\n", "")]);
    let mut err = Vec::new();
    // A stream that takes nothing: a file opened for reading refuses a write.
    let mut closed = std::fs::File::open(root.path()).expect("open");
    let exit = far.run(&mut "r".as_bytes(), &mut closed, &mut err);
    handle.join().expect("server");
    assert_eq!(exit, 1);
    assert!(String::from_utf8_lossy(&err).contains("relaying the stream"));
}

#[test]
fn an_unreadable_request_is_refused_before_anything_is_asked() {
    let root = TempDir::new().expect("tmp");
    let mut out = Vec::new();
    let exit = far(root.path(), 9).run(&mut Broken, &mut out, &mut Vec::new());
    let message = refusal(&String::from_utf8_lossy(&out)).message;
    assert!(message.contains("reading the request"), "{message}");
    assert_eq!(exit, 70);
}

/// The seam itself: `bz_host::run` hands a qualified call to this leg, and
/// what comes back is this leg's answer rather than the linked brazen's.
#[test]
fn the_runner_seam_takes_the_routed_leg_for_a_qualified_name() {
    let root = TempDir::new().expect("tmp");
    let env = name::world(root.path(), "alba");
    let (handle, seen) = scripted(
        &env.yog_state_root(),
        &[json!({"ok": false, "error": "refused by the stand-in"})],
    );
    let mut out = Vec::new();
    let exit = crate::bz_host::run(
        names(&["--json", "--provider", "sub@laptop"]),
        &env,
        crate::bz_host::Tty::PIPED,
        &mut "r".as_bytes(),
        &mut out,
        &mut Vec::new(),
    );
    handle.join().expect("server");
    assert!(
        refusal(&String::from_utf8_lossy(&out))
            .message
            .contains("stand-in")
    );
    assert_eq!(exit, 70);
    assert_eq!(seen.iter().next().expect("asked")["tool"], "bz");
}
