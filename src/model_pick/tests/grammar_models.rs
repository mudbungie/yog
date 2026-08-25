//! The §9.4 anchored block grammar over `models.yaml`'s **`models:` table**:
//! the declaration one pick writes — where the entry lands, what it carries
//! and what it leaves alone — and the context window read back out of it.
//!
//! The `roles:` half of the same grammar is [`super::grammar_roles`].

use super::SEEDED_MODELS;
use crate::model_pick::grammar::{
    DEFAULT_CONTEXT_WINDOW, GrammarError, MODELS_YAML, context_windows, declare_model,
};

/// The new entry lands **directly after `models:`**, so a file carrying a later
/// top-level key stays valid (§9.4).
#[test]
fn declare_model_inserts_after_the_models_key_not_at_eof() {
    let text = format!("adapter: /usr/bin/bz\n{SEEDED_MODELS}");
    let out = declare_model(&text, "gpt-5.6-sol").unwrap().unwrap();
    let (head, _) = out.split_once("  gpt-5.6-sol:").unwrap();
    assert!(head.ends_with("models:\n") || head.contains("models:\n  #"));
    assert!(out.contains("adapter: /usr/bin/bz"));
    // The operator's own entry is untouched — its context window is theirs, and
    // so are the fields yog no longer writes.
    assert!(out.contains("    context_window: 400000"));
    assert!(out.contains("    capabilities: [tool_use_native, streaming]"));
}

/// bl-3ffa. The generated entry is **the id and the window**, under a comment
/// saying the number is a declared default and naming the one thing that reads
/// it — the operator's whole reason to edit it. The two fields nothing consumed
/// are not written: no `provider:` (its only reader was the §9.2 gate that
/// judged it) and no `capabilities:` (no reader in either program). No
/// `model_id:` either — the entry key is the wire id [`context_windows`] falls
/// back to, so writing it twice would be two spellings of one fact.
#[test]
fn declare_model_writes_the_id_and_the_one_fact_read_out_of_it() {
    let out = declare_model(SEEDED_MODELS, "gpt-5.6-sol")
        .unwrap()
        .unwrap();
    assert!(out.contains(&format!(
        "  gpt-5.6-sol:\n    context_window: {DEFAULT_CONTEXT_WINDOW}"
    )));
    assert!(out.contains("declared default"), "{out}");
    assert!(out.contains("context-fullness figure"), "{out}");
    // The window it declares is the one the reader answers with.
    assert_eq!(
        context_windows(&out).get("gpt-5.6-sol").copied(),
        Some(u64::from(DEFAULT_CONTEXT_WINDOW))
    );
    // Neither dead field is authored, and no seat claims a provider published
    // any of it. The entry is its own four-space lines and stops at the sibling
    // the fixture already carried — which does declare all three, untouched.
    let entry: Vec<&str> = out
        .split_once("  gpt-5.6-sol:\n")
        .unwrap()
        .1
        .lines()
        .take_while(|l| l.starts_with("    "))
        .collect();
    assert_eq!(
        entry,
        [format!("    context_window: {DEFAULT_CONTEXT_WINDOW}")]
    );
    assert!(!out.contains("served"), "{out}");
    assert!(!out.contains("list-models"), "{out}");
}

/// An id the file already declares is nothing to write, so the operator's own
/// entry stands whole — every field of it, including the two yog stopped
/// writing. bl-bd89's re-point arm went with the row it moved (bl-3ffa): a table
/// that names no provider row cannot declare an id "on another row".
#[test]
fn a_declaration_that_exists_is_nothing_to_write() {
    assert_eq!(declare_model(SEEDED_MODELS, "gpt-5.4"), Ok(None));
    // Not even a file whose entry carries no field yog would write.
    assert_eq!(
        declare_model("models:\n  m:\n    model_id: m\n", "m"),
        Ok(None)
    );
}

/// An absent (or key-less) models.yaml is the general path with empty input:
/// the block is created, not special-cased.
#[test]
fn declare_model_creates_the_block_when_there_is_none() {
    let out = declare_model("", "gpt-5.6-sol").unwrap().unwrap();
    assert!(out.starts_with("models:\n"));
    assert!(out.contains("  gpt-5.6-sol:"));
    let kept = declare_model("adapter: /usr/bin/bz\n", "m")
        .unwrap()
        .unwrap();
    assert!(kept.starts_with("adapter: /usr/bin/bz\nmodels:\n"));
}

#[test]
fn declare_model_declines_an_inline_models_key() {
    let err = declare_model("models: {}\n", "gpt-5.6-sol").unwrap_err();
    assert_eq!(
        err,
        GrammarError::Inline {
            file: MODELS_YAML,
            key: "models".into(),
        }
    );
    assert!(err.to_string().contains("models.yaml"));
}
