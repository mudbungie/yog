//! The §3.5 price table's one home in `ui.json` (DESIGN §4.1 `prices`).
//!
//! **Read and written through the boundary** since bl-53d1 (`Query::Prices`,
//! `Action::Price`). It was read-only — *no setter, no editor, no verb* — on
//! the argument that a hand edit is live within a tick through the fs
//! watcher's whole-file `adopt` (§4.1, I5); that was true while the window ran
//! in the engine's process on the engine's box, and since REMOTE §12 the seat
//! is another machine and `ui.json` is on the server, so the empty table was
//! the default nobody could leave. The write is the same write-through every
//! `ui.json` mutation takes (`set_pinned`'s shape); the file stays the one
//! home, and a second config artifact for one object would give one fact two
//! homes (the reasoning §4.1 records for the density knobs).
//!
//! Absent ⇒ an empty table ⇒ no cost anywhere. That is the severability §3.5
//! demands, and it is one `get` away from being obvious.

use super::UiState;
use crate::spend::Prices;

/// The price-table key (§4.1): provider row → model id → USD-per-million rates.
const PRICES: &str = "prices";

impl UiState {
    /// The §3.5 price table. Absent, or of the wrong shape, reads empty — the
    /// forgiving read every `ui.json` key gets, so a typo costs a column and
    /// never the window.
    pub fn prices(&self) -> Prices {
        match self.world.root.get(PRICES) {
            Some(value) => Prices::from_json(value),
            None => Prices::default(),
        }
    }

    /// Replace the table — `Action::Price`'s write-through (bl-53d1). An empty
    /// table **deletes the key** rather than storing `{}`, so the document
    /// after the last row is deleted is the document that never had one, and
    /// the §3.5 severability reads the same either way.
    pub fn set_prices(&mut self, prices: &Prices) {
        if prices.is_empty() {
            self.world.root.remove(PRICES);
        } else {
            self.world.root.insert(PRICES.to_owned(), prices.to_json());
        }
        self.world.save();
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

    /// The pre-bl-53d1 model-keyed shape reads as empty: no migration, and
    /// no refusal — the table comes back through `/price`.
    #[test]
    fn the_old_model_keyed_shape_reads_empty() {
        assert!(
            opened(r#"{"v":1,"prices":{"opus":{"input":15,"output":75}}}"#)
                .prices()
                .is_empty()
        );
    }

    /// The write-through, both directions: a row set is read back by the next
    /// open of the same file, and deleting the last row deletes the key.
    #[test]
    fn a_row_written_is_read_back_and_the_last_delete_removes_the_key() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("ui.json");
        let mut ui = UiState::open(path.clone());
        let mut table = crate::spend::Prices::default();
        table.set(
            "anthropic",
            "opus",
            Some(crate::spend::Price {
                input: 15_000_000,
                ..Default::default()
            }),
        );
        ui.set_prices(&table);
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.contains("\"input\": 15"), "{text}");
        assert_eq!(UiState::open(path.clone()).prices(), table);
        table.set("anthropic", "opus", None);
        ui.set_prices(&table);
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(!text.contains("prices"), "{text}");
        assert!(UiState::open(path).prices().is_empty());
    }

    #[test]
    fn reads_rates_for_a_model() {
        let prices = opened(
            r#"{"v":1,"prices":{"anthropic":{"opus":{"input":15,"output":75,
               "cache_read":1.5,"cache_write":18.75}}}}"#,
        )
        .prices();
        let price = prices.of(Some("anthropic"), Some("opus")).unwrap();
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
