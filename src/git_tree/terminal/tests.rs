//! Unit tests for the settled-tail classifier (terminal).

use super::*;

/// The live classifier's boolean view: complete transport **and** a turn that
/// ended on its own terms (§4.4, [`Settled::whole`]).
fn whole(bytes: &[u8]) -> bool {
    settled(bytes).whole()
}

/// The transport half alone, for the tests that name a [`Framing`].
fn framing_of(bytes: &[u8]) -> Framing {
    settled(bytes).framing
}

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
    assert!(whole(FINISH_END));
}

#[test]
fn error_then_end_is_not_complete() {
    // A failed attempt (error+end) is *failed*, not complete (§4.4).
    assert!(!whole(ERROR_END));
}

#[test]
fn no_trailing_end_is_not_complete() {
    let jsonl = br#"{"type":"content_delta","index":0,"delta":{"text_delta":"hi"}}
"#;
    assert!(!whole(jsonl));
}

#[test]
fn empty_or_newline_only_is_not_complete() {
    assert!(!whole(b""));
    assert!(!whole(b"\n\n"));
}

#[test]
fn trailing_partial_line_after_end_is_ignored() {
    let jsonl = b"{\"type\":\"finish\",\"reason\":\"stop\"}\n{\"type\":\"end\"}\n{partial";
    assert!(whole(jsonl));
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
    assert!(whole(jsonl));
}

#[test]
fn latest_segment_error_after_earlier_finish_is_not_complete() {
    // A clean attempt then a failed retry: failed (last segment wins).
    let jsonl = br#"{"type":"finish","reason":"stop"}
{"type":"end"}
{"type":"error","kind":"x"}
{"type":"end"}
"#;
    assert!(!whole(jsonl));
}

#[test]
fn end_without_finish_or_error_is_not_complete() {
    // Defensive: an `end` with neither finish nor error in its segment
    // is not a clean completion.
    assert!(!whole(
        b"{\"type\":\"message_start\"}\n{\"type\":\"end\"}\n"
    ));
}

#[test]
fn malformed_last_line_is_not_complete() {
    assert!(!whole(b"{\"type\":\"finish\"}\nnot json\n"));
}

