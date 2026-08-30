//! The line codec's own tables: every truncation branch of the ≤`CAP`
//! serializer, the UTF-8 head snap, and the caller-side goal clip. Pure — no
//! filesystem, no clock. The file-level tests (append/tail round-trip, the
//! forgiving parse over real bytes) are `super::super`'s.

use super::super::tests::sample;
use super::*;
use serde_json::Value;

/// Parse a built line back to a JSON object for field assertions.
fn parsed(line: &[u8]) -> serde_json::Map<String, Value> {
    assert_eq!(*line.last().unwrap(), b'\n', "line must end in newline");
    serde_json::from_slice::<Value>(&line[..line.len() - 1])
        .unwrap()
        .as_object()
        .unwrap()
        .clone()
}

#[test]
fn build_line_serializes_all_fields_untruncated() {
    let line = build_line(&sample());
    assert!(line.len() <= CAP);
    let obj = parsed(&line);
    assert_eq!(obj["ts"], Value::from("2026-07-17T12:00:00Z"));
    assert_eq!(obj["argv"], Value::from(vec!["bl", "close", "bl-4db6"]));
    assert_eq!(obj["cwd"], Value::from("/home/u/dev/brazen"));
    assert_eq!(obj["exit"], Value::from(0));
    assert_eq!(obj["stdout"], Value::from("ok"));
    assert_eq!(obj["stderr"], Value::from(""));
    assert!(!obj.contains_key("truncated"));
}

#[test]
fn build_line_boundary_exactly_cap_then_one_over() {
    let base = build_line(&OpEntry {
        stdout: String::new(),
        ..sample()
    })
    .len();

    // Exactly CAP: 'a' is unescaped, so each byte of stdout adds one byte.
    let at_cap = build_line(&OpEntry {
        stdout: "a".repeat(CAP - base),
        ..sample()
    });
    assert_eq!(at_cap.len(), CAP);
    assert!(!parsed(&at_cap).contains_key("truncated"));

    // One over: must truncate stdout and mark the line.
    let over = build_line(&OpEntry {
        stdout: "a".repeat(CAP - base + 1),
        ..sample()
    });
    assert!(over.len() <= CAP);
    let obj = parsed(&over);
    assert_eq!(obj["truncated"], Value::Bool(true));
    let kept = obj["stdout"].as_str().unwrap();
    assert!(kept.len() < CAP - base + 1);
    assert!(kept.bytes().all(|b| b == b'a'));
}

#[test]
fn build_line_truncates_huge_stdout_keeping_stderr() {
    let line = build_line(&OpEntry {
        stdout: "a".repeat(100_000),
        stderr: "keep-me".into(),
        ..sample()
    });
    assert!(line.len() <= CAP);
    let obj = parsed(&line);
    assert_eq!(obj["truncated"], Value::Bool(true));
    assert_eq!(obj["stderr"], Value::from("keep-me"));
    let kept = obj["stdout"].as_str().unwrap();
    assert!(!kept.is_empty() && kept.bytes().all(|b| b == b'a'));
}

#[test]
fn build_line_sacrifices_stdout_before_stderr() {
    // stderr so large that even an empty stdout overflows: stdout must go to
    // zero first, then stderr shrinks (§4.2 order).
    let line = build_line(&OpEntry {
        stdout: "dropped-entirely".into(),
        stderr: "b".repeat(100_000),
        ..sample()
    });
    assert!(line.len() <= CAP);
    let obj = parsed(&line);
    assert_eq!(obj["truncated"], Value::Bool(true));
    assert_eq!(obj["stdout"], Value::from(""));
    let kept = obj["stderr"].as_str().unwrap();
    assert!(!kept.is_empty() && kept.bytes().all(|b| b == b'b'));
}

#[test]
fn build_line_fixed_fields_exceeding_cap_zero_the_outputs() {
    // A pathological argv alone blows the cap: outputs go to zero and the
    // line is allowed to exceed — structurally unavoidable.
    let line = build_line(&OpEntry {
        argv: vec!["x".repeat(5_000)],
        stdout: "gone".into(),
        stderr: "also-gone".into(),
        ..sample()
    });
    assert!(line.len() > CAP);
    let obj = parsed(&line);
    assert_eq!(obj["truncated"], Value::Bool(true));
    assert_eq!(obj["stdout"], Value::from(""));
    assert_eq!(obj["stderr"], Value::from(""));
    assert_eq!(obj["argv"].as_array().unwrap().len(), 1);
}

