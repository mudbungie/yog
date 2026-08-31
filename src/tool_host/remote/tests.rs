//! The driver's end of the routing leg: the two round trips, and every way
//! each ends in a sentence rather than a wait (REMOTE §5).

use super::*;
use crate::tool_host::tests::{budget, impatient, scripted, site, tool};
use serde_json::json;
use std::time::Duration;
use tempfile::TempDir;

fn entry() -> loaded::Entry {
    loaded::Entry {
        client: "laptop".to_owned(),
        tool: tool("Bash"),
    }
}

fn quiet() -> AtomicBool {
    AtomicBool::new(false)
}

/// The production patience is a real bound and a *different* one from the
/// engine budget — an engine that has not answered is down, a tool that has
/// not answered is working.
#[test]
fn the_tool_bound_is_longer_than_the_engine_bound() {
    let (tool_bound, engine_bound) = (patience(), Budget::default());
    assert!(tool_bound.tick * tool_bound.waits > engine_bound.tick * engine_bound.waits);
}

/// The poll runs more than once: the first look finds nothing, the second
/// finds the capture. That is the arm a tool that takes any time at all takes.
#[test]
fn the_poll_waits_for_a_capture_that_is_not_there_yet() {
    let root = TempDir::new().expect("tmp");
    let (handle, seen) = scripted(
        root.path(),
        &[
            json!({"ok": true, "kind": "routed", "invocation": "inv-7"}),
            json!({"ok": true, "kind": "routed", "invocation": "inv-7"}),
            json!({"ok": true, "kind": "routed", "invocation": "inv-7",
                   "capture": {"stdout": "done", "stderr": "", "exit_code": 0}}),
        ],
    );
    let mut s = site(root.path(), budget());
    s.patience = Budget {
        waits: 8,
        tick: Duration::from_millis(1),
    };
    let got = invoke(&s, &entry(), &json!({"command": "ls"}), None, &quiet());
    handle.join().expect("engine");
    assert_eq!(
        got,
        Ok(Capture {
            stdout: "done".to_owned(),
            stderr: String::new(),
            exit_code: 0,
        })
    );
    assert_eq!(seen.iter().count(), 3, "one queue, two polls");
}

/// **The deadline is the visible refusal** (REMOTE §5): a machine that never
/// answers costs the caller's patience and then a sentence naming the handle
/// it was waiting on — never a hang.
#[test]
fn a_machine_that_never_answers_runs_out_and_says_so() {
    let root = TempDir::new().expect("tmp");
    let (handle, _seen) = scripted(
        root.path(),
        &[
            json!({"ok": true, "kind": "routed", "invocation": "inv-7"}),
            json!({"ok": true, "kind": "routed", "invocation": "inv-7"}),
        ],
    );
    let mut s = site(root.path(), budget());
    s.patience = Budget {
        waits: 1,
        tick: Duration::ZERO,
    };
    let e = invoke(&s, &entry(), &json!({}), None, &quiet()).expect_err("nothing answered");
    handle.join().expect("engine");
    assert!(e.contains("inv-7") && e.contains("laptop"), "{e}");
}

/// A stop landing mid-run ends the wait, which is the router obligation litany
/// states and cannot enforce. The flag is raised by the stand-in engine
/// **before** it writes the answer the driver is waiting on — the hostile
/// order, and the one this test used to be flaky on (bl-3a88): the driver may
/// read the flag inside that window, or read the reply and come back around to
/// the next round trip and read it there. Both are the same fact and now say so
/// in the same sentence, so the assertion no longer depends on which of two
/// nested waits won a race no test can observe. Verified by widening the window
/// with a sleep between the flag and the reply, which reddened this assertion
/// every run before the fix and passes every run after it.
#[test]
fn a_stop_ends_the_wait_on_the_tool() {
    let root = TempDir::new().expect("tmp");
    let stop = AtomicBool::new(false);
    let mut s = site(root.path(), budget());
    s.patience = Budget {
        waits: 400,
        tick: Duration::from_millis(1),
    };
    let got = std::thread::scope(|scope| {
        scope.spawn(|| {
            for served in 0..2 {
                for _ in 0..40_000 {
                    if let Some((id, _)) = crate::boundary::deposit::pending(root.path())
                        .into_iter()
                        .next()
                    {
                        let _ = crate::boundary::deposit::claim(root.path(), &id);
                        if served == 1 {
                            stop.store(true, Ordering::Relaxed);
                        }
                        let _ = crate::boundary::deposit::write_reply(
                            root.path(),
                            &id,
                            &json!({"ok": true, "kind": "routed", "invocation": "inv-7"}),
                        );
                        break;
                    }
                    std::thread::sleep(Duration::from_millis(1));
                }
            }
        });
        invoke(&s, &entry(), &json!({}), None, &stop)
    });
    let said = format!("{got:?}");
    assert!(
        got.is_err_and(|e| e == "stopped while waiting on laptop"),
        "the stop is named, and named the one way: {said}"
    );
}

/// Three ways an engine can answer something other than an invocation, each
/// naming what came back rather than guessing at it.
#[test]
fn an_answer_that_is_not_a_routed_invocation_names_itself() {
    let root = TempDir::new().expect("tmp");
    for (reply, needle) in [
        (json!({"ok": true, "kind": "acked"}), "not a routed"),
        (json!({"ok": false, "error": "no such client"}), "no such"),
        (json!({"ok": true, "kind": "teleported"}), "undecodable"),
    ] {
        let (handle, _seen) = scripted(root.path(), &[reply]);
        let e = invoke(
            &site(root.path(), budget()),
            &entry(),
            &json!({}),
            None,
            &quiet(),
        )
        .expect_err("not an invocation");
        handle.join().expect("engine");
        assert!(e.contains(needle), "{e}");
    }
}

/// No engine at all is the same class of answer, at the first round trip.
#[test]
fn no_engine_is_a_sentence_at_the_first_ask() {
    let root = TempDir::new().expect("tmp");
    let e = invoke(
        &site(root.path(), impatient()),
        &entry(),
        &json!({}),
        None,
        &quiet(),
    )
    .expect_err("no consumer");
    assert!(e.contains("no engine answered"), "{e}");
}
