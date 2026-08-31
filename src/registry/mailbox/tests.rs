//! The routing leg's vocabulary: the three JSON spellings, and how each
//! refuses.

use super::*;

fn ran(exit_code: i32) -> Capture {
    Capture {
        stdout: "out".to_owned(),
        stderr: String::new(),
        exit_code,
    }
}

/// The three JSON spellings round-trip, and each refuses strictly — a capture
/// is what a model reads, so a guessed field would be a guessed answer.
#[test]
fn the_wire_spellings_round_trip_and_refuse_by_name() {
    let capture = ran(2);
    assert_eq!(capture_of(&capture_value(&capture)), Ok(capture));
    let invocation = Invocation {
        id: "inv-1".to_owned(),
        tool: "Bash".to_owned(),
        input: json!({"command": "ls"}),
        cwd: None,
    };
    assert_eq!(
        invocation_of(&invocation_value(&invocation)),
        Ok(invocation)
    );
    assert!(capture_of(&json!("no")).is_err());
    assert!(capture_of(&json!({"stdout": "", "stderr": ""})).is_err());
    assert!(
        capture_of(&json!({"stdout": "", "stderr": "", "exit_code": i64::MAX}))
            .is_err_and(|e| e.contains("out of range"))
    );
    assert!(invocation_of(&json!([])).is_err());
    assert!(
        invocation_of(&json!({"invocation": "i", "tool": "t"})).is_err_and(|e| e.contains("input"))
    );
}

/// **The subject's location reads strictly** (bl-77be): a carried cwd rounds,
/// absence stays absent, and a mistyped one refuses — where a tool will run
/// is an instruction, not an observation.
#[test]
fn a_carried_cwd_round_trips_and_a_mistyped_one_refuses() {
    let placed = Invocation {
        id: "inv-2".to_owned(),
        tool: "bash".to_owned(),
        input: json!({"command": "true"}),
        cwd: Some("/w/home/agents/c-1".to_owned()),
    };
    let spelled = invocation_value(&placed);
    assert_eq!(spelled.get("cwd"), Some(&json!("/w/home/agents/c-1")));
    assert_eq!(invocation_of(&spelled), Ok(placed));
    assert!(
        invocation_of(&json!({"invocation": "i", "tool": "t", "input": {}, "cwd": 7}))
            .is_err_and(|e| e.contains("\"cwd\" is not a string"))
    );
    assert_eq!(
        invocation_of(&json!({"invocation": "i", "tool": "t", "input": {}, "cwd": null}))
            .expect("null is the ordinary no-location case")
            .cwd,
        None
    );
}
