//! **The step row's money** (DESIGN §3.5, REMOTE §9.23; bl-53d1):
//! every carrier of a token count says the cost the table puts on it, and the
//! Steps list is the first carrier — one figure per step, that step's own
//! bill priced by its own `(provider, model)`.
//!
//! **The join is over the worker's walk, not a second one** (bl-9dd4). The
//! list itself is read live off `steps/` by [`super::build`]; what it costs is
//! answered from `Snapshot::bills`, the walk the derivation worker already
//! made of this workspace, matched to each row by the conversation and the
//! sequence the bill already carries. A step written since the last
//! derivation therefore has no bill yet and answers no cost — absent, which
//! is the reading QUALITY H2 asks for of a fact yog cannot yet derive, never
//! a zero and never a price guessed off a sibling.

use crate::budgets::StepBill;
use crate::spend::Prices;

use super::StepsView;

impl StepsView {
    /// Every row's cost, off `bills` — the workspace's pre-walked set — at
    /// `prices`. `None` on every row when the table is empty (the §3.5
    /// severability gate) or when the row's bill is not in the walk yet.
    #[must_use]
    pub fn priced(mut self, bills: &[StepBill], agent: &str, prices: &Prices) -> Self {
        for step in &mut self.steps {
            step.cost = bills
                .iter()
                .find(|bill| bill.conv == agent && bill.seq == step.seq)
                .and_then(|bill| crate::spend::priced(std::slice::from_ref(bill), prices));
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use crate::budgets::{BudgetSpend, StepBill};
    use crate::spend::Prices;
    use crate::steps_view::{StepSummary, StepsView, Wound};
    use serde_json::json;

    fn step(seq: &str) -> StepSummary {
        StepSummary {
            seq: seq.to_owned(),
            framing: crate::git_tree::Framing::Complete,
            attempts: 1,
            tokens: BudgetSpend::default(),
            cost: None,
            commit: None,
            started_at: None,
            ended_at: None,
            wound: Wound::None,
        }
    }

    fn bill(conv: &str, seq: &str, model: Option<&str>) -> StepBill {
        StepBill {
            conv: conv.to_owned(),
            seq: seq.to_owned(),
            model: model.map(str::to_owned),
            provider: None,
            spend: BudgetSpend {
                input_tokens: 1_000_000,
                ..BudgetSpend::default()
            },
            last_usage: BudgetSpend::default(),
            window: None,
            wall_secs: 0,
        }
    }

    /// Each row is joined to its own bill and nobody else's: a priced step,
    /// an unpriced one (the floor), a step the walk has not billed, and the
    /// whole list unpriced when the table is empty.
    #[test]
    fn every_row_takes_its_own_bill_and_an_empty_table_prices_none() {
        let view = StepsView {
            steps: vec![step("001"), step("002"), step("003")],
            orphan: crate::steps_view::Orphan::None,
        };
        let bills = [
            bill("a", "001", Some("opus")),
            bill("a", "002", Some("mystery")),
            bill("b", "003", Some("opus")),
        ];
        let table = Prices::from_json(&json!({ "p": { "opus": { "input": 2 } } }));
        let priced = view.clone().priced(&bills, "a", &table);
        let costs: Vec<Option<(u64, u64)>> = priced
            .steps
            .iter()
            .map(|s| s.cost.map(|c| (c.micro_usd, c.unpriced_tokens)))
            .collect();
        assert_eq!(costs, [Some((2_000_000, 0)), Some((0, 1_000_000)), None]);
        let unpriced = view.priced(&bills, "a", &Prices::default());
        assert!(unpriced.steps.iter().all(|s| s.cost.is_none()));
    }
}
