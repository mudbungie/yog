//! The Usage-event parse and the §5.1 #16 fold, at the one home of both.

use super::*;

/// The three provider shapes the fold has to answer for, at the one place the
/// formula lives — the counters are NOT four disjoint slices (bl-6621, litany
/// bl-68f5).
///
/// *Contained* (OpenAI-shaped, Google): the cached slice sits inside the prompt
/// counter, so the prompt is `input` and the cached tokens are counted once —
/// this is the measured line the defect was found on, `input 93_556 /
/// cache_read 91_648`, which the old four-counter sum billed as 185,336.
/// *Disjoint* (Anthropic): `input` is only the uncached tail, so the prompt is
/// the cached slices and the figure is a floor, never an over-statement.
/// *No cache counters* (ollama): plain `input + output`.
#[test]
fn the_cached_slice_is_billed_once_whatever_shape_the_provider_reports() {
    // The measured line, verbatim off an OpenAI-shaped row.
    let contained = spend_from_bytes(
        br#"{"type":"usage","input_tokens":93556,"output_tokens":132,"cache_read_tokens":91648,"cache_write_tokens":null}"#,
    );
    assert_eq!(contained.prompt_tokens(), 93_556);
    assert_eq!(contained.uncached_prompt_tokens(), 1_908);
    assert_eq!(contained.total_tokens(), 93_688);

    let disjoint = BudgetSpend {
        input_tokens: 4_000,
        output_tokens: 500,
        cache_read_tokens: 90_000,
        cache_write_tokens: 6_000,
    };
    assert_eq!(disjoint.prompt_tokens(), 96_000);
    assert_eq!(disjoint.uncached_prompt_tokens(), 0);
    assert_eq!(disjoint.total_tokens(), 96_500);

    let uncached = BudgetSpend {
        input_tokens: 30_000,
        output_tokens: 900,
        ..BudgetSpend::default()
    };
    assert_eq!(uncached.prompt_tokens(), 30_000);
    assert_eq!(uncached.uncached_prompt_tokens(), 30_000);
    assert_eq!(uncached.total_tokens(), 30_900);

    assert_eq!(BudgetSpend::default().total_tokens(), 0);
}

/// The invariant that keeps the token figure and the dollar figure from ever
/// telling different stories: the three priced prompt slices plus the output
/// are exactly the tokens `total_tokens` counts, on every shape above.
#[test]
fn the_priced_partition_sums_to_the_counted_total() {
    for s in [
        BudgetSpend {
            input_tokens: 93_556,
            output_tokens: 132,
            cache_read_tokens: 91_648,
            cache_write_tokens: 0,
        },
        BudgetSpend {
            input_tokens: 0,
            output_tokens: 0,
            cache_read_tokens: 5,
            cache_write_tokens: 0,
        },
        BudgetSpend {
            input_tokens: 4_000,
            output_tokens: 500,
            cache_read_tokens: 90_000,
            cache_write_tokens: 6_000,
        },
        BudgetSpend {
            input_tokens: 30_000,
            output_tokens: 900,
            ..BudgetSpend::default()
        },
    ] {
        assert_eq!(
            s.uncached_prompt_tokens() + s.cached_tokens() + s.output_tokens,
            s.total_tokens(),
            "{s:?}"
        );
    }
}

/// The Usage-event vocabulary, at the one place it is parsed: every
/// segment counts (a billed retry, ARCH §6), and everything that is not a
/// well-shaped `usage` event counts zero rather than refusing the file.
/// The tree walk that feeds these bytes is `bills/tests.rs`.
#[test]
fn folds_every_segment_and_ignores_everything_else() {
    let jsonl = [
        r#"{"type":"usage","input_tokens":10,"output_tokens":5,"cache_read_tokens":2,"cache_write_tokens":1}"#,
        r#"{"type":"content_delta","index":0,"delta":{"text_delta":"hi"}}"#,
        r#"{"type":"usage","input_tokens":3,"output_tokens":null}"#,
        "not json",
        r#"{"no_type":1}"#,
        r#"{"type":123}"#,
    ]
    .join("\n");

    let s = spend_from_bytes(jsonl.as_bytes());
    // The second segment's `null` output and absent cache counters each
    // count zero — unknown is never fabricated.
    assert_eq!(s.input_tokens, 13);
    assert_eq!(s.output_tokens, 5);
    assert_eq!(s.cache_read_tokens, 2);
    assert_eq!(s.cache_write_tokens, 1);
    // 18, not 21: the fold is `max(input, cache_read + cache_write) + output`,
    // so the three-token cached slice is inside the 13 rather than beside it.
    assert_eq!(s.total_tokens(), 18);
}

#[test]
fn empty_bytes_are_zero_spend() {
    assert_eq!(spend_from_bytes(b""), BudgetSpend::default());
}

