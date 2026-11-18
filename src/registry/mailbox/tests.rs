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

/// **The streamed form's spellings** (REMOTE §5.7) round-trip and refuse as
/// strictly: a piece states both streams, offsets state both counts, and a
/// post states a piece or a capture — the capture winning where both ride, so
/// today's foot is read exactly as it was.
#[test]
fn the_streamed_spellings_round_trip_and_refuse_by_name() {
    use super::stream::{offsets_of, offsets_value, piece_of, piece_value, post_of};
    let piece = Piece {
        stdout: "a".to_owned(),
        stderr: "b".to_owned(),
    };
    assert_eq!(piece_of(&piece_value(&piece)), Ok(piece.clone()));
    assert!(piece_of(&json!("no")).is_err());
    assert!(piece_of(&json!({"stdout": ""})).is_err_and(|e| e.contains("stderr")));
    let since = Offsets {
        stdout: 7,
        stderr: 0,
    };
    assert_eq!(offsets_of(&offsets_value(since)), Ok(since));
    assert!(offsets_of(&json!(7)).is_err());
    assert!(offsets_of(&json!({"stdout": -1, "stderr": 0})).is_err());

    let read = |v: Value| post_of(v.as_object().expect("an envelope"));
    assert_eq!(
        read(json!({"partial": piece_value(&piece), "capture": capture_value(&ran(2))})),
        Ok(Post::Capture(ran(2)))
    );
    assert_eq!(
        read(json!({"partial": piece_value(&piece)})),
        Ok(Post::Partial(piece))
    );
    assert_eq!(read(json!({})), Err("complete: missing capture".to_owned()));
}
