//! Reading `models.yaml` back, and judging what it says (bl-53be): which rows
//! the file declares, which of them brazen does not have, and the one number
//! anything reads out of the table (§5.1 #35's denominator).
//!
//! The role-row sentence moved out with bl-d9cb: it is over `providers.yaml`'s
//! live pointer now, and lives in `plan.rs` beside the gate it shares a wording
//! with.

use super::SEEDED_MODELS;
use crate::model_pick::grammar::{DeclaredModel, context_windows, declared, unknown_rows};
use std::collections::BTreeMap;

/// The live file as the operator's world carried it: two Claude entries on a
/// row brazen has no listing for, one working codex entry.
const LIVE_MODELS: &str = "# header\n\nmodels:\n  claude-sonnet-5:\n    provider: anthropic\n    \
     model_id: claude-sonnet-5\n    context_window: 1000000\n  claude-haiku-4-5:\n    \
     provider: anthropic\n    model_id: claude-haiku-4-5\n  gpt-5.4:\n    provider: codex\n    \
     model_id: gpt-5.4\n";

/// brazen's effective table on that box, `bz --list-providers` order.
fn rows() -> Vec<String> {
    ["codex", "local", "claude-code", "claude-session-direct"]
        .iter()
        .map(|s| (*s).to_string())
        .collect()
}

fn declared_model(model: &str, provider: &str) -> DeclaredModel {
    DeclaredModel {
        model: model.to_string(),
        provider: provider.to_string(),
    }
}

#[test]
fn reads_every_entry_the_seeded_file_declares() {
    assert_eq!(
        declared(SEEDED_MODELS),
        vec![declared_model("gpt-5.4", "codex")]
    );
    assert_eq!(
        declared(LIVE_MODELS),
        vec![
            declared_model("claude-sonnet-5", "anthropic"),
            declared_model("claude-haiku-4-5", "anthropic"),
            declared_model("gpt-5.4", "codex"),
        ]
    );
}

/// The shapes the grammar declines, and the entry it simply omits: an absent,
/// inline or flow-styled `models:` declares nothing, and an entry with no
/// `provider:` line names no row, so it can name no wrong one.
#[test]
fn unrecognized_shapes_and_provider_less_entries_declare_nothing() {
    assert!(declared("").is_empty());
    assert!(declared("models: {}\n").is_empty());
    assert!(declared("modelsx:\n  m:\n    provider: p\n").is_empty());
    assert!(declared("models:\n  m: { provider: p }\n").is_empty());
    assert_eq!(
        declared("models:\n  half:\n    model_id: half\n  whole:\n    provider: codex\n"),
        vec![declared_model("whole", "codex")]
    );
}

/// The defect this ball was filed for: `provider: anthropic` against a table
/// that has no such row. Both entries are named, the live one is not.
#[test]
fn unknown_rows_names_every_entry_brazen_cannot_route() {
    assert_eq!(
        unknown_rows(LIVE_MODELS, &rows()),
        vec![
            declared_model("claude-sonnet-5", "anthropic"),
            declared_model("claude-haiku-4-5", "anthropic"),
        ]
    );
    // Re-pointed at a row that exists, the file is clean.
    let fixed = LIVE_MODELS.replace("anthropic", "claude-session-direct");
    assert!(unknown_rows(&fixed, &rows()).is_empty());
    // And a file with no `models:` block has nothing to judge — the reason the
    // gate can run over every §9.2 file without asking which one it holds.
    assert!(unknown_rows("steps:\n  - run: x\n", &rows()).is_empty());
}

/// An EMPTY table is no answer, not an empty answer: brazen could not be asked,
/// so nothing is refused. The alternative would brick every §9.2 Apply on the
/// strength of a question that went unanswered.
#[test]
fn an_empty_provider_table_gates_nothing() {
    assert!(unknown_rows(LIVE_MODELS, &[]).is_empty());
}

/// The rendered pair a rejection prints, so the Apply status line and this
/// projection can never drift.
#[test]
fn a_declared_model_renders_as_model_arrow_row() {
    assert_eq!(
        declared_model("claude-sonnet-5", "anthropic").to_string(),
        "claude-sonnet-5 → anthropic"
    );
}

/// The §5.1 #35 denominator, read off the same file by the same grammar: the
/// window an entry declares, keyed on the **wire id** a step's `request.json`
/// names rather than on the alias the entry is filed under. Since bl-d9cb this
/// is the ONLY number anything reads out of the `models:` table — lernie reads
/// none of it, and the picker writes none of it.
#[test]
fn reads_the_context_window_each_entry_declares_keyed_on_the_wire_id() {
    assert_eq!(
        context_windows(SEEDED_MODELS),
        BTreeMap::from([("gpt-5.4".to_owned(), 400_000)])
    );
    // Two of the live file's three entries declare no window at all — and an
    // undeclared window is absent, never a default, so no figure is rendered
    // against a number nobody wrote.
    assert_eq!(
        context_windows(LIVE_MODELS),
        BTreeMap::from([("claude-sonnet-5".to_owned(), 1_000_000)])
    );
}

/// An alias entry keys the map on its `model_id`; a window that is zero, not a
/// number, or on a file with no `models:` block at all declares nothing.
#[test]
fn an_undeclarable_window_is_absent_rather_than_guessed() {
    let aliased = "models:\n  sonnet:\n    provider: anthropic\n    \
         model_id: claude-sonnet-5\n    context_window: 200000\n";
    assert_eq!(
        context_windows(aliased),
        BTreeMap::from([("claude-sonnet-5".to_owned(), 200_000)])
    );
    let junk = "models:\n  a:\n    context_window: 0\n  b:\n    context_window: lots\n  \
         c:\n    provider: codex\n";
    assert!(context_windows(junk).is_empty());
    assert!(context_windows("roles:\n  worker:\n    provider: codex\n").is_empty());
}
