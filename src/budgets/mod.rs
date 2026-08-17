//! Whole-tree budget-spend fold (DESIGN §5.1 #16; ARCH §6 budgets).
//!
//! Yog is a pure reader (§3.5). Budget *spent* is derived, never stored:
//! sum every brazen `Usage` event's token counters across **every attempt
//! segment of every step** under a root agent and its entire hyphenated
//! descent — `steps/<root>/` plus every `steps/<root>-*/` (the one shared
//! `steps/` subtree; budgets are whole-tree consumables, ARCH §6, §2.2).
//! This mirrors lernie's own `budget::spend` derivation exactly — a failed
//! or superseded attempt still burned tokens and real money (§4.4, §6) —
//! so the figure yog shows is the figure that exhausts `max_total_tokens`.
//!
//! Forgiving by construction: a missing `steps/` tree, an unreadable step
//! dir, a missing `response.json`, or a malformed / forward-compat event
//! line each contributes zero — the fold never panics on a partial or
//! mid-stream tree.
//!
//! This module is the **fold only** — it counts tokens and never learns a
//! price, mirroring brazen's own boundary (§3.5; the vision's stated brazen boundary). Pricing the fold and
//! rendering the figure both live in [`crate::spend`], which owns the price
//! table; the tree walk both share is [`bills`].

mod bills;
pub use bills::{Scope, StepBill, bills, total, wall};

/// Conv-repo subdir of per-conversation step records (ARCH §2.2).
const STEPS_DIR: &str = "steps";
/// Per-step JSONL of `v=1` events (ARCH §2.3, §4.4).
const RESPONSE_FILE: &str = "response.json";
/// Zero-padded step-sequence width (`001`, `002`, …) per ARCH §2.3.
const STEP_SEQ_WIDTH: usize = 3;

/// Tokens spent by a root agent and its descent, split by the four brazen
/// `Usage` counters. The wire shape is
/// `{"type":"usage","input_tokens":N,"output_tokens":M,
/// "cache_read_tokens":R,"cache_write_tokens":W}` (brazen
/// `canonical::event`) — every counter nullable, a `null`/absent field
/// counting **zero, never fabricated**. [`total_tokens`](BudgetSpend::total_tokens)
/// is what exhausts `max_total_tokens` (ARCH §6: all four summed).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BudgetSpend {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_tokens: u64,
    pub cache_write_tokens: u64,
}

impl BudgetSpend {
    /// Total tokens billed against the tree's `max_total_tokens` ceiling —
    /// all four counters summed (ARCH §6).
    pub fn total_tokens(&self) -> u64 {
        self.input_tokens + self.output_tokens + self.cache_read_tokens + self.cache_write_tokens
    }

    fn add(&mut self, other: BudgetSpend) {
        self.input_tokens = self.input_tokens.saturating_add(other.input_tokens);
        self.output_tokens = self.output_tokens.saturating_add(other.output_tokens);
        self.cache_read_tokens = self
            .cache_read_tokens
            .saturating_add(other.cache_read_tokens);
        self.cache_write_tokens = self
            .cache_write_tokens
            .saturating_add(other.cache_write_tokens);
    }
}

/// Fold every `Usage` line of a `response.json` payload — every attempt
/// segment (ARCH §6). Pure over the bytes: the per-step unit the Y13 steps
/// inspector reuses so the brazen Usage-event vocabulary lives in exactly
/// one place (single source of truth). A malformed or non-`Usage` line
/// contributes zero.
pub fn spend_from_bytes(bytes: &[u8]) -> BudgetSpend {
    let mut total = BudgetSpend::default();
    for line in bytes.split(|b| *b == b'\n') {
        if let Some(spend) = usage_line(line) {
            total.add(spend);
        }
    }
    total
}

/// The **last** `Usage` line's counters, not the fold — the final attempt
/// segment's, which is the only one whose prompt still describes the context
/// as it now stands (§5.1 #35). Summing segments answers *spend*; the last
/// segment answers *fullness*, and a step retried three times would read as a
/// context three times its real size under the fold. Zero when the payload
/// carries no `Usage` line at all — the general path with no inputs.
pub fn last_usage(bytes: &[u8]) -> BudgetSpend {
    bytes
        .split(|b| *b == b'\n')
        .filter_map(usage_line)
        .next_back()
        .unwrap_or_default()
}

/// The counters of one JSONL event line iff it is a `{"type":"usage",…}`
/// event (brazen's internally-tagged `Event::Usage`). A non-`usage` type,
/// a line that does not parse, or one with no string `type` yields `None`.
fn usage_line(line: &[u8]) -> Option<BudgetSpend> {
    let value: serde_json::Value = serde_json::from_slice(line).ok()?;
    if value.get("type")?.as_str()? != "usage" {
        return None;
    }
    Some(BudgetSpend {
        input_tokens: counter(&value, "input_tokens"),
        output_tokens: counter(&value, "output_tokens"),
        cache_read_tokens: counter(&value, "cache_read_tokens"),
        cache_write_tokens: counter(&value, "cache_write_tokens"),
    })
}

/// One `Usage` counter as `u64`: a `null`, absent, or non-integer field is
/// zero — brazen's Usage-Option contract (a counter a provider never
/// reported is unknown, rendered zero, never fabricated).
fn counter(value: &serde_json::Value, key: &str) -> u64 {
    value
        .get(key)
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(s.total_tokens(), 21);
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
}
