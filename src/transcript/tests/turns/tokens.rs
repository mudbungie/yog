//! The aggregate's token terms (§11, bl-8b3c): sums of the counted entries'
//! committed `usage` records, stated verbatim — never estimated. Three
//! directions: usage present → tokens stated; absent → counts only (that
//! direction is [`super`]'s fixtures, whose entries commit no usage and whose
//! aggregate line carries no token term); mixed → the `+` wording.

use super::{call, delivered, model, model_with_usage, prefixes, result};
use crate::transcript::tests::render::painted_with;
use crate::transcript::tests::rows::{default_rows, tx};
use crate::transcript::{AutoExpand, Block, Transcript};
use std::collections::HashSet;

/// [`super`]'s finished turn, with every machinery entry usage-bearing. The
/// answer entry stays legacy on purpose: the aggregate covers exactly the
/// entries its inference count covers — the machinery run, not the answer.
fn tokened_turn(second: &[(&str, u64)]) -> Transcript {
    tx(vec![
        delivered("001-user.md", "do the thing"),
        model_with_usage(
            "002-opus.json",
            vec![Block::Thinking("weighing it".into()), call("t1", "Read")],
            &[("input_tokens", 100), ("output_tokens", 2000)],
        ),
        result("003-tool.json", "t1"),
        model_with_usage("004-opus.json", vec![call("t2", "Bash")], second),
        result("005-tool.json", "t2"),
        model("006-opus.json", vec![Block::Text("done".into())]),
    ])
}

#[test]
fn a_usage_bearing_turn_states_each_committed_counter_sum() {
    // Sums ride under the counters' own committed names; a counter that sums
    // to zero goes unsaid like any other zero term.
    let t = tokened_turn(&[
        ("input_tokens", 40),
        ("output_tokens", 1150),
        ("cache_read_tokens", 0),
    ]);
    assert_eq!(
        prefixes(&default_rows(&t))[1],
        "⚙ 2 inference calls · 2 tool calls · 1 thinking block \
         · 140 input tokens · 3150 output tokens",
        "the bytes' own counters, summed and said"
    );
}

#[test]
fn a_mixed_turn_says_at_least_with_the_plus_suffix() {
    // One entry reported, one is legacy: the sum covers only what the bytes
    // carry, so each token term wears `+` — at least this many.
    let t = tokened_turn(&[]);
    assert_eq!(
        prefixes(&default_rows(&t))[1],
        "⚙ 2 inference calls · 2 tool calls · 1 thinking block \
         · 100+ input tokens · 2000+ output tokens",
        "a partial sum never poses as the total"
    );
}

#[test]
fn the_token_terms_paint_on_the_aggregate_line() {
    let t = tokened_turn(&[("output_tokens", 1150)]);
    let painted = painted_with(&t, false, AutoExpand::default(), &mut HashSet::new());
    assert!(
        painted.contains("3150 output tokens"),
        "the sum reaches the screen:\n{painted}"
    );
}
