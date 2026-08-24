//! §3.7 item 4: the glob that makes a frozen document compose, and its fixed
//! point. `SHIPPED` is lernie 0.0.8's `template/manifest.yaml`, verbatim in
//! shape — pinning only `goal.md`, `soul.md`, `descriptions/**`, which is
//! exactly why a pin at `instructions/…` reaches no model without this.

use super::*;
use tempfile::tempdir;

const SHIPPED: &str = "\
roles:
  worker:
    pinned:
      - goal.md
      - soul.md
      - descriptions/**
    order:
      - summary/**
      - skills/**
    budget_tokens: 150000
    overflow: drop_oldest_summaries
  compactor:
    pinned:
      - goal.md
      - soul.md
    # A comment inside the role's block.
    order:
      - summary/**
    budget_tokens: 50000
    overflow: truncate
";

#[test]
fn the_glob_joins_the_workers_pinned_list_and_no_other_role() {
    let out = authored(SHIPPED);
    assert!(
        out.contains("      - descriptions/**\n      - instructions/**\n"),
        "{out}"
    );
    assert_eq!(out.matches(GLOB).count(), 1, "the compactor is untouched");
    assert!(out.contains("budget_tokens: 150000"), "{out}");
    assert!(out.contains("overflow: truncate"), "{out}");
    assert!(
        out.contains("# A comment inside the role's block."),
        "{out}"
    );
}

#[test]
fn authoring_is_a_fixed_point_which_is_the_whole_convergence_test() {
    let once = authored(SHIPPED);
    assert_eq!(authored(&once), once);
}

#[test]
fn the_item_indent_is_taken_from_the_list_it_joins() {
    let base = "roles:\n  worker:\n    pinned:\n    - goal.md\n    order: []\n";
    assert!(authored(base).contains("\n    - goal.md\n    - instructions/**\n"));
}

#[test]
fn an_empty_pinned_list_still_takes_the_glob_one_step_past_its_key() {
    let base = "roles:\n  worker:\n    pinned:\n    order: []\n";
    assert!(
        authored(base).contains("    pinned:\n      - instructions/**\n    order: []"),
        "{}",
        authored(base)
    );
}

#[test]
fn a_manifest_with_no_anchor_is_left_alone() {
    for base in [
        "roles:\n  compactor:\n    pinned:\n      - goal.md\n",
        "roles:\n  worker:\n    order: []\n",
        "budget_tokens: 1\n",
        "roles:\n",
        "roles:\n  worker:\n",
    ] {
        assert_eq!(authored(base), base, "an operator's own manifest: {base:?}");
    }
}

#[test]
fn a_workspace_with_no_committed_manifest_drifts_nothing() {
    let dir = tempdir().unwrap();
    let ws = dir.path().join("ws");
    crate::test_support::workspace::seed_workspace_workflow(&ws, "events: {}\n");
    assert!(
        drift(&ws, "default").is_none(),
        "nothing to author onto is not an error"
    );
}

#[test]
fn a_committed_manifest_without_the_glob_drifts_the_whole_file() {
    let dir = tempdir().unwrap();
    let ws = dir.path().join("ws");
    crate::test_support::workspace::seed_workspace_config(&ws, &[(MANIFEST_YAML, SHIPPED)]);
    let draft = drift(&ws, "default").expect("the shipped manifest composes no instructions");
    assert_eq!(draft.rel_path, MANIFEST_YAML);
    let text = String::from_utf8(draft.bytes).unwrap();
    assert_eq!(text, authored(SHIPPED));
    assert!(text.contains("overflow: drop_oldest_summaries"), "{text}");
}

#[test]
fn a_manifest_that_already_composes_instructions_drifts_nothing() {
    let dir = tempdir().unwrap();
    let ws = dir.path().join("ws");
    crate::test_support::workspace::seed_workspace_config(
        &ws,
        &[(MANIFEST_YAML, &authored(SHIPPED))],
    );
    assert!(drift(&ws, "default").is_none());
}
