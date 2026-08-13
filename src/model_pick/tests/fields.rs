//! The grammar's generic field access — the prelude every rewrite shares, and
//! the plain read/replace the §9.5 pane is built on.

use super::{SEEDED_MODELS, TEMPLATE_PROVIDERS};
use crate::model_pick::grammar::{
    GrammarError, MODELS, MODELS_YAML, PROVIDERS_YAML, ROLES, entry_field, entry_names, set_field,
};

#[test]
fn entry_names_lists_the_block_in_file_order_and_absence_is_empty() {
    assert_eq!(
        entry_names(TEMPLATE_PROVIDERS, ROLES),
        ["worker", "compactor"]
    );
    assert_eq!(entry_names(SEEDED_MODELS, MODELS), ["gpt-5.4"]);
    // No such block, an inline one, and an empty file all declare nothing.
    assert!(entry_names(TEMPLATE_PROVIDERS, MODELS).is_empty());
    assert!(entry_names("roles: {}\n", ROLES).is_empty());
    assert!(entry_names("", ROLES).is_empty());
}

#[test]
fn entry_field_reads_one_value_and_folds_every_miss_to_none() {
    assert_eq!(
        entry_field(TEMPLATE_PROVIDERS, ROLES, "worker", "model").as_deref(),
        Some("gpt-5.4")
    );
    // No block, no entry, no field — three misses, one value.
    assert_eq!(
        entry_field(TEMPLATE_PROVIDERS, MODELS, "worker", "model"),
        None
    );
    assert_eq!(
        entry_field(TEMPLATE_PROVIDERS, ROLES, "nobody", "model"),
        None
    );
    assert_eq!(
        entry_field(TEMPLATE_PROVIDERS, ROLES, "compactor", "tools"),
        None
    );
}

#[test]
fn set_field_replaces_one_line_and_preserves_every_other_byte() {
    let out = set_field(
        PROVIDERS_YAML,
        TEMPLATE_PROVIDERS,
        ROLES,
        "compactor",
        "model",
        "haiku-9",
    )
    .unwrap();
    assert!(out.contains("    model: haiku-9\n"));
    assert!(out.contains("    tools: [bash, read_file, load_skill]\n"));
    assert!(out.contains("  worker:\n    provider: codex\n    model: gpt-5.4\n"));
}

/// One refusal from a `providers.yaml`-shaped rewrite of `entry.field`.
fn refusal(text: &str, entry: &str, field: &'static str) -> GrammarError {
    set_field(PROVIDERS_YAML, text, ROLES, entry, field, "x").unwrap_err()
}

#[test]
fn set_field_declines_loudly_on_every_shape_it_does_not_edit() {
    // An inline block key would need a YAML transform.
    assert!(
        matches!(refusal("roles: {}\n", "worker", "model"), GrammarError::Inline { key, .. } if key == ROLES)
    );
    // No block at all: there is no such entry to rewrite.
    assert!(
        matches!(refusal("models:\n  m:\n", "worker", "model"), GrammarError::NoEntry { entry, .. } if entry == "worker")
    );
    // The block is there; the entry is not.
    assert!(
        matches!(refusal(TEMPLATE_PROVIDERS, "nobody", "model"), GrammarError::NoEntry { entry, .. } if entry == "nobody")
    );
    // The entry is there; the field is not.
    assert!(
        matches!(refusal(TEMPLATE_PROVIDERS, "compactor", "tools"), GrammarError::NoField { field, .. } if field == "tools")
    );
}

#[test]
fn the_file_name_a_refusal_carries_is_the_callers() {
    let err = set_field(MODELS_YAML, "", MODELS, "m", "provider", "codex").unwrap_err();
    assert!(err.to_string().starts_with(MODELS_YAML));
}