#[test]
fn framing_classifies_the_three_outcomes() {
    // Complete, Failed (error segment), Killed (no trailing end) — the
    // three states the steps inspector renders per step (§15 Y13).
    assert_eq!(framing_of(FINISH_END), Framing::Complete);
    assert_eq!(framing_of(ERROR_END), Framing::Failed);
    assert_eq!(
        framing_of(b"{\"type\":\"content_delta\",\"index\":0}\n"),
        Framing::Killed
    );
    // An `end` with neither finish nor error is not a clean completion.
    assert_eq!(
        framing_of(b"{\"type\":\"message_start\"}\n{\"type\":\"end\"}\n"),
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
    assert_eq!(framing_of(failed), Framing::Failed);
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
    assert_eq!(framing_of(retried), Framing::Complete);
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

/// The semantic half (bl-fb87): what ended the turn, read off the canonical
/// `finish.reason` alone. Every reason but `length` leaves the pre-existing
/// classification exactly where it was.
#[test]
fn the_ending_reads_the_canonical_finish_reason_and_nothing_else() {
    let of = |reason: &str| {
        let jsonl =
            format!("{{\"type\":\"finish\",\"reason\":\"{reason}\"}}\n{{\"type\":\"end\"}}\n");
        settled(jsonl.as_bytes())
    };
    // The one reason that changes what yog may say about the turn.
    assert_eq!(of("length").ending, Ending::OutputLimit);
    // Every other canonical word — and a provider's own, passed through as
    // `FinishReason::Other` — ends on the model's own terms.
    for reason in [
        "stop",
        "tool_use",
        "stop_sequence",
        "pause",
        "refusal",
        "eos",
    ] {
        assert_eq!(of(reason).ending, Ending::OwnTerms, "{reason}");
    }
    // A `finish` with no readable reason is not a claim that the turn was cut
    // off: nothing said `length`, so nothing here says truncated.
    assert_eq!(
        settled(b"{\"type\":\"finish\"}\n{\"type\":\"end\"}\n").ending,
        Ending::OwnTerms
    );
    assert_eq!(
        settled(b"{\"type\":\"finish\",\"reason\":7}\n{\"type\":\"end\"}\n").ending,
        Ending::OwnTerms
    );
}

/// The framing stays honest while the ending says what framing cannot: the
/// bl-fb87 shape frames `Complete` — the transport kept every promise, and the
/// entry lernie sealed off it really is there — and is not *whole*.
#[test]
fn an_output_limited_tail_is_complete_transport_and_an_unfinished_turn() {
    let jsonl = br#"{"type":"message_start","v":1,"role":"assistant"}
{"type":"content_delta","index":0,"delta":{"thinking_delta":"hmm"}}
{"type":"finish","reason":"length"}
{"type":"end"}
"#;
    let read = settled(jsonl);
    assert_eq!(read.framing, Framing::Complete);
    assert_eq!(read.ending, Ending::OutputLimit);
    assert!(!read.whole(), "complete on the wire, unfinished as a turn");
    // A clean stop is both.
    assert!(settled(FINISH_END).whole());
}

/// Partial text is text: the turn that ran out of room still said something,
/// and the ending marks it truncated rather than deleting it.
#[test]
fn a_partial_text_length_turn_is_truncated_without_losing_what_it_said() {
    let jsonl = br#"{"type":"content_delta","index":0,"delta":{"text_delta":"as I was say"}}
{"type":"finish","reason":"length"}
{"type":"end"}
"#;
    assert_eq!(settled(jsonl).ending, Ending::OutputLimit);
    assert_eq!(
        super::super::fold_stream(jsonl).text.as_deref(),
        Some("as I was say"),
        "the §5.1 #10 fold still hands the fragment to the transcript"
    );
}

/// A tail with no semantic result to read says so, rather than borrowing one:
/// failed and killed framings carry `Unread`, never `OwnTerms`.
#[test]
fn a_tail_with_no_finish_reads_no_ending_at_all() {
    for bytes in [
        ERROR_END,                                             // error + end
        b"{\"type\":\"content_delta\",\"index\":0}\n",         // no trailing end
        b"{\"type\":\"message_start\"}\n{\"type\":\"end\"}\n", // end, no finish
        b"",
    ] {
        assert_eq!(settled(bytes).ending, Ending::Unread);
        assert!(!settled(bytes).whole());
    }
}

/// Only the last segment decides the ending, exactly as it decides the
/// framing: a retry that ran out of room supersedes a clean earlier attempt,
/// and a clean retry supersedes an earlier truncation.
#[test]
fn only_the_latest_segment_decides_the_ending() {
    let truncated_retry = br#"{"type":"finish","reason":"stop"}
{"type":"end"}
{"type":"finish","reason":"length"}
{"type":"end"}
"#;
    assert_eq!(settled(truncated_retry).ending, Ending::OutputLimit);
    let clean_retry = br#"{"type":"finish","reason":"length"}
{"type":"end"}
{"type":"finish","reason":"stop"}
{"type":"end"}
"#;
    assert_eq!(settled(clean_retry).ending, Ending::OwnTerms);
    // Two `finish` events inside one segment: the last one is the outcome.
    let doubled = br#"{"type":"finish","reason":"length"}
{"type":"finish","reason":"stop"}
{"type":"end"}
"#;
    assert_eq!(settled(doubled).ending, Ending::OwnTerms);
}

/// An `error` beside a `finish` in the same segment is still *failed*, and a
/// failed segment reads no ending — the error outranks whatever the finish
/// said, as it always has.
#[test]
fn an_error_in_the_settled_segment_outranks_its_finish() {
    let jsonl = br#"{"type":"error","kind":"x"}
{"type":"finish","reason":"length"}
{"type":"end"}
"#;
    let read = settled(jsonl);
    assert_eq!(read.framing, Framing::Failed);
    assert_eq!(read.ending, Ending::Unread);
    assert_eq!(Ending::default(), Ending::Unread);
}
