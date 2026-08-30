use super::*;
use tempfile::tempdir;

/// A representative untruncated entry.
pub(super) fn sample() -> OpEntry {
    OpEntry {
        ts: "2026-07-17T12:00:00Z".into(),
        argv: vec!["bl".into(), "close".into(), "bl-4db6".into()],
        cwd: "/home/u/dev/brazen".into(),
        exit: 0,
        stdout: "ok".into(),
        stderr: String::new(),
        origin: Origin::default(),
    }
}
#[test]
fn append_then_tail_round_trips_in_order() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let first = sample();
    let second = OpEntry {
        argv: vec!["litany".into(), "scan".into()],
        stdout: "summary".into(),
        ..sample()
    };
    append(root, &first).unwrap();
    append(root, &second).unwrap();

    assert_eq!(tail(root, 10), vec![first, second.clone()]);
    // Cap trims the head, keeping newest-last.
    assert_eq!(tail(root, 1), vec![second]);
}

#[test]
fn append_creates_missing_state_dirs() {
    let dir = tempdir().unwrap();
    let root = dir.path().join("nested").join("state");
    append(&root, &sample()).unwrap();
    assert_eq!(tail(&root, 5), vec![sample()]);
}

#[test]
fn tail_of_missing_file_is_empty() {
    let dir = tempdir().unwrap();
    assert!(tail(&dir.path().join("absent"), 5).is_empty());
}

#[test]
fn tail_skips_corrupt_and_non_object_lines() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let path = root.join(FILENAME);
    let good = build_line(&sample());
    let mut bytes = good.clone();
    bytes.extend_from_slice(b"{not valid json\n"); // corrupt -> skipped
    bytes.extend_from_slice(b"42\n"); // valid JSON, not an object -> skipped
    bytes.extend_from_slice(b"\n"); // blank -> skipped
    bytes.extend_from_slice(&[0xff, 0xfe, b'\n']); // invalid UTF-8 -> skipped
    bytes.extend_from_slice(&good); // a second good line
    fs::write(&path, &bytes).unwrap();

    let got = tail(root, 10);
    assert_eq!(got, vec![sample(), sample()]);
}

#[test]
fn parse_line_defaults_absent_and_mistyped_fields() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let path = root.join(FILENAME);
    // Empty object -> all defaults; mixed argv drops non-strings; wrong-typed
    // exit falls back to 0.
    fs::write(&path, b"{}\n{\"argv\":[\"a\",5,\"b\"],\"exit\":\"nope\"}\n").unwrap();

    let got = tail(root, 10);
    assert_eq!(
        got,
        vec![
            OpEntry {
                ts: String::new(),
                argv: vec![],
                cwd: String::new(),
                exit: 0,
                stdout: String::new(),
                stderr: String::new(),
                origin: Origin::default(),
            },
            OpEntry {
                ts: String::new(),
                argv: vec!["a".into(), "b".into()],
                cwd: String::new(),
                exit: 0,
                stdout: String::new(),
                stderr: String::new(),
                origin: Origin::default(),
            },
        ]
    );
}

#[test]
fn synthetic_failure_encodes_the_intended_argv_and_stderr() {
    let e = OpEntry::synthetic_failure(
        "TS".into(),
        vec!["litany".into(), "prompt".into()],
        "/proj".into(),
        "No such file or directory".into(),
        Origin::Balls,
    );
    assert_eq!(e.exit, SYNTHETIC_EXIT);
    assert_eq!(e.argv, vec!["litany".to_string(), "prompt".to_string()]);
    assert_eq!(e.cwd, "/proj");
    assert!(e.stdout.is_empty());
    assert_eq!(e.stderr, "No such file or directory");
}

#[test]
fn step_failure_encodes_the_yog_step_argv() {
    let e = OpEntry::step_failure(
        "TS".into(),
        "mint",
        String::new(),
        "pool exhausted".into(),
        Origin::Balls,
    );
    assert_eq!(e.argv, vec![YOG_STEP.to_string(), "mint".to_string()]);
    assert_eq!(e.exit, SYNTHETIC_EXIT);
    assert_eq!(e.stderr, "pool exhausted");
}
