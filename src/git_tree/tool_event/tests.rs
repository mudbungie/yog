//! What one per-call record says, and the spelling that carries it (bl-5305).

use super::{ToolEvent, calls, captured, complete, event_of, event_value, posted};
use serde_json::json;
use std::path::{Path, PathBuf};

/// One step directory with a call landed in it, exactly as litany lands the
/// pair: `input.json` before the call is dispatched, `output.json` when the
/// capture returns.
fn call(step: &Path, id: &str, input: &serde_json::Value) -> PathBuf {
    let dir = step.join(super::TOOLS_SUBDIR).join(id);
    std::fs::create_dir_all(&dir).expect("call dir");
    std::fs::write(
        dir.join("input.json"),
        serde_json::to_vec(input).expect("input"),
    )
    .expect("write input");
    dir
}

fn capture(dir: &Path, exit_code: i32) {
    std::fs::write(
        dir.join("output.json"),
        serde_json::to_vec(&json!({
            "stdout": "", "stderr": "", "exit_code": exit_code,
            "started_at": "t0", "ended_at": "t1"
        }))
        .expect("output"),
    )
    .expect("write output");
}

/// **The beat this ball exists for.** The opening event names the tool and what
/// it was asked to run, and the closing one names the status it came back with
/// — the two facts an operator watching a command land on their server needs
/// and had no way to read live.
#[test]
fn a_call_says_what_it_runs_when_it_opens_and_how_it_ended_when_it_closes() {
    let step = tempfile::tempdir().expect("tmp");
    let dir = call(
        step.path(),
        "toolu_01a",
        &json!({"id": "toolu_01a", "name": "box2_Bash",
                "input": {"command": "hostname && uptime"}}),
    );
    let open = posted("toolu_01a", &dir);
    assert_eq!(
        open.name.as_deref(),
        Some("box2_Bash"),
        "the tool, verbatim"
    );
    assert!(
        open.input.as_deref().is_some_and(|i| i.contains("uptime")),
        "and the command line it was handed: {:?}",
        open.input
    );
    assert_eq!(open.exit_code, None, "nothing has come back yet");
    assert!(!captured(&dir), "and the window is open");

    capture(&dir, 3);
    assert!(captured(&dir), "the capture landed");
    let closed = complete("toolu_01a", &dir);
    assert_eq!(closed.exit_code, Some(3), "with the status it earned");
    assert_eq!(closed.tool_use, "toolu_01a", "on the same identity");
    assert_eq!(
        (closed.name, closed.input),
        (None, None),
        "and it restates neither fact the opening event already carried"
    );
}

/// The client a routed call ran on is already in the name REMOTE §5.1 presents
/// it under, so reading it costs no join — the module's whole claim.
#[test]
fn a_routed_call_names_the_machine_it_ran_on_through_its_own_name() {
    let step = tempfile::tempdir().expect("tmp");
    let dir = call(
        step.path(),
        "toolu_01b",
        &json!({"id": "toolu_01b", "name": "box2_Bash", "input": {}}),
    );
    assert!(
        posted("toolu_01b", &dir)
            .name
            .is_some_and(|n| n.starts_with("box2_")),
        "the presented name is <client>_<tool>"
    );
}

/// The listing is the calls that have a landed input, in call order — a
/// directory mid-creation is not a call, and `read_dir` order is not an order.
#[test]
fn the_listing_is_landed_calls_in_id_order() {
    let step = tempfile::tempdir().expect("tmp");
    call(
        step.path(),
        "toolu_02",
        &json!({"name": "Read", "input": {}}),
    );
    call(
        step.path(),
        "toolu_01",
        &json!({"name": "Read", "input": {}}),
    );
    std::fs::create_dir_all(step.path().join(super::TOOLS_SUBDIR).join("toolu_03"))
        .expect("bare dir");
    let ids: Vec<String> = calls(step.path()).into_keys().collect();
    assert_eq!(ids, vec!["toolu_01".to_owned(), "toolu_02".to_owned()]);
}

/// No `tools/` at all is a step that has called nothing — the general path with
/// no input, never an error with an opinion.
#[test]
fn a_step_with_no_tool_subtree_lists_nothing() {
    let step = tempfile::tempdir().expect("tmp");
    assert!(calls(step.path()).is_empty());
}

/// Every partial-write shape a record can be caught in reads as absence, and a
/// capture whose status is unreadable still closes the window — the file's
/// presence is what closing means.
#[test]
fn an_unreadable_record_degrades_rather_than_lying() {
    let step = tempfile::tempdir().expect("tmp");
    let dir = step.path().join(super::TOOLS_SUBDIR).join("toolu_01");
    std::fs::create_dir_all(&dir).expect("dir");
    std::fs::write(dir.join("input.json"), b"{ half").expect("torn input");
    let open = posted("toolu_01", &dir);
    assert_eq!((open.name, open.input), (None, None), "nothing invented");

    std::fs::write(dir.join("output.json"), b"{}").expect("statusless output");
    assert_eq!(
        complete("toolu_01", &dir).exit_code,
        Some(0),
        "a landed capture is closed, whatever it managed to say"
    );
}

/// A long input is bounded, and the cut keeps both ends — the head says which
/// command and the tail says which invocation of it.
#[test]
fn a_long_input_is_bounded_at_both_ends() {
    let step = tempfile::tempdir().expect("tmp");
    let long = "x".repeat(4000);
    let dir = call(
        step.path(),
        "toolu_01",
        &json!({"name": "Write", "input": {"head": "apk add", "body": long, "tail": "done"}}),
    );
    let input = posted("toolu_01", &dir).input.expect("an input");
    assert!(input.chars().count() <= super::INPUT_MAX, "{}", input.len());
    assert!(input.contains("apk add"), "the head survives: {input}");
    assert!(input.contains("done"), "and so does the tail: {input}");
}

/// The spelling round-trips both shapes of the event, and absence stays absence
/// — an in-flight call has no `exit_code` key at all, which is the fact.
#[test]
fn both_events_round_trip_and_absence_stays_absence() {
    let open = ToolEvent {
        tool_use: "toolu_01".into(),
        name: Some("box2_Bash".into()),
        input: Some("{\"command\":\"ps aux\"}".into()),
        exit_code: None,
    };
    let value = event_value(&open);
    assert!(value.get("exit_code").is_none(), "in flight says nothing");
    assert_eq!(event_of(&value).expect("decodes"), open);

    let closed = ToolEvent {
        tool_use: "toolu_01".into(),
        exit_code: Some(0),
        ..ToolEvent::default()
    };
    let value = event_value(&closed);
    assert_eq!(value.get("exit_code"), Some(&json!(0)));
    assert_eq!(event_of(&value).expect("decodes"), closed);
}

/// The decoder is strict where a reading would otherwise be invented: the
/// identity a follower keys on, the frame's own shape, and a status that is not
/// one.
#[test]
fn the_decoder_refuses_what_it_cannot_read() {
    assert!(event_of(&json!("a string")).is_err(), "not an object");
    assert!(event_of(&json!({})).is_err(), "no tool_use to key on");
    assert!(
        event_of(&json!({"tool_use": "t", "exit_code": "0"})).is_err(),
        "a status that is not a number"
    );
    assert!(
        event_of(&json!({"tool_use": "t", "exit_code": 9_999_999_999_i64})).is_err(),
        "nor one no exit code can be"
    );
    assert_eq!(
        event_of(&json!({"tool_use": "t", "exit_code": null}))
            .expect("null reads as absent")
            .exit_code,
        None
    );
}
