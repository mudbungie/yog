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
