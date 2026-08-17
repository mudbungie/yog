//! The §9.4 anchored block grammar: what it recognizes, what it rewrites
//! byte-for-byte around, and — as loudly — what it refuses.

use super::{SEEDED_MODELS, TEMPLATE_PROVIDERS};
use crate::model_pick::grammar::{
    DEFAULT_CONTEXT_WINDOW, GrammarError, MODELS_YAML, PROVIDERS_YAML, RoleModel, context_windows,
    declare_model, roles, set_role_model,
};

#[test]
fn reads_every_role_lernies_template_declares() {
    assert_eq!(
        roles(TEMPLATE_PROVIDERS),
        vec![
            RoleModel {
                role: "worker".into(),
                provider: "codex".into(),
                model: "gpt-5.4".into(),
            },
            RoleModel {
                role: "compactor".into(),
                provider: "codex".into(),
                model: "gpt-5.4-mini".into(),
            },
        ]
    );
}

/// A `roles:` block that is absent, inline, or flow-styled yields no roles —
/// the picker then paints its "use the raw editor" refusal (§7.3).
#[test]
fn unrecognized_shapes_declare_no_roles() {
    assert!(roles("").is_empty());
    assert!(roles("version: 1\n").is_empty());
    assert!(roles("roles: {}\n").is_empty());
    // A key that merely *starts* with the block name is not the block name.
    assert!(roles("rolesx:\n  worker:\n    provider: p\n    model: m\n").is_empty());
    assert!(roles("roles:\n  worker: { provider: codex, model: gpt-5.4 }\n").is_empty());
    // An entry missing a half of its assignment is not an assignment — whether
    // the file simply ends, or the next entry begins.
    assert!(roles("roles:\n  worker:\n    provider: codex\n").is_empty());
    assert!(roles("roles:\n  worker:\n    model: gpt-5.4\n").is_empty());
    assert_eq!(
        roles("roles:\n  worker:\n    provider: p\n  compactor:\n    provider: q\n    model: m\n")
            .len(),
        1,
        "the worker's missing `model:` is not read off the compactor"
    );
}

/// Comments and blank lines inside a block do not end it — at either indent.
#[test]
fn comments_and_blanks_ride_inside_a_block() {
    let text = "roles:\n  # the role that talks to you\n  worker:\n\n    provider: codex\n  # a note mid-entry\n    model: gpt-5.4\n";
    assert_eq!(
        roles(text),
        vec![RoleModel {
            role: "worker".into(),
            provider: "codex".into(),
            model: "gpt-5.4".into(),
        }]
    );
}

/// A column-0 line ends the block: neither a later top-level key's own entries
/// nor its fields are mistaken for the block's (the `roles:` block in lernie's
/// template is last, but nothing guarantees it stays that way).
#[test]
fn a_later_top_level_key_ends_the_block() {
    let text = "roles:\n  worker:\n    provider: codex\n    model: gpt-5.4\nversion: 1\n";
    assert_eq!(roles(text).len(), 1);
    // The trailing key survives a rewrite, and did not become a role.
    let out = set_role_model(text, "worker", "codex", "gpt-5.6-sol").unwrap();
    assert!(out.ends_with("version: 1\n"));
    assert_eq!(roles(&out).len(), 1);
    // An entry whose `model:` line is past the block boundary is not its field.
    assert!(matches!(
        set_role_model(
            "roles:\n  worker:\n    provider: codex\nversion: 1\n    model: x\n",
            "worker",
            "codex",
            "y"
        ),
        Err(GrammarError::NoField { .. })
    ));
}

/// A rewrite moves exactly two lines and leaves every other byte — `tools:`,
/// the sibling role, the ordering — alone.
#[test]
fn set_role_model_moves_two_lines_and_nothing_else() {
    let out = set_role_model(TEMPLATE_PROVIDERS, "worker", "codex", "gpt-5.6-sol").unwrap();
    assert_eq!(
        roles(&out),
        vec![
            RoleModel {
                role: "worker".into(),
                provider: "codex".into(),
                model: "gpt-5.6-sol".into(),
            },
            RoleModel {
                role: "compactor".into(),
                provider: "codex".into(),
                model: "gpt-5.4-mini".into(),
            },
        ]
    );
    assert!(out.contains("    tools: [bash, read_file, load_skill]"));
    // The compactor is untouched: one click retargets one role (§9.4).
    assert!(out.contains("  compactor:\n    provider: codex\n    model: gpt-5.4-mini"));
}

#[test]
fn set_role_model_declines_an_absent_or_flow_role() {
    let err = set_role_model(TEMPLATE_PROVIDERS, "auditor", "codex", "x").unwrap_err();
    assert_eq!(
        err,
        GrammarError::NoEntry {
            file: PROVIDERS_YAML,
            entry: "auditor".into(),
        }
    );
    assert!(err.to_string().contains("raw editor"));
    assert!(matches!(
        set_role_model("version: 1\n", "worker", "codex", "x"),
        Err(GrammarError::NoEntry { .. })
    ));
}

#[test]
fn set_role_model_declines_an_inline_roles_key() {
    let err = set_role_model("roles: {}\n", "worker", "codex", "x").unwrap_err();
    assert_eq!(
        err,
        GrammarError::Inline {
            file: PROVIDERS_YAML,
            key: "roles".into(),
        }
    );
    assert!(err.to_string().contains("inline value"));
}

#[test]
fn set_role_model_declines_a_role_missing_the_field_it_must_move() {
    let err = set_role_model(
        "roles:\n  worker:\n    provider: codex\n",
        "worker",
        "codex",
        "x",
    )
    .unwrap_err();
    assert_eq!(
        err,
        GrammarError::NoField {
            file: PROVIDERS_YAML,
            entry: "worker".into(),
            field: "model",
        }
    );
    assert!(err.to_string().contains("model"));
}

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
