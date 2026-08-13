//! The price table — yog world config (DESIGN §3.5 and §4.1 `prices`).
//!
//! **Severable by construction:** the table is a `ui.json` object keyed by
//! model id. Delete the key and every cost figure disappears, deleting a
//! column and not a code path — [`Prices::is_empty`] is the one gate, and an
//! empty table is the default, so a yog that was never priced renders exactly
//! the token figures it always did.
//!
//! **No crate below yog learns a price** (§3.5): brazen counts tokens, lernie
//! commits them into step records, balls stays metric-free. The rate lives
//! here and nowhere else.
//!
//! Money is **micro-USD integers**, never `f64`: the quoted rate is parsed
//! once from the operator's decimal USD and every arithmetic step after that
//! is exact and saturating.

use std::collections::BTreeMap;

use serde_json::Value;

use crate::budgets::BudgetSpend;

/// Micro-USD in one USD — the unit every stored and computed figure is in.
pub const MICRO_PER_USD: u64 = 1_000_000;
/// Micro-USD in one cent, the resolution [`super::Cost::usd`] renders at.
pub const MICRO_PER_CENT: u64 = 10_000;
/// Tokens a rate is quoted per — the industry's per-million convention, so
/// the operator writes the number the provider's price page prints.
const TOKENS_PER_QUOTE: u64 = 1_000_000;
/// [`MICRO_PER_USD`] as the one parse's multiplier. A literal rather than a
/// `u64 as f64`, which is the precision-loss cast this tree denies.
const MICRO_PER_USD_F: f64 = 1_000_000.0;

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
    /// Micro-USD billed for `spend` at this price. Saturating throughout: a
    /// nonsense rate in a hand-edited table yields a huge figure, never a
    /// panic and never a wrap to a small one.
    pub fn cost(&self, spend: BudgetSpend) -> u64 {
        rate(spend.input_tokens, self.input)
            .saturating_add(rate(spend.output_tokens, self.output))
            .saturating_add(rate(spend.cache_read_tokens, self.cache_read))
            .saturating_add(rate(spend.cache_write_tokens, self.cache_write))
    }
}

/// One counter × its per-million rate.
fn rate(tokens: u64, per_quote: u64) -> u64 {
    tokens.saturating_mul(per_quote) / TOKENS_PER_QUOTE
}

/// Model id → [`Price`]. Empty is the default and means *unpriced*, not free:
/// [`super::figure`] renders no cost at all rather than `$0.00`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Prices(BTreeMap<String, Price>);

impl Prices {
    /// Read the table from `ui.json`'s `prices` value (§4.1). Forgiving like
    /// every other `ui.json` read: a non-object document, a non-object row, or
    /// a non-numeric rate degrades to "absent" rather than refusing to load —
    /// a typo in a hand-edited table must not cost the operator the window.
    pub fn from_json(value: &Value) -> Self {
        let Some(rows) = value.as_object() else {
            return Self::default();
        };
        Self(
            rows.iter()
                .filter_map(|(model, row)| Some((model.clone(), price(row)?)))
                .collect(),
        )
    }

    /// No rate for anything — the severability gate (§3.5).
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// The rate for a step's model, or `None` when the step named no model or
    /// named one the table does not price. Those tokens are reported as
    /// unpriced (`super::Cost::unpriced_tokens`), never silently free.
    pub fn of(&self, model: Option<&str>) -> Option<Price> {
        self.0.get(model?).copied()
    }
}

/// One table row: an object of USD-per-million rates. A non-object row is no
/// row at all.
fn price(value: &Value) -> Option<Price> {
    let row = value.as_object()?;
    Some(Price {
        input: micros(row.get("input")),
        output: micros(row.get("output")),
        cache_read: micros(row.get("cache_read")),
        cache_write: micros(row.get("cache_write")),
    })
}

/// One quoted USD figure as micro-USD, or `None` when it is absent,
/// non-numeric or negative — the forgiving read, with the *presence* kept,
/// because [`super::Ceiling`] has to tell "no key" from "zero". The `f64`
/// stops here: nothing downstream sees one.
pub(super) fn quoted(value: Option<&Value>) -> Option<u64> {
    let usd = value.and_then(Value::as_f64).filter(|usd| *usd >= 0.0)?;
    Some((usd * MICRO_PER_USD_F).round() as u64)
}

/// One quoted rate as micro-USD, an unreadable one reading zero — a rate has
/// no use for the absent/zero distinction a ceiling turns on.
fn micros(value: Option<&Value>) -> u64 {
    quoted(value).unwrap_or(0)
}
