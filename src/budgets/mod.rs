//! Whole-tree budget-spend fold (DESIGN §5.1 #16; ARCH §6 budgets).
//!
//! Yog is a pure reader (§3.5). Budget *spent* is derived, never stored:
//! sum every brazen `Usage` event's token counters across **every attempt
//! segment of every step** under a root agent and its entire hyphenated
//! descent — `steps/<root>/` plus every `steps/<root>-*/` (the one shared
//! `steps/` subtree; budgets are whole-tree consumables, ARCH §6, §2.2).
//! This mirrors litany's own `budget::spend` derivation exactly — a failed
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
mod last;
pub use bills::{Scope, StepBill, bills, total, wall};
pub use last::{context_window, last_usage};

/// Conv-repo subdir of per-conversation step records (ARCH §2.2).
const STEPS_DIR: &str = "steps";
/// Per-step JSONL of `v=1` events (ARCH §2.3, §4.4).
const RESPONSE_FILE: &str = "response.json";
/// Zero-padded step-sequence width (`001`, `002`, …) per ARCH §2.3.
const STEP_SEQ_WIDTH: usize = 3;

/// Tokens spent by a root agent and its descent, split by the four brazen
/// `Usage` counters. The wire shape is
/// `{"type":"usage","input_tokens":N,"output_tokens":M,
/// "cache_read_tokens":R,"cache_write_tokens":W,"input_total_tokens":T}`
/// (brazen `canonical::event`) — every counter nullable, a `null`/absent field
/// counting **zero, never fabricated**. `input_tokens` here is the **whole
/// prompt** — `T` where the line carries it (brazen 0.0.10), else `N`, which
/// [`prompt_tokens`](BudgetSpend::prompt_tokens) reads by the containment
/// rule below. [`total_tokens`](BudgetSpend::total_tokens)
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

    /// The prompt these counters describe. **Where the line carried
    /// `input_total_tokens`, this is that number and the `max` decides
    /// nothing**: the served total already contains the cached slice, so it is
    /// never below `cache_read + cache_write` and the expression returns it
    /// unchanged. The max rule is the **fallback for a line written before that
    /// counter existed**, not the fold's rule.
    ///
    /// Which shape a line is, is decided once, at [`prompt`] — brazen's
    /// `input_total_tokens` (0.0.10, upstream bl-d192) is the whole prompt,
    /// sealed by the decoder that knows the provider's containment shape, and
    /// [`counters`] puts it in [`input_tokens`](Self::input_tokens). So the
    /// containment argument below is history for every record a pinned `bz`
    /// writes, and live only for the older ones on disk.
    ///
    /// **The fallback, and why it is a `max`.** brazen's canonical `Usage`
    /// reported each provider's own counters unaltered, and the providers
    /// disagree about overlap: Anthropic's prompt counters are **disjoint**
    /// slices (`input_tokens` beside `cache_read_input_tokens` /
    /// `cache_creation_input_tokens`), while the OpenAI-shaped and Google
    /// decoders map a prompt counter that **contains** the cached one
    /// (`prompt_tokens` ⊇ `prompt_tokens_details.cached_tokens`,
    /// `input_tokens` ⊇ `input_tokens_details.cached_tokens`,
    /// `promptTokenCount` ⊇ `cachedContentTokenCount`), and ollama reports no
    /// cache counters at all. Nothing on such a line says which shape it is and
    /// a step record carries no protocol, so the fold takes the larger of the
    /// two readings of the prompt rather than their sum: **exact** where the
    /// slice is contained, a **floor** (never an over-statement) where the
    /// counters are disjoint, and plain `input_tokens` where no cache counter is
    /// reported. One expression covers both eras with no version branch and no
    /// per-provider branch, which is why it stays written this way: normalizing
    /// the overlap was brazen's to do and brazen did it.
    pub fn prompt_tokens(&self) -> u64 {
        self.input_tokens.max(self.cached_tokens())
    }

    /// The part of the prompt no cache served — the slice the **input** rate
    /// applies to (§3.5). The fold's remainder by construction: `uncached +
    /// cache_read + cache_write + output` is exactly [`Self::total_tokens`], so
    /// the dollar figure prices the very tokens the token figure counts, and the
    /// two can never tell different stories about one usage line. On a line
    /// carrying `input_total_tokens` it is **exact on every provider shape** —
    /// the served total contains the cached slice, so subtracting it leaves the
    /// tail — where under the fallback it is exact only where the provider's own
    /// counter contained the slice, and zero on a disjoint one.
    pub fn uncached_prompt_tokens(&self) -> u64 {
        self.input_tokens.saturating_sub(self.cached_tokens())
    }

    /// Total tokens billed against the tree's `max_total_tokens` ceiling:
    /// `max(input, cache_read + cache_write) + output` (ARCH §6 "The cached
    /// slice is billed once").
    ///
    /// **Lockstep with litany**, whose `prompt/budget/derive.rs::usage_tokens`
    /// folds the identical shape (litany bl-68f5): this figure is a *preview* of
    /// the one that exhausts `max_total_tokens` one layer down, so the two must
    /// be the same arithmetic — change one only by changing both. A floor rather
    /// than a ceiling on purpose: spend is what was really consumed, and billing
    /// a prompt twice ends a conversation before the bound its operator
    /// declared.
    ///
    /// **The named exit was reached and it was the wrong shape** (bl-bf8b). This
    /// doc used to say the figure "collapses back to the plain four-counter sum
    /// the day brazen normalizes the overlap in its decoders (brazen bl-d192)".
    /// brazen bl-d192 shipped in 0.0.10 and the sum is *more* wrong for it: the
    /// counter it added is the whole prompt **with** the cached slices inside,
    /// so `input + read + write + output` now double-bills them by
    /// construction. What normalization actually bought is one line above —
    /// `prompt_tokens` stops flooring and starts being exact — and this
    /// expression, `prompt + output`, is the collapsed form. There is no
    /// remaining exit to wait for.
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
    for event in usage_events(bytes) {
        total.add(counters(&event));
    }
    total
}

