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
/// is what exhausts `max_total_tokens` (ARCH §6), and it is **not** the four
/// summed: the counters overlap on some providers, so the prompt is folded
/// once ([`prompt_tokens`](BudgetSpend::prompt_tokens)).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BudgetSpend {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_tokens: u64,
    pub cache_write_tokens: u64,
}

impl BudgetSpend {
    /// The cached slice this reading reports — read plus write. Zero on a
    /// provider that reports no cache counters at all.
    pub fn cached_tokens(&self) -> u64 {
        self.cache_read_tokens
            .saturating_add(self.cache_write_tokens)
    }

    /// The prompt these counters describe: `max(input, cache_read +
    /// cache_write)`, so a cached slice that already sits **inside** the prompt
    /// counter is counted once.
    ///
    /// brazen's canonical `Usage` reports each provider's own counters
    /// unaltered, and the providers disagree about overlap: Anthropic's prompt
    /// counters are **disjoint** slices (`input_tokens` beside
    /// `cache_read_input_tokens` / `cache_creation_input_tokens`), while the
    /// OpenAI-shaped and Google decoders map a prompt counter that **contains**
    /// the cached one (`prompt_tokens` ⊇ `prompt_tokens_details.cached_tokens`,
    /// `input_tokens` ⊇ `input_tokens_details.cached_tokens`,
    /// `promptTokenCount` ⊇ `cachedContentTokenCount`), and ollama reports no
    /// cache counters at all. Nothing on the `Usage` event says which shape it
    /// is and a step record carries no protocol, so the fold takes the larger of
    /// the two readings of the prompt rather than their sum: **exact** where the
    /// slice is contained, a **floor** (never an over-statement) where the
    /// counters are disjoint, and plain `input_tokens` where no cache counter is
    /// reported. One formula, no per-provider branch — normalizing the overlap
    /// is brazen's to do, not yog's to guess at.
    pub fn prompt_tokens(&self) -> u64 {
        self.input_tokens.max(self.cached_tokens())
    }

    /// The part of the prompt no cache served — the slice the **input** rate
    /// applies to (§3.5). The fold's remainder by construction: `uncached +
    /// cache_read + cache_write + output` is exactly [`Self::total_tokens`], so
    /// the dollar figure prices the very tokens the token figure counts, and the
    /// two can never tell different stories about one usage line.
    pub fn uncached_prompt_tokens(&self) -> u64 {
        self.input_tokens.saturating_sub(self.cached_tokens())
    }

    /// Total tokens billed against the tree's `max_total_tokens` ceiling:
    /// `max(input, cache_read + cache_write) + output` (ARCH §6 "The cached
    /// slice is billed once").
    ///
    /// **Lockstep with lernie**, whose `prompt/budget/derive.rs::usage_tokens`
    /// folds the identical shape (lernie bl-68f5): this figure is a *preview* of
    /// the one that exhausts `max_total_tokens` one layer down, so the two must
    /// be the same arithmetic — change one only by changing both. A floor rather
    /// than a ceiling on purpose: spend is what was really consumed, and billing
    /// a prompt twice ends a conversation before the bound its operator
    /// declared. Collapses back to the plain four-counter sum the day brazen
    /// normalizes the overlap in its decoders (brazen bl-d192) — the named exit,
    /// and the only thing that would make a sum correct here.
    pub fn total_tokens(&self) -> u64 {
        self.prompt_tokens().saturating_add(self.output_tokens)
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
mod tests;
