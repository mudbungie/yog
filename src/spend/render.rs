//! egui widget: one attributed spend figure (§11's "budget-spent figures",
//! §3.5's cost beside it) — seated in the conversation's bottom settings rows
//! since the settings-seat ruling (bl-2e18), not in the altitude-1 header.
//!
//! A pure function of [`Figure`] — the tokens that exhaust `max_total_tokens`
//! (ARCH §6), the four counters behind them, what they cost when a price table
//! exists, and always how honestly the sum is attributed. No click, so it is a
//! headless shape-walk-tested pure render; the fold and the join are tested in
//! [`super`].
//!
//! **The unpriced line is not decoration.** A figure with unpriced tokens is a
//! floor, and it says `+N tok unpriced` so nobody reads a half-priced table as
//! a full answer.
//!
//! **Neither is the attribution clause, and it does not ride on the price
//! table** (bl-1765). The clause says what the sum is *over* — the §3.5 ruling's
//! accepted limit, where a ball no conversation stamps falls back to the whole
//! workspace's spend. That is a fact about the derivation, not about pricing, so
//! it paints whether or not `ui.json.prices` exists. It used to sit below the
//! cost seat's early return, which made the default install — no price table, so
//! no cost — the one configuration that showed a workspace-wide total with
//! nothing marking it as one: two ball rows carrying the same figure, one of
//! them not the ball's, indistinguishable. Severability deletes a *column*
//! (§3.5), never the sentence that keeps a number honest.

use super::Figure;

/// Render one figure: total tokens, the four counters, the dollars and any
/// unpriced remainder when a price table exists, and — independently of pricing
/// — the attribution clause whenever the sum is wider than the seat claims.
///
/// **A part per line, not five parts per row** (§11 rule 1, bl-0424). Laid
/// horizontally these five are one greedy run (the counter clause, ~300 pt of
/// its own) followed by three more, and under rule 1's `Truncate` the greedy
/// one takes the whole remaining width: each part after it is laid at zero
/// available width, paints a bare `…` — which says nothing and is rule 1d's own
/// defect — and allocates ~20 pt past the seat's edge anyway. Measured in a
/// 260 pt navigator column the row laid 319 pt, and in a side panel that rect
/// is next frame's panel width, so this row alone ratcheted the column to its
/// half-window ceiling and held it there against the splitter.
///
/// Given a line apiece every part truncates against the **seat's** width rather
/// than against its neighbours' leavings, so nothing overflows and nothing is
/// elided to nothing. The wide centre settings seat (`shell::settings`) is
/// unaffected in what it says — every part still reaches the glass whole; it
/// says it down the column instead of across it.
pub fn render(ui: &mut egui::Ui, figure: &Figure) {
    ui.vertical(|ui| {
        let spend = figure.tokens;
        ui.label(format!("budget {} tok", spend.total_tokens()));
        ui.weak(format!(
            "(in {} · out {} · cache r {} · w {})",
            spend.input_tokens,
            spend.output_tokens,
            spend.cache_read_tokens,
            spend.cache_write_tokens
        ));
        if let Some(cost) = figure.cost {
            ui.label(cost.usd());
            if cost.unpriced_tokens > 0 {
                ui.weak(format!("+{} tok unpriced", cost.unpriced_tokens))
                    .on_hover_text(
                        "These tokens ran on a model the price table has no rate for \
                         (ui.json `prices`), so the figure beside them is a floor.",
                    );
            }
        }
        if let Some(note) = figure.attribution.note() {
            ui.weak(note.label).on_hover_text(note.hover);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::budgets::BudgetSpend;
    use crate::spend::{Attribution, Cost};

    /// What the seat **shows**, through the one paint walk (bl-36c3). This test
    /// carried its own copy of it, and that copy read `Galley::text()` — the
    /// string that went *in*, so a row egui had truncated to `…` still reported
    /// itself whole and every assertion below was blind to the elision. Both
    /// halves are `paint_probe`'s: the glyph read, and a screen large enough
    /// that what is missing is missing because the row dropped it.
    fn painted(figure: &Figure) -> String {
        crate::paint_probe::paint(|ui| render(ui, figure))
    }

    fn tokens() -> BudgetSpend {
        BudgetSpend {
            input_tokens: 10,
            output_tokens: 5,
            cache_read_tokens: 2,
            cache_write_tokens: 1,
        }
    }

    #[test]
    fn renders_total_and_counters_unpriced() {
        let text = painted(&Figure {
            tokens: tokens(),
            cost: None,
            attribution: Attribution::Conversations(1),
        });
        // 15, not 18: `max(in 10, cache 2+1) + out 5` — the counters overlap on
        // some providers, so the total is a fold and not their sum (bl-6621).
        assert!(text.contains("budget 15 tok"), "got:\n{text}");
        assert!(
            text.contains("in 10 · out 5 · cache r 2 · w 1"),
            "got:\n{text}"
        );
        // No table ⇒ no cost seat at all (§3.5 severability).
        assert!(!text.contains('$'), "got:\n{text}");
    }

    /// The regression (bl-1765). With no price table there is no cost seat, and
    /// the attribution clause used to sit behind it — so exactly the default
    /// install painted a workspace-wide total with nothing saying so. The audit
    /// caught two ball rows carrying one identical figure, only one of which was
    /// that ball's.
    #[test]
    fn a_workspace_wide_sum_says_so_with_no_price_table() {
        let text = painted(&Figure {
            tokens: tokens(),
            cost: None,
            attribution: Attribution::Workspace,
        });
        assert!(!text.contains('$'), "still no cost column: {text}");
        assert!(
            text.contains("workspace-wide"),
            "the honesty clause does not ride on pricing: {text}"
        );
    }

    /// The same independence for the other disclosing arm — a figure summing
    /// several stamped conversations names the count without a price table.
    #[test]
    fn a_multi_conversation_sum_says_so_with_no_price_table() {
        let text = painted(&Figure {
            tokens: tokens(),
            cost: None,
            attribution: Attribution::Conversations(3),
        });
        assert!(text.contains("over 3 conversations"), "got:\n{text}");
    }

    #[test]
    fn renders_zero_spend() {
        assert!(
            painted(&Figure {
                tokens: BudgetSpend::default(),
                cost: None,
                attribution: Attribution::Conversations(1),
            })
            .contains("budget 0 tok")
        );
    }

    #[test]
    fn renders_cost_with_unpriced_remainder_and_attribution() {
        let text = painted(&Figure {
            tokens: tokens(),
            cost: Some(Cost {
                micro_usd: 1_234_500,
                unpriced_tokens: 9,
            }),
            attribution: Attribution::Workspace,
        });
        assert!(text.contains("$1.23"), "got:\n{text}");
        assert!(text.contains("+9 tok unpriced"), "got:\n{text}");
        assert!(text.contains("workspace-wide"), "got:\n{text}");
    }

    #[test]
    fn fully_priced_single_conversation_renders_cost_alone() {
        let text = painted(&Figure {
            tokens: tokens(),
            cost: Some(Cost {
                micro_usd: 0,
                unpriced_tokens: 0,
            }),
            attribution: Attribution::Conversations(1),
        });
        assert!(text.contains("$0.00"), "got:\n{text}");
        assert!(!text.contains("unpriced"), "got:\n{text}");
        assert!(!text.contains("conversations"), "got:\n{text}");
    }
}
