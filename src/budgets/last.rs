//! **The last reading of a step, not its fold** (DESIGN §5.1 #35): the two
//! facts the context-fullness figure takes off a `response.json`, split off
//! [`super`] on the seam its header names — the fold of every attempt segment
//! is *spend*, the final reading is *fullness*, and only the second describes
//! the context as it now stands. Same bytes, same event vocabulary
//! ([`super::usage_events`]), two questions.
//!
//! **Numerator and denominator ride one line.** brazen states the resolved
//! model row's context window on every `Usage` event it emits (its
//! model-discovery §5.5), which is the number litany's own `window_percent`
//! compaction trigger divides by; yog reads the same one off the same record
//! and keeps no table of windows (bl-9c8a — the declaration it once kept was a
//! second representation of this fact, one the engine could not see).

use super::{BudgetSpend, counter, counters, usage_events};

/// The counters that describe the context as it now stands (§5.1 #35): the
/// **last** attempt segment's, merged **per field, last wins** — brazen's own
/// consumer rule, because a stream's usage events are partial (Anthropic's
/// `message_start` carries the prompt and its `message_delta` only the
/// output; taking the final line alone read that shape as a prompt of zero).
/// Summing segments answers *spend*; the last reading answers *fullness*,
/// and a step retried three times must not read as a context three times its
/// size. Zero when the payload carries no `Usage` line at all — the general
/// path with no inputs.
pub fn last_usage(bytes: &[u8]) -> BudgetSpend {
    usage_events(bytes).fold(BudgetSpend::default(), |mut last, event| {
        let seen = counters(&event);
        for (slot, fresh, key) in [
            (&mut last.input_tokens, seen.input_tokens, "input_tokens"),
            (&mut last.output_tokens, seen.output_tokens, "output_tokens"),
            (
                &mut last.cache_read_tokens,
                seen.cache_read_tokens,
                "cache_read_tokens",
            ),
            (
                &mut last.cache_write_tokens,
                seen.cache_write_tokens,
                "cache_write_tokens",
            ),
        ] {
            if counter(&event, key).is_some() {
                *slot = fresh;
            }
        }
        last
    })
}

/// The context window the step's own usage lines carry — brazen stamps the
/// resolved model row's window on every `Usage` event it emits (its
/// model-discovery §5.5, 0.0.9), so the denominator of the §5.1 #35 figure is
/// read off the same line as its numerator, recorded per step, and yog keeps
/// no table of windows. The last one stated, by the rule above; `None` when
/// no line states one — the row served no window — or it is zero. Never a
/// default: a percentage against a number nobody published is capability
/// theater.
pub fn context_window(bytes: &[u8]) -> Option<u64> {
    usage_events(bytes)
        .filter_map(|event| counter(&event, "context_window"))
        .filter(|window| *window > 0)
        .last()
}
