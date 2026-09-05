//! The §5.1 #35 derivation: the latest step of the root, its prompt reading,
//! and every way the figure declines to be fabricated.

use super::{Fullness, of_agent};
use crate::budgets::{BudgetSpend, StepBill};

const ROOT: &str = "20260802T120000Z-root";
const KID: &str = "20260802T120000Z-root-20260802T120100Z-kid0";

fn usage(input: u64, read: u64, write: u64) -> BudgetSpend {
    BudgetSpend {
        input_tokens: input,
        output_tokens: 7,
        cache_read_tokens: read,
        cache_write_tokens: write,
    }
}

/// A step on a row whose usage lines stated a 200 000-token window.
fn bill(conv: &str, seq: &str, model: Option<&str>, last: BudgetSpend) -> StepBill {
    StepBill {
        conv: conv.to_owned(),
        seq: seq.to_owned(),
        model: model.map(str::to_owned),
        spend: usage(999, 999, 999),
        last_usage: last,
        window: Some(200_000),
        wall_secs: 0,
    }
}

/// The prompt reading itself is pinned where the formula lives
/// (`budgets::BudgetSpend::prompt_tokens`); what this module owes is that the
/// fullness figure asks for THAT reading of the right step. A contained cached
/// slice is the case that used to divide the two: a step reading `input 2_000,
/// cache_read 48_000` is a 48,000-token context, not a 50,000-token one.
#[test]
fn fullness_reads_the_folded_prompt_not_the_bare_input_counter() {
    let bills = [bill(ROOT, "001", Some("sonnet"), usage(2_000, 48_000, 0))];
    let full = of_agent(&bills, ROOT).expect("a measured context");
    assert_eq!(full.prompt_tokens, 48_000);
}

/// The figure is the LATEST step of the ROOT: an earlier step of the same
/// conversation is history, and a dispatched child runs its own context.
#[test]
fn the_root_s_latest_step_is_the_context() {
    let bills = [
        bill(ROOT, "001", Some("sonnet"), usage(10_000, 0, 0)),
        bill(ROOT, "002", Some("sonnet"), usage(2_000, 48_000, 0)),
        bill(KID, "009", Some("sonnet"), usage(190_000, 0, 0)),
    ];
    let full = of_agent(&bills, ROOT).expect("a measured context");
    assert_eq!(
        full,
        Fullness {
            model: "sonnet".to_owned(),
            prompt_tokens: 48_000,
            window: 200_000,
        }
    );
    assert_eq!(full.percent(), 24);
}

/// Step order is the zero-padded dir name's own order, so `010` follows `009`
/// — the property the fixed width exists for.
#[test]
fn step_order_is_lexical_because_the_width_is_fixed() {
    let bills = [
        bill(ROOT, "009", Some("sonnet"), usage(9, 0, 0)),
        bill(ROOT, "010", Some("sonnet"), usage(100_000, 0, 0)),
    ];
    let full = of_agent(&bills, ROOT).expect("a measured context");
    assert_eq!(full.prompt_tokens, 100_000);
    assert_eq!(full.percent(), 50);
}

/// Three silences, each rendering nothing rather than a guess: no step, a step
/// whose `request.json` named no model, and a step whose usage lines stated no
/// window — the row served none, and yog holds no table to fill it from.
#[test]
fn nothing_honest_to_say_is_no_figure_at_all() {
    assert_eq!(of_agent(&[], ROOT), None);
    assert_eq!(
        of_agent(&[bill(ROOT, "001", None, usage(10, 0, 0))], ROOT),
        None
    );
    let unstated = StepBill {
        window: None,
        ..bill(ROOT, "001", Some("opus"), usage(10, 0, 0))
    };
    assert_eq!(of_agent(&[unstated], ROOT), None);
}

/// An overflowing context reads as itself. A clamp would render "the row's
/// window is wrong" and "the context is exactly full" as the same 100%.
#[test]
fn a_context_past_its_declared_window_is_not_clamped() {
    let bills = [bill(ROOT, "001", Some("sonnet"), usage(280_000, 0, 0))];
    let full = of_agent(&bills, ROOT).expect("a measured context");
    assert_eq!(full.percent(), 140);
}
