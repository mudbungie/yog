//! The §5.1 #35 derivation: the latest step of the root, its prompt reading,
//! and every way the figure declines to be fabricated.

use super::{Fullness, of_conversation, prompt_tokens};
use crate::budgets::{BudgetSpend, StepBill};
use std::collections::BTreeMap;

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

fn bill(conv: &str, seq: &str, model: Option<&str>, last: BudgetSpend) -> StepBill {
    StepBill {
        conv: conv.to_owned(),
        seq: seq.to_owned(),
        model: model.map(str::to_owned),
        spend: usage(999, 999, 999),
        last_usage: last,
        wall_secs: 0,
    }
}

fn windows() -> BTreeMap<String, u64> {
    BTreeMap::from([("sonnet".to_owned(), 200_000)])
}

/// The two provider readings, at the one place the rule lives: a disjoint
/// slicing (Anthropic) reads as its cached prefix, a contained one (OpenAI —
/// `cached_tokens` inside `prompt_tokens`) reads as the prompt itself, and a
/// provider reporting no cache counters at all degrades to `input_tokens`.
#[test]
fn the_prompt_reading_never_over_states_either_provider_shape() {
    assert_eq!(prompt_tokens(usage(4_000, 90_000, 6_000)), 96_000);
    assert_eq!(prompt_tokens(usage(120_000, 100_000, 0)), 120_000);
    assert_eq!(prompt_tokens(usage(30_000, 0, 0)), 30_000);
    assert_eq!(prompt_tokens(BudgetSpend::default()), 0);
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
    let full = of_conversation(&bills, ROOT, &windows()).expect("a measured context");
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
    let full = of_conversation(&bills, ROOT, &windows()).expect("a measured context");
    assert_eq!(full.prompt_tokens, 100_000);
    assert_eq!(full.percent(), 50);
}

/// Three silences, each rendering nothing rather than a guess: no step, a step
/// whose `request.json` named no model, and a model nothing declared a window
/// for.
#[test]
fn nothing_honest_to_say_is_no_figure_at_all() {
    assert_eq!(of_conversation(&[], ROOT, &windows()), None);
    assert_eq!(
        of_conversation(
            &[bill(ROOT, "001", None, usage(10, 0, 0))],
            ROOT,
            &windows()
        ),
        None
    );
    assert_eq!(
        of_conversation(
            &[bill(ROOT, "001", Some("opus"), usage(10, 0, 0))],
            ROOT,
            &windows()
        ),
        None
    );
}

/// An overflowing context reads as itself. A clamp would render "the
/// declaration is stale" and "the context is exactly full" as the same 100%.
#[test]
fn a_context_past_its_declared_window_is_not_clamped() {
    let bills = [bill(ROOT, "001", Some("sonnet"), usage(280_000, 0, 0))];
    let full = of_conversation(&bills, ROOT, &windows()).expect("a measured context");
    assert_eq!(full.percent(), 140);
}
