//! The `(provider, model)` table (DESIGN §3.5, bl-53d1): the one lookup's
//! five arms — exact, the row's `*`, unpriced, unique-by-model, and
//! ambiguous-by-model — the zero row that prices to `$0.00`, the old
//! model-keyed document reading empty, and the write-through's spelling
//! round-tripping through `to_json`.

use crate::budgets::BudgetSpend;
use crate::spend::{ANY, Price, PriceRow, Prices, decimal, parse_usd, priced};
use serde_json::json;

/// Two rows: an API-key row pricing `opus` at list with a `*` for the rest,
/// and a subscription row pricing everything it serves at zero.
fn table() -> Prices {
    Prices::from_json(&json!({
        "anthropic": { "opus": { "input": 15, "output": 75 }, "*": { "input": 3 } },
        "claude-session-direct": { "*": {} },
    }))
}

#[test]
fn the_lookup_has_five_arms() {
    let t = table();
    // Exact `(provider, model)`.
    assert_eq!(
        t.of(Some("anthropic"), Some("opus")).map(|p| p.input),
        Some(15_000_000)
    );
    // The row's `*` for a model it does not name.
    assert_eq!(
        t.of(Some("anthropic"), Some("haiku")).map(|p| p.input),
        Some(3_000_000)
    );
    // Unpriced: a row the table has not got, and a step naming no model.
    assert_eq!(t.of(Some("openai"), Some("opus")), None);
    assert_eq!(t.of(Some("anthropic"), None), None);
    // No provider named: unique by model over every row — only one row
    // prices `sonnet`… no, every row here has a `*`, so it is ambiguous.
    assert_eq!(t.of(None, Some("opus")), None, "two rows price it");
    let one = Prices::from_json(&json!({ "anthropic": { "opus": { "input": 1 } } }));
    assert_eq!(one.of(None, Some("opus")).map(|p| p.input), Some(1_000_000));
    assert_eq!(one.of(None, Some("haiku")), None, "no row prices it");
}

/// `{}` is a row priced at zero — `Some`, and `$0.00` — which is a different
/// fact from unpriced, and the figure says both without a status word.
#[test]
fn a_zero_row_prices_to_nothing_rather_than_being_unpriced() {
    let t = table();
    let price = t
        .of(Some("claude-session-direct"), Some("sonnet"))
        .expect("the subscription row prices everything it serves");
    assert_eq!(price, Price::default());
    let bill = crate::budgets::StepBill {
        conv: "c".to_owned(),
        seq: "001".to_owned(),
        model: Some("sonnet".to_owned()),
        provider: Some("claude-session-direct".to_owned()),
        spend: BudgetSpend {
            input_tokens: 1_000_000,
            ..BudgetSpend::default()
        },
        last_usage: BudgetSpend::default(),
        window: None,
        wall_secs: 0,
    };
    let cost = priced(std::slice::from_ref(&bill), &t).expect("a priced world");
    assert_eq!(cost.micro_usd, 0);
    assert_eq!(cost.unpriced_tokens, 0, "priced, at nothing");
    assert_eq!(cost.usd(), "$0.00");
    // The same tokens through an unknown row are the other fact.
    let unknown = crate::budgets::StepBill {
        provider: Some("nobody".to_owned()),
        ..bill
    };
    let cost = priced(std::slice::from_ref(&unknown), &t).expect("still a priced world");
    assert_eq!(cost.unpriced_tokens, 1_000_000);
}

#[test]
fn the_old_model_keyed_shape_reads_empty() {
    let old = Prices::from_json(&json!({ "opus": { "input": 15, "output": 75 } }));
    assert!(old.is_empty());
    assert!(old.rows().is_empty());
}

/// The write-through's spelling: rows flatten in key order, `*` as the
/// literal string, and `to_json` → `from_json` is the identity.
#[test]
fn rows_flatten_in_key_order_and_the_document_round_trips() {
    let t = table();
    let rows = t.rows();
    assert_eq!(
        rows.iter()
            .map(|r| (r.provider.as_str(), r.model.as_str()))
            .collect::<Vec<_>>(),
        [
            ("anthropic", ANY),
            ("anthropic", "opus"),
            ("claude-session-direct", ANY)
        ]
    );
    assert_eq!(
        rows[1],
        PriceRow {
            provider: "anthropic".to_owned(),
            model: "opus".to_owned(),
            rates: Price {
                input: 15_000_000,
                output: 75_000_000,
                cache_read: 0,
                cache_write: 0,
            },
        }
    );
    let doc = t.to_json();
    assert_eq!(doc["anthropic"]["opus"]["input"], 15);
    assert_eq!(doc["claude-session-direct"]["*"]["output"], 0);
    assert_eq!(Prices::from_json(&doc), t);
}

/// `set` writes, overwrites and deletes, and a row emptied by its last delete
/// leaves the document rather than lingering as a row that prices nothing.
#[test]
fn set_writes_overwrites_and_deletes_and_an_emptied_row_goes() {
    let mut t = Prices::default();
    let rate = |input: u64| Price {
        input,
        ..Price::default()
    };
    t.set("p", "m", Some(rate(1_000_000)));
    t.set("p", "m", Some(rate(2_000_000)));
    assert_eq!(t.of(Some("p"), Some("m")), Some(rate(2_000_000)));
    t.set("p", "n", Some(rate(3)));
    t.set("p", "m", None);
    assert_eq!(t.rows().len(), 1);
    t.set("q", "absent", None);
    t.set("p", "n", None);
    assert!(t.is_empty(), "the last delete empties the table");
}

/// The one `f64` on the way in and the one on the way out: a typed word reads
/// as micro-USD, refusing a negative or a non-number, and a figure written
/// back reads as the decimal it was — never `15.0`, never a lost digit.
#[test]
fn a_figure_typed_and_a_figure_written_agree_to_the_micro() {
    assert_eq!(parse_usd("15"), Some(15_000_000));
    assert_eq!(parse_usd("18.75"), Some(18_750_000));
    assert_eq!(parse_usd("0"), Some(0));
    assert_eq!(parse_usd("-1"), None);
    assert_eq!(parse_usd("lots"), None);
    assert_eq!(parse_usd("inf"), None);
    assert_eq!(decimal(15_000_000).to_string(), "15");
    assert_eq!(decimal(18_750_000).to_string(), "18.75");
    assert_eq!(decimal(200_000).to_string(), "0.2");
    for micro in [0, 1, 200_000, 1_500_000, 18_750_000, 123_456_789] {
        assert_eq!(
            crate::spend::quoted(Some(&decimal(micro))),
            Some(micro),
            "{micro}"
        );
    }
}
