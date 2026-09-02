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
fn reads_every_role_litanys_template_declares() {
    assert_eq!(
        roles(TEMPLATE_PROVIDERS),
        vec![
            RoleModel {
                role: "worker".into(),
                provider: "codex".into(),
                model: "gpt-5.4".into(),
                effort: None,
                priority: false,
            },
            RoleModel {
                role: "compactor".into(),
                provider: "codex".into(),
                model: "gpt-5.4-mini".into(),
                effort: None,
                priority: false,
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
            effort: None,
            priority: false,
        }]
    );
}

/// A column-0 line ends the block: neither a later top-level key's own entries
/// nor its fields are mistaken for the block's (the `roles:` block in litany's
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
                effort: None,
                priority: false,
            },
            RoleModel {
                role: "compactor".into(),
                provider: "codex".into(),
                model: "gpt-5.4-mini".into(),
                effort: None,
                priority: false,
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

/// **The entry's two optional knobs are read with the pointer** (bl-2410), so
/// the workspace-assignments answer and the `/effort` / `/priority` writes are
/// one vocabulary over one line-block. A role carrying neither reads as neither.
#[test]
fn the_tuning_knobs_are_read_beside_the_pointer() {
    let tuned = "roles:\n  worker:\n    provider: codex\n    model: gpt-5.4\n    effort: high\n    \
                 priority: true\n  compactor:\n    provider: codex\n    model: gpt-5.4-mini\n";
    assert_eq!(
        roles(tuned),
        vec![
            RoleModel {
                role: "worker".into(),
                provider: "codex".into(),
                model: "gpt-5.4".into(),
                effort: Some("high".into()),
                priority: true,
            },
            RoleModel {
                role: "compactor".into(),
                provider: "codex".into(),
                model: "gpt-5.4-mini".into(),
                effort: None,
                priority: false,
            },
        ]
    );
}

/// **A level yog would never write is carried, not swallowed.** The §9.1 raw
/// editor is the operator's own authority, so the file can say anything; a read
/// that normalized it to absent would report *nothing is set*, which is the
/// defect bl-2410 exists to end. `priority` has no such case — anything but
/// `true` is *not asking*, which is the engine's own reading said back.
#[test]
fn an_unrecognized_level_survives_the_read_and_a_stray_priority_word_does_not() {
    let odd = "roles:\n  worker:\n    provider: codex\n    model: gpt-5.4\n    effort: extreme\n    \
               priority: yes\n";
    let read = roles(odd);
    assert_eq!(read[0].effort.as_deref(), Some("extreme"));
    assert!(!read[0].priority, "only `true` asks for the lane");
}

/// A role missing half its pointer is not an assignment and is dropped, while a
/// role missing a knob is an ordinary role — the required/optional split, at the
/// one place it is decided.
#[test]
fn a_half_declared_pointer_drops_the_role_but_a_missing_knob_does_not() {
    let half = "roles:\n  worker:\n    provider: codex\n  compactor:\n    provider: codex\n    \
                model: m\n";
    let read = roles(half);
    assert_eq!(read.len(), 1);
    assert_eq!(read[0].role, "compactor");
    assert_eq!(read[0].effort, None);
}