/// Fullness reads the LAST segment, spend reads them all — over the very
/// same bytes, so a retried step can never be read as a context that grew
/// by the retry (§5.1 #35). Per field, last wins: an Anthropic stream's
/// `message_delta` usage carries only the output, and reading that line
/// alone was a prompt of zero.
#[test]
fn last_usage_takes_the_final_segment_per_field_not_the_fold() {
    let jsonl = [
        r#"{"type":"usage","input_tokens":10,"cache_read_tokens":90}"#,
        r#"{"type":"content_delta","index":0,"delta":{"text_delta":"hi"}}"#,
        r#"{"type":"usage","input_tokens":4,"cache_read_tokens":120,"output_tokens":null}"#,
        r#"{"type":"usage","input_tokens":null,"output_tokens":15}"#,
    ]
    .join("\n");
    let last = last_usage(jsonl.as_bytes());
    assert_eq!(last.input_tokens, 4);
    assert_eq!(last.cache_read_tokens, 120);
    assert_eq!(last.output_tokens, 15);
    assert_eq!(spend_from_bytes(jsonl.as_bytes()).input_tokens, 14);
}

#[test]
fn a_payload_with_no_usage_line_has_no_last_segment() {
    assert_eq!(last_usage(b"{\"type\":\"end\"}"), BudgetSpend::default());
    assert_eq!(context_window(b"{\"type\":\"end\"}"), None);
}

/// brazen's `input_total_tokens` is the whole prompt, sealed by the decoder
/// that knows whether the cached slice sits inside or beside the provider's
/// own counter — so where a line carries it, it IS the prompt, on the disjoint
/// shape the max rule could only floor. A line without it reads as before.
#[test]
fn the_sealed_prompt_total_is_read_where_a_line_carries_it() {
    let sealed = spend_from_bytes(
        br#"{"type":"usage","input_tokens":4000,"output_tokens":500,"cache_read_tokens":90000,"cache_write_tokens":6000,"input_total_tokens":100000}"#,
    );
    assert_eq!(sealed.prompt_tokens(), 100_000);
    assert_eq!(sealed.uncached_prompt_tokens(), 4_000);
    assert_eq!(sealed.total_tokens(), 100_500);
    assert_eq!(
        sealed.uncached_prompt_tokens() + sealed.cached_tokens() + sealed.output_tokens,
        sealed.total_tokens()
    );
}

/// The shape is decided **per line, not per record** (bl-bf8b): a
/// `response.json` written across a `bz` upgrade — an attempt segment from
/// before 0.0.10 folded with one after it — reads each line by what that line
/// carries, because [`prompt`] asks the question once per event and
/// [`BudgetSpend::add`] folds the answers. There is no version to detect and no
/// record-wide branch: the sealed line contributes its served total, the older
/// one its max reading, and the ceiling sees their sum.
#[test]
fn each_line_is_read_by_its_own_shape_when_a_record_carries_both() {
    let mixed = [
        // Pre-0.0.10: disjoint Anthropic counters, floored by the max rule to
        // 96_000 — the cached slices, since `input` is only the uncached tail.
        r#"{"type":"usage","input_tokens":4000,"output_tokens":500,"cache_read_tokens":90000,"cache_write_tokens":6000}"#,
        // 0.0.10 and after: the served total, which the max cannot move.
        r#"{"type":"usage","input_tokens":4000,"output_tokens":500,"cache_read_tokens":90000,"cache_write_tokens":6000,"input_total_tokens":100000}"#,
    ]
    .join("\n");
    let folded = spend_from_bytes(mixed.as_bytes());
    assert_eq!(folded.input_tokens, 4_000 + 100_000);
    assert_eq!(folded.cached_tokens(), 192_000);
    assert_eq!(folded.prompt_tokens(), 192_000, "the floor is the fold's");
    assert_eq!(folded.total_tokens(), 193_000);
    // Read alone, each line is exactly what its own shape says.
    let old_only = spend_from_bytes(mixed.split('\n').next().unwrap().as_bytes());
    assert_eq!(old_only.prompt_tokens(), 96_000);
    let new_only = spend_from_bytes(mixed.split('\n').nth(1).unwrap().as_bytes());
    assert_eq!(new_only.prompt_tokens(), 100_000);
}

/// The denominator rides the same lines as the numerator (§5.1 #35): the
/// window brazen stamps on the usage event, last stated wins, a zero or an
/// absent one is no window at all.
#[test]
fn the_window_is_the_last_one_the_usage_lines_state() {
    let stated = [
        r#"{"type":"usage","input_tokens":10,"context_window":200000}"#,
        r#"{"type":"usage","output_tokens":5}"#,
    ]
    .join("\n");
    assert_eq!(context_window(stated.as_bytes()), Some(200_000));
    let moved = [
        r#"{"type":"usage","input_tokens":10,"context_window":200000}"#,
        r#"{"type":"usage","input_tokens":10,"context_window":1000000}"#,
    ]
    .join("\n");
    assert_eq!(context_window(moved.as_bytes()), Some(1_000_000));
    assert_eq!(
        context_window(br#"{"type":"usage","input_tokens":10,"context_window":0}"#),
        None
    );
    assert_eq!(
        context_window(br#"{"type":"usage","input_tokens":10}"#),
        None
    );
}
