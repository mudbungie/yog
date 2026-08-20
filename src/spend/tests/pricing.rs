//! The price table itself (DESIGN §3.5, §4.1 `prices`): its forgiving parse,
//! its micro-USD arithmetic over the three-way partition of a prompt, and the
//! cent-resolution render. The join that spends it is `super`.

use crate::budgets::BudgetSpend;
use crate::spend::{Cost, Prices};
use serde_json::json;

#[test]
fn a_malformed_table_degrades_row_by_row() {
    let prices = Prices::from_json(&json!({
        "opus": { "input": 3, "output": "nope", "cache_read": -5 },
        "broken": "not an object",
    }));
    assert!(prices.of(Some("broken")).is_none());
    assert!(prices.of(Some("absent")).is_none());
    assert!(prices.of(None).is_none());
    let price = prices.of(Some("opus")).unwrap();
    assert_eq!(price.input, 3_000_000);
    assert_eq!(price.output, 0, "a non-numeric rate reads zero");
    assert_eq!(price.cache_read, 0, "a negative rate reads zero");
    // Not an object at all.
    assert!(Prices::from_json(&json!([1, 2])).is_empty());
}

#[test]
fn money_renders_at_cent_resolution_without_conflating_small_with_none() {
    assert_eq!(Cost::default().usd(), "$0.00");
    assert_eq!(
        Cost {
            micro_usd: 5_000,
            unpriced_tokens: 0
        }
        .usd(),
        "<$0.01"
    );
    assert_eq!(
        Cost {
            micro_usd: 10_000,
            unpriced_tokens: 0
        }
        .usd(),
        "$0.01"
    );
    assert_eq!(
        Cost {
            micro_usd: 1_234_567,
            unpriced_tokens: 0
        }
        .usd(),
        "$1.23"
    );
}

/// The dollar half of bl-6621, on all three provider shapes. A rate table with
/// distinct `input` and `cache_read` rates describes disjoint slices, so the
/// prompt is partitioned before it is priced: the cached slice is billed at the
/// cache rate **or** the input rate, never both.
#[test]
fn a_contained_cached_slice_is_priced_once_and_the_partition_is_exhaustive() {
    // $2/Mtok in, $8 out, $0.20 cache read, $2.50 cache write.
    let price = Prices::from_json(
        &json!({ "opus": { "input": 2, "output": 8, "cache_read": 0.2, "cache_write": 2.5 } }),
    )
    .of(Some("opus"))
    .unwrap();

    // Contained (OpenAI-shaped, Google): 0.9 Mtok of the 1 Mtok prompt came out
    // of cache. 0.1 Mtok at $2 + 0.9 Mtok at $0.20 + 0.1 Mtok out at $8 = $1.18.
    // The four-counter sum billed the cached 0.9 Mtok twice, for $2.98.
    let contained = BudgetSpend {
        input_tokens: 1_000_000,
        output_tokens: 100_000,
        cache_read_tokens: 900_000,
        cache_write_tokens: 0,
    };
    assert_eq!(price.cost(contained), 1_180_000);

    // Disjoint (Anthropic): `input` is the uncached tail beside the cached
    // slices, so the whole cached mass prices at its own two rates — a floor,
    // short by the same tail the token fold is short by.
    let disjoint = BudgetSpend {
        input_tokens: 100_000,
        output_tokens: 0,
        cache_read_tokens: 900_000,
        cache_write_tokens: 200_000,
    };
    assert_eq!(price.cost(disjoint), 680_000);

    // No cache counters at all (ollama): plain input + output.
    let uncached = BudgetSpend {
        input_tokens: 500_000,
        output_tokens: 100_000,
        ..BudgetSpend::default()
    };
    assert_eq!(price.cost(uncached), 1_800_000);

    // And on every shape, one flat rate over the whole partition prices exactly
    // the tokens the token figure counts — the two figures cannot diverge.
    let flat = Prices::from_json(
        &json!({ "m": { "input": 1, "output": 1, "cache_read": 1, "cache_write": 1 } }),
    )
    .of(Some("m"))
    .unwrap();
    for s in [contained, disjoint, uncached] {
        assert_eq!(flat.cost(s), s.total_tokens(), "{s:?}");
    }
}

#[test]
fn a_nonsense_rate_saturates_rather_than_wrapping() {
    let prices = Prices::from_json(&json!({ "opus": { "input": 1e300 } }));
    let price = prices.of(Some("opus")).unwrap();
    let cost = price.cost(BudgetSpend {
        input_tokens: u64::MAX,
        ..BudgetSpend::default()
    });
    assert!(cost > 0, "saturating, never a wrap to a small figure");
}