#[test]
fn build_line_snaps_head_to_utf8_boundary() {
    // 'é' is two bytes; truncation lands the byte budget mid-char, so the head
    // must snap back to a boundary and stay valid UTF-8.
    let line = build_line(&OpEntry {
        stdout: "é".repeat(50_000),
        stderr: "z".into(),
        ..sample()
    });
    assert!(line.len() <= CAP);
    let obj = parsed(&line);
    let kept = obj["stdout"].as_str().unwrap();
    assert!(!kept.is_empty());
    assert!(kept.chars().all(|c| c == 'é'));
}

#[test]
fn clip_arg_returns_short_verbatim_and_marks_a_long_one() {
    // Verbatim branch: within budget → unchanged, no marker.
    assert_eq!(clip_arg("goal", 16), "goal");
    // Truncate branch: over budget → head + an explicit elision marker.
    let clipped = clip_arg(&"x".repeat(100), 10);
    assert!(clipped.starts_with(&"x".repeat(10)));
    assert!(
        clipped.contains("[+90 bytes elided]"),
        "the marker names the dropped byte count"
    );
}

#[test]
fn clip_goal_leaves_a_fitting_entry_untouched() {
    // A small entry already serializes ≤ CAP → returned verbatim, no clip.
    let e = sample();
    assert_eq!(clip_goal(&e), e);
}

#[test]
fn clip_goal_on_empty_argv_is_a_no_op() {
    // Nothing to clip: split_last is None, the entry rides back unchanged.
    let e = OpEntry {
        argv: vec![],
        ..sample()
    };
    assert_eq!(clip_goal(&e), e);
}

#[test]
fn clip_goal_holds_cap_after_json_escape() {
    // Each byte JSON-escapes to `\u00XX` (6 bytes): a 9000-byte goal would
    // serialize to ~54 KB unclipped. clip_goal trims the tail argv element against
    // the *serialized* length, so the line lands ≤ CAP — not a raw-byte proxy.
    let e = OpEntry {
        argv: vec![
            "litany".into(),
            "prompt".into(),
            "/ws".into(),
            "\u{1}".repeat(9000),
        ],
        ..sample()
    };
    assert!(build_line(&e).len() > CAP, "the raw goal overflows the cap");
    let clipped = clip_goal(&e);
    assert!(
        build_line(&clipped).len() <= CAP,
        "serialized line ≤ CAP post-escape"
    );
    assert!(
        clipped.argv.last().unwrap().contains("bytes elided"),
        "the clipped goal carries the elision marker"
    );
    // Only the tail element clips; the fixed argv prefix is preserved.
    assert_eq!(&clipped.argv[..3], &e.argv[..3]);
}

/// bl-48f8: `origin` is a fixed field — it survives the round trip, it survives
/// truncation (only `stdout`/`stderr` are sacrificed), and a line an older yog
/// wrote without it reads as the composer's, so the failure still banners once
/// rather than nowhere (INV-2).
#[test]
fn origin_round_trips_and_a_line_without_one_reads_as_the_composer() {
    let entry = OpEntry {
        origin: Origin::Balls,
        ..sample()
    };
    let line = String::from_utf8(build_line(&entry)).unwrap();
    assert_eq!(parse_line(&line).unwrap().origin, Origin::Balls);

    let truncating = OpEntry {
        origin: Origin::World,
        stdout: "a".repeat(100_000),
        ..sample()
    };
    let line = String::from_utf8(build_line(&truncating)).unwrap();
    let back = parse_line(&line).unwrap();
    assert_eq!(back.origin, Origin::World, "a fixed field never truncates");

    let legacy = r#"{"ts":"T","argv":["bl","close"],"cwd":"/p","exit":1,"stderr":"boom"}"#;
    assert_eq!(parse_line(legacy).unwrap().origin, Origin::Conversation);
}
