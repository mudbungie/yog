//! The price table's **parse and arithmetic** (DESIGN §3.5, §4.1 `prices`),
//! split off [`super`] at §12's pre-split band when the table gained its
//! provider dimension (bl-53d1): one rate's shape, the forgiving read of one
//! quoted figure, the micro-USD fold over the §3.5 prompt partition, and the
//! one rendering that writes a figure back as the decimal the operator typed.
//! The lookup — which rate a `(provider, model)` takes — is [`super`]'s.
//!
//! Money is **micro-USD integers**, never `f64`: the quoted rate is parsed
//! once from the operator's decimal USD and every arithmetic step after that
//! is exact and saturating. The `f64` exists on the way in ([`quoted`]) and on
//! the way out ([`decimal`]), and nothing between the two sees one.

use serde_json::{Map, Value};

use crate::budgets::BudgetSpend;

/// Micro-USD in one USD — the unit every stored and computed figure is in.
pub const MICRO_PER_USD: u64 = 1_000_000;
/// Micro-USD in one cent, the resolution [`super::super::Cost::usd`] renders at.
pub const MICRO_PER_CENT: u64 = 10_000;
/// Tokens a rate is quoted per — the industry's per-million convention, so
/// the operator writes the number the provider's price page prints.
const TOKENS_PER_QUOTE: u64 = 1_000_000;
/// [`MICRO_PER_USD`] as the one parse's multiplier. A literal rather than a
/// `u64 as f64`, which is the precision-loss cast this tree denies.
const MICRO_PER_USD_F: f64 = 1_000_000.0;

/// The four rate keys, in the order a row is written (§4.1).
const RATES: [&str; 4] = ["input", "output", "cache_read", "cache_write"];

/// One model's four rates, in **micro-USD per million tokens**. The four
/// counters are brazen's own (`BudgetSpend`), so a table that priced only
/// input/output leaves cache traffic at zero rather than guessing it.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Price {
    pub input: u64,
    pub output: u64,
    pub cache_read: u64,
    pub cache_write: u64,
}

impl Price {
    /// Micro-USD billed for `spend` at this price, over the same overlap fold
    /// the token figure uses ([`BudgetSpend::total_tokens`]).
    ///
    /// **A rate table with distinct `input` and `cache_read` rates describes
    /// disjoint slices by construction, so it must be fed disjoint ones.** The
    /// prompt is partitioned into exactly three non-overlapping parts — the
    /// cached read, the cache write, and the uncached remainder
    /// [`BudgetSpend::uncached_prompt_tokens`] (`max(input - (cache_read +
    /// cache_write), 0)`) — and each is priced at its own rate. Where the cached
    /// slice is *contained* in the prompt counter (OpenAI-shaped, Google — which
    /// report no cache-write counter at all) that remainder is precisely what
    /// the provider charges the input rate for, so the cached tokens are billed
    /// once instead of at the input rate **and** the cache-read rate. Where the
    /// counters are disjoint (Anthropic) it is a floor, short by the same tail
    /// the token fold is short by — the two stay in lockstep because the tokens
    /// priced here sum to `spend.total_tokens()` exactly.
    ///
    /// Saturating throughout: a nonsense rate in a hand-edited table yields a
    /// huge figure, never a panic and never a wrap to a small one.
    pub fn cost(&self, spend: BudgetSpend) -> u64 {
        rate(spend.uncached_prompt_tokens(), self.input)
            .saturating_add(rate(spend.output_tokens, self.output))
            .saturating_add(rate(spend.cache_read_tokens, self.cache_read))
            .saturating_add(rate(spend.cache_write_tokens, self.cache_write))
    }

    /// One table entry: an object of USD-per-million rates. A non-object entry
    /// is no entry at all; a rate that is absent, non-numeric or negative
    /// reads zero — a rate has no use for the absent/zero distinction a
    /// ceiling turns on.
    pub(super) fn from_json(value: &Value) -> Option<Self> {
        let row = value.as_object()?;
        let micros = |key: &str| quoted(row.get(key)).unwrap_or(0);
        Some(Self {
            input: micros("input"),
            output: micros("output"),
            cache_read: micros("cache_read"),
            cache_write: micros("cache_write"),
        })
    }

    /// The four rates as the decimals the operator quoted, under their §4.1
    /// keys — the one spelling `ui.json` stores and the `prices` reply carries,
    /// so the two cannot drift.
    pub fn json_fields(&self) -> Map<String, Value> {
        RATES
            .iter()
            .zip([self.input, self.output, self.cache_read, self.cache_write])
            .map(|(key, micro)| ((*key).to_owned(), decimal(micro)))
            .collect()
    }
}

/// One counter × its per-million rate.
fn rate(tokens: u64, per_quote: u64) -> u64 {
    tokens.saturating_mul(per_quote) / TOKENS_PER_QUOTE
}

/// One quoted USD figure as micro-USD, or `None` when it is absent,
/// non-numeric or negative — the forgiving read, with the *presence* kept,
/// because [`super::super::Ceiling`] has to tell "no key" from "zero". The
/// `f64` stops here: nothing downstream sees one.
pub(crate) fn quoted(value: Option<&Value>) -> Option<u64> {
    let usd = value.and_then(Value::as_f64).filter(|usd| *usd >= 0.0)?;
    Some((usd * MICRO_PER_USD_F).round() as u64)
}

/// One typed USD word as micro-USD — the line's reader — or `None` for a word
/// that is not a non-negative number. The same read as [`quoted`], through
/// the same one `f64`, so a rate typed and a rate stored cannot round apart.
pub(crate) fn parse_usd(word: &str) -> Option<u64> {
    quoted(Some(&Value::from(word.parse::<f64>().ok()?)))
}

/// A micro-USD figure as the JSON decimal it was quoted as — `15`, `1.5`,
/// `18.75` — read back exactly by [`quoted`] for every figure of at most six
/// decimals, which is every figure this unit can hold. Rendered as text and
/// parsed as a number rather than divided, because a `u64 as f64` is the cast
/// this tree denies and a whole-dollar figure must not come back as `15.0`.
pub(crate) fn decimal(micro: u64) -> Value {
    let (whole, frac) = (micro / MICRO_PER_USD, micro % MICRO_PER_USD);
    let text = if frac == 0 {
        whole.to_string()
    } else {
        format!("{whole}.{frac:06}")
            .trim_end_matches('0')
            .to_owned()
    };
    serde_json::from_str(&text).unwrap_or_default()
}
