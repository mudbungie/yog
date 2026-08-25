//! The settled tail's **semantic** half (bl-fb87): what ended the turn, read
//! off the canonical `finish.reason` alone — the [`Ending`] that says what the
//! [`Framing`] cannot, because a turn the request's `max_tokens` cut off keeps
//! every transport promise and is still not whole.
//!
//! [`super`] owns the transport half and the fixtures both suites read.

use super::super::*;
use super::{ERROR_END, FINISH_END};

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
        crate::git_tree::fold_stream(jsonl).text.as_deref(),
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
