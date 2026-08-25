//! The §9.4 anchored block grammar over `providers.yaml`'s **`roles:` block**:
//! what it recognizes, what it rewrites byte-for-byte around, and — as loudly
//! — what it refuses.
//!
//! The other block the same grammar reads, `models.yaml`'s `models:` table, is
//! [`super::grammar_models`] — the corpus split on the seam the production
//! module already has (`grammar/roles.rs` beside `grammar/models.rs`).

use super::TEMPLATE_PROVIDERS;
use crate::model_pick::grammar::{GrammarError, PROVIDERS_YAML, RoleModel, roles, set_role_model};

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
