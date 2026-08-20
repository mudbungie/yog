//! The §3.5 join: the pricing of a bill set, the bl-9dd4 selection rule, and
//! the two attribution altitudes of a ball figure. The table's own parse and
//! arithmetic are [`pricing`].

mod pricing;

use super::{Attribution, Prices, of_ball, of_conversation, select};
use crate::budgets::{Scope, StepBill};
use serde_json::json;
use std::path::Path;

const ROOT: &str = "20260717T120000Z-root";
const KID: &str = "20260717T120000Z-root-20260717T120100Z-kid0";
const OTHER: &str = "20260717T130000Z-othr";

/// One priced model at $1/Mtok in, $2/Mtok out, cache unpriced.
fn table() -> Prices {
    Prices::from_json(&json!({ "opus": { "input": 1, "output": 2 } }))
}

fn write_step(ws: &Path, conv: &str, seq: &str, input: u64, output: u64, model: Option<&str>) {
    let step = ws.join("steps").join(conv).join(seq);
    std::fs::create_dir_all(&step).unwrap();
    std::fs::write(
        step.join("response.json"),
        format!(r#"{{"type":"usage","input_tokens":{input},"output_tokens":{output}}}"#),
    )
    .unwrap();
    if let Some(model) = model {
        std::fs::write(
            step.join("request.json"),
            format!(r#"{{"model":"{model}"}}"#),
        )
        .unwrap();
    }
}

/// The worker's one walk, which every figure below is then a filter over —
/// the bl-9dd4 seam: `spend` reads bills, never disk.
fn walk(ws: &Path) -> Vec<StepBill> {
    crate::budgets::bills(ws, &Scope::Workspace)
}

#[test]
fn an_absent_table_prices_nothing_and_deletes_the_column() {
    let dir = tempfile::tempdir().unwrap();
    write_step(dir.path(), ROOT, "001", 1_000_000, 0, Some("opus"));
    let figure = of_conversation(&walk(dir.path()), ROOT, &Prices::default());
    assert_eq!(figure.tokens.input_tokens, 1_000_000);
    assert!(figure.cost.is_none(), "empty table ⇒ no cost seat at all");
}

#[test]
fn a_conversation_figure_prices_its_whole_descent() {
    let dir = tempfile::tempdir().unwrap();
    write_step(dir.path(), ROOT, "001", 1_000_000, 500_000, Some("opus"));
    write_step(dir.path(), KID, "001", 2_000_000, 0, Some("opus"));
    // A sibling conversation must not leak in.
    write_step(dir.path(), OTHER, "001", 9_000_000, 0, Some("opus"));

    let figure = of_conversation(&walk(dir.path()), ROOT, &table());
    assert_eq!(figure.attribution, Attribution::Conversations(1));
    assert_eq!(figure.tokens.total_tokens(), 3_500_000);
    // 3 Mtok in at $1 + 0.5 Mtok out at $2 = $4.00.
    let cost = figure.cost.unwrap();
    assert_eq!(cost.micro_usd, 4_000_000);
    assert_eq!(cost.unpriced_tokens, 0);
    assert_eq!(cost.usd(), "$4.00");
}

#[test]
fn a_step_on_an_unpriced_model_is_reported_not_rounded_to_free() {
    let dir = tempfile::tempdir().unwrap();
    write_step(dir.path(), ROOT, "001", 1_000_000, 0, Some("opus"));
    write_step(dir.path(), ROOT, "002", 7, 3, Some("mystery"));
    write_step(dir.path(), ROOT, "003", 5, 0, None);

    let cost = of_conversation(&walk(dir.path()), ROOT, &table())
        .cost
        .unwrap();
    assert_eq!(cost.micro_usd, 1_000_000);
    assert_eq!(cost.unpriced_tokens, 15);
}

#[test]
fn a_stamped_ball_attributes_to_its_conversations() {
    let dir = tempfile::tempdir().unwrap();
    write_step(dir.path(), ROOT, "001", 1_000_000, 0, Some("opus"));
    write_step(dir.path(), OTHER, "001", 2_000_000, 0, Some("opus"));

    let roots = vec![ROOT.to_owned(), OTHER.to_owned()];
    let figure = of_ball(&walk(dir.path()), &roots, &table());
    assert_eq!(figure.attribution, Attribution::Conversations(2));
    assert_eq!(figure.cost.unwrap().micro_usd, 3_000_000);
}

#[test]
fn an_unstamped_ball_falls_back_to_the_workspace_and_says_so() {
    let dir = tempfile::tempdir().unwrap();
    write_step(dir.path(), ROOT, "001", 1_000_000, 0, Some("opus"));
    write_step(dir.path(), OTHER, "001", 2_000_000, 0, Some("opus"));

    let figure = of_ball(&walk(dir.path()), &[], &table());
    assert_eq!(figure.attribution, Attribution::Workspace);
    assert_eq!(figure.tokens.input_tokens, 3_000_000);
    let note = figure.attribution.note().unwrap();
    assert_eq!(note.label, "workspace-wide");
    assert!(note.hover.contains("upper bound on the ball"), "{note:?}");
}

#[test]
fn attribution_notes_only_what_the_seat_does_not_already_claim() {
    assert!(Attribution::Conversations(1).note().is_none());
    let note = Attribution::Conversations(3).note().unwrap();
    assert_eq!(note.label, "over 3 conversations");
    assert!(note.hover.contains('3'));
}

/// The bl-9dd4 selection rule, both directions: a root takes its own tree and
/// its descent and nothing else, an empty root list is the whole workspace,
/// and a root listed twice still bills its steps once.
#[test]
fn selection_takes_each_bill_at_most_once_and_empty_means_everything() {
    let dir = tempfile::tempdir().unwrap();
    write_step(dir.path(), ROOT, "001", 10, 0, Some("opus"));
    write_step(dir.path(), KID, "001", 20, 0, Some("opus"));
    write_step(dir.path(), OTHER, "001", 40, 0, Some("opus"));
    let bills = walk(dir.path());
    assert_eq!(bills.len(), 3);

    let once = select(&bills, &[ROOT.to_owned()]);
    assert_eq!(
        crate::budgets::total(&once).input_tokens,
        30,
        "root + descent"
    );
    let twice = select(&bills, &[ROOT.to_owned(), ROOT.to_owned()]);
    assert_eq!(
        crate::budgets::total(&twice).input_tokens,
        30,
        "a repeated root does not double-bill"
    );
    assert_eq!(
        crate::budgets::total(&select(&bills, &[])).input_tokens,
        70,
        "no roots is the whole workspace"
    );
}

/// Every bill knows which conversation billed it — the fact that lets the walk
/// and the scope be two separate moments.
#[test]
fn a_bill_carries_its_conversation() {
    let dir = tempfile::tempdir().unwrap();
    write_step(dir.path(), ROOT, "001", 1, 0, Some("opus"));
    let bills = walk(dir.path());
    assert_eq!(bills.first().map(|b| b.conv.as_str()), Some(ROOT));
}