/// Every `{"type":"usage",…}` event line of a JSONL payload (brazen's
/// internally-tagged `Event::Usage`). A non-`usage` type, a line that does not
/// parse, or one with no string `type` is skipped.
pub(crate) fn usage_events(bytes: &[u8]) -> impl Iterator<Item = serde_json::Value> + '_ {
    bytes.split(|b| *b == b'\n').filter_map(|line| {
        let value: serde_json::Value = serde_json::from_slice(line).ok()?;
        (value.get("type")?.as_str()? == "usage").then_some(value)
    })
}

/// One event's counters, each absent one zero.
pub(crate) fn counters(event: &serde_json::Value) -> BudgetSpend {
    BudgetSpend {
        input_tokens: prompt(event).unwrap_or(0),
        output_tokens: counter(event, "output_tokens").unwrap_or(0),
        cache_read_tokens: counter(event, "cache_read_tokens").unwrap_or(0),
        cache_write_tokens: counter(event, "cache_write_tokens").unwrap_or(0),
    }
}

/// The prompt one line reports: brazen's `input_total_tokens` — the whole
/// prompt, cached slices included, sealed by the decoder that knows the
/// provider's containment shape (brazen bl-d192, 0.0.10) — or, on a line
/// written before that counter existed, the provider's own `input_tokens`,
/// which [`BudgetSpend::prompt_tokens`]' max rule still reads. Not a version
/// shim: every step record on disk is one shape or the other forever, and
/// `input_total_tokens` is `None` exactly when `input_tokens` is, so presence
/// is one question.
fn prompt(event: &serde_json::Value) -> Option<u64> {
    counter(event, "input_total_tokens").or_else(|| counter(event, "input_tokens"))
}

/// One `Usage` counter, or `None` for a `null`, absent, or non-integer field —
/// brazen's Usage-Option contract (a counter a provider never reported is
/// unknown, never fabricated).
pub(crate) fn counter(event: &serde_json::Value, key: &str) -> Option<u64> {
    event.get(key).and_then(serde_json::Value::as_u64)
}

#[cfg(test)]
mod tests;
