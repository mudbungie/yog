//! Unit tests for the settled-tail classifier (terminal).

use super::*;

const FINISH_END: &[u8] = br#"{"type":"message_start","v":1,"role":"assistant"}
{"type":"finish","reason":"stop"}
{"type":"end"}
"#;
const ERROR_END: &[u8] = br#"{"type":"message_start","v":1,"role":"assistant"}
{"type":"error","kind":"transport","message":"reset"}
{"type":"end"}
"#;

#[test]
fn finish_then_end_is_complete() {
    assert!(last_segment_complete(FINISH_END));
}

#[test]
fn error_then_end_is_not_complete() {
    // A failed attempt (error+end) is *failed*, not complete (§4.4).
    assert!(!last_segment_complete(ERROR_END));
}

#[test]
fn no_trailing_end_is_not_complete() {
    let jsonl = br#"{"type":"content_delta","index":0,"delta":{"text_delta":"hi"}}
"#;
    assert!(!last_segment_complete(jsonl));
}

#[test]
fn empty_or_newline_only_is_not_complete() {
    assert!(!last_segment_complete(b""));
    assert!(!last_segment_complete(b"\n\n"));
}

#[test]
fn trailing_partial_line_after_end_is_ignored() {
    let jsonl = b"{\"type\":\"finish\",\"reason\":\"stop\"}\n{\"type\":\"end\"}\n{partial";
    assert!(last_segment_complete(jsonl));
}

#[test]
fn only_latest_segment_decides_complete() {
    // A prior failed attempt then a clean retry: complete.
    let jsonl = br#"{"type":"error","kind":"x"}
{"type":"end"}
{"type":"message_start","v":1}
{"type":"finish","reason":"stop"}
{"type":"end"}
"#;
    assert!(last_segment_complete(jsonl));
}

#[test]
fn latest_segment_error_after_earlier_finish_is_not_complete() {
    // A clean attempt then a failed retry: failed (last segment wins).
    let jsonl = br#"{"type":"finish","reason":"stop"}
{"type":"end"}
{"type":"error","kind":"x"}
{"type":"end"}
"#;
    assert!(!last_segment_complete(jsonl));
}

#[test]
fn end_without_finish_or_error_is_not_complete() {
    // Defensive: an `end` with neither finish nor error in its segment
    // is not a clean completion.
    assert!(!last_segment_complete(
        b"{\"type\":\"message_start\"}\n{\"type\":\"end\"}\n"
    ));
}

#[test]
fn malformed_last_line_is_not_complete() {
    assert!(!last_segment_complete(b"{\"type\":\"finish\"}\nnot json\n"));
}

#[test]
fn framing_classifies_the_three_outcomes() {
    // Complete, Failed (error segment), Killed (no trailing end) — the
    // three states the steps inspector renders per step (§15 Y13).
    assert_eq!(framing(FINISH_END), Framing::Complete);
    assert_eq!(framing(ERROR_END), Framing::Failed);
    assert_eq!(
        framing(b"{\"type\":\"content_delta\",\"index\":0}\n"),
        Framing::Killed
    );
    // An `end` with neither finish nor error is not a clean completion.
    assert_eq!(
        framing(b"{\"type\":\"message_start\"}\n{\"type\":\"end\"}\n"),
        Framing::Killed
    );
}

#[test]
fn error_text_returns_the_error_line_iff_framing_is_failed() {
    // Failed: the verbatim error event line comes back (its status/message the
    // auth heuristic reads); complete and killed yield None — error_text Some
    // ⟺ framing Failed.
    let failed = br#"{"type":"error","kind":"http","status":401}
{"type":"end"}
"#;
    assert_eq!(
        error_text(failed).as_deref(),
        Some(r#"{"type":"error","kind":"http","status":401}"#)
    );
    assert_eq!(framing(failed), Framing::Failed);
    assert_eq!(error_text(FINISH_END), None); // complete
    assert_eq!(error_text(b"{\"type\":\"content_delta\"}\n"), None); // killed
    assert_eq!(error_text(b""), None); // empty
    assert_eq!(error_text(b"\n\n"), None); // newline-only: no settled segment
    assert_eq!(error_text(b"no trailing newline"), None); // unterminated
}

#[test]
fn error_text_reads_only_the_latest_segment() {
    // A failed attempt then a clean retry: complete, so no error text (the
    // prior segment's error is behind an `end` boundary the walk stops at).
    let retried = br#"{"type":"error","kind":"x"}
{"type":"end"}
{"type":"finish","reason":"stop"}
{"type":"end"}
"#;
    assert_eq!(error_text(retried), None);
    assert_eq!(framing(retried), Framing::Complete);
}

#[test]
fn segment_count_counts_end_events() {
    // Two completed attempts (an errored retry then a clean one) → 2.
    let jsonl = br#"{"type":"error","kind":"x"}
{"type":"end"}
{"type":"finish","reason":"stop"}
{"type":"end"}
"#;
    assert_eq!(segment_count(jsonl), 2);
    // A single in-flight segment with no trailing `end` → 0 completed.
    assert_eq!(
        segment_count(b"{\"type\":\"content_delta\",\"index\":0}\n"),
        0
    );
    assert_eq!(segment_count(b""), 0);
}
