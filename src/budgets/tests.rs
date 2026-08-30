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
/// by the retry (§5.1 #35).
#[test]
fn last_usage_takes_the_final_segment_not_the_fold() {
    let jsonl = [
        r#"{"type":"usage","input_tokens":10,"cache_read_tokens":90}"#,
        r#"{"type":"content_delta","index":0,"delta":{"text_delta":"hi"}}"#,
        r#"{"type":"usage","input_tokens":4,"cache_read_tokens":120}"#,
    ]
    .join("\n");
    let last = last_usage(jsonl.as_bytes());
    assert_eq!(last.input_tokens, 4);
    assert_eq!(last.cache_read_tokens, 120);
    assert_eq!(spend_from_bytes(jsonl.as_bytes()).input_tokens, 14);
}

#[test]
fn a_payload_with_no_usage_line_has_no_last_segment() {
    assert_eq!(last_usage(b"{\"type\":\"end\"}"), BudgetSpend::default());
}
