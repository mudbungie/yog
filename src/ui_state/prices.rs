//! The §3.5 price table's one home in `ui.json` (DESIGN §4.1 `prices`).
//!
//! Read-only, and deliberately so: no setter, no editor, no verb. The rates
//! are operator policy with no other authority, hand-written into the file yog
//! already keeps — a second config artifact for one object would give one fact
//! two homes (the same reasoning §4.1 records for the density knobs). A hand
//! edit is live within a tick: the fs watcher's whole-file `adopt` (§4.1, I5)
//! is already the convergence path for an externally-changed `ui.json`.
//!
//! Absent ⇒ an empty table ⇒ no cost anywhere. That is the severability §3.5
//! demands, and it is one `get` away from being obvious.

use super::UiState;

/// The price-table key (§4.1): model id → USD-per-million-token rates.
const PRICES: &str = "prices";

impl UiState {
    /// The §3.5 price table. Absent, or of the wrong shape, reads empty — the
    /// forgiving read every `ui.json` key gets, so a typo costs a column and
    /// never the window.
    pub fn prices(&self) -> crate::spend::Prices {
        match self.root.get(PRICES) {
            Some(value) => crate::spend::Prices::from_json(value),
            None => crate::spend::Prices::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::budgets::BudgetSpend;
    use crate::ui_state::UiState;
    use tempfile::tempdir;

    fn opened(doc: &str) -> UiState {
        let dir = tempdir().unwrap();
        let path = dir.path().join("ui.json");
        std::fs::write(&path, doc).unwrap();
        UiState::open(path)
    }

    #[test]
    fn absent_table_is_empty() {
        assert!(opened(r#"{"v":1}"#).prices().is_empty());
    }

    #[test]
    fn wrong_shape_is_empty() {
        assert!(opened(r#"{"v":1,"prices":"nope"}"#).prices().is_empty());
    }

    #[test]
    fn reads_rates_for_a_model() {
        let prices = opened(
            r#"{"v":1,"prices":{"opus":{"input":15,"output":75,
               "cache_read":1.5,"cache_write":18.75}}}"#,
        )
        .prices();
        let price = prices.of(Some("opus")).unwrap();
        assert_eq!(price.input, 15_000_000);
        assert_eq!(price.cache_write, 18_750_000);
        // One million input tokens at $15/Mtok is $15.
        assert_eq!(
            price.cost(BudgetSpend {
                input_tokens: 1_000_000,
                ..BudgetSpend::default()
            }),
            15_000_000
        );
    }
}
