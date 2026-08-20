//! Authoring the control onto a workspace's `config/default`. The *drive* that
//! commits the drift is `start::ensure`'s single convergence (§3.7 item 4), and
//! is tested there; this file owns the fixed point and the drift.

use super::*;
use std::path::PathBuf;
use tempfile::tempdir;

/// lernie's shipped workflow as `0.0.11` seeds it: no ceiling, and a second
/// top-level block that must survive every transform here.
const SHIPPED: &str = "events:\n  user_message:\n    - dispatch(worker)\n\n\
     compaction:\n  intermediate:\n    trigger: on_flush\n";

/// The same file as a workspace born before lernie retired the seed carries
/// it — the stale whole-tree ceiling this convergence exists to remove.
const SEEDED: &str = "events:\n  user_message:\n    - dispatch(worker)\n\n\
     budgets:\n  max_total_tokens: 2000000\n  max_wall_seconds: 3600\n  max_depth: 4\n\n\
     compaction:\n  intermediate:\n    trigger: on_flush\n";

fn shim() -> PathBuf {
    PathBuf::from("/data/yog/world/tools/tool-control")
}

#[test]
fn authoring_appends_the_block_and_keeps_every_other_default() {
    let out = authored(SHIPPED, &shim());
    assert!(out.contains("events:"), "{out}");
    assert!(out.contains("trigger: on_flush"), "{out}");
    assert!(
        out.contains("tool_control:\n  command: /data/yog/world/tools/tool-control\n"),
        "{out}"
    );
}

/// The whole point of bl-56af: the seeded whole-tree ceiling is gone, every
/// axis of it, and its absence is stated rather than merely true.
#[test]
fn the_seeded_whole_tree_ceiling_is_removed_and_its_absence_is_stated() {
    let out = authored(SEEDED, &shim());
    assert!(!out.contains("budgets:"), "{out}");
    assert!(!out.contains("max_total_tokens"), "{out}");
    assert!(!out.contains("max_wall_seconds"), "{out}");
    assert!(!out.contains("max_depth"), "{out}");
    assert!(out.contains(BUDGETS_MARK), "{out}");
    // Everything around it survives — this is a subtraction, not a rewrite.
    assert!(out.contains("events:"), "{out}");
    assert!(out.contains("trigger: on_flush"), "{out}");
}

/// A ceiling an operator re-adds by hand is removed again on the next start:
/// yog holds this block empty the way it holds `tool_control:` its own, and a
/// half-honored strip would be a cap that is invisible again.
#[test]
fn a_hand_added_ceiling_does_not_survive_the_next_convergence() {
    let hand = format!(
        "{}budgets:\n  max_wall_seconds: 60\n",
        authored(SEEDED, &shim())
    );
    let out = authored(&hand, &shim());
    assert!(!out.contains("max_wall_seconds"), "{out}");
    assert_eq!(out, authored(SEEDED, &shim()));
}

/// A workspace born after lernie retired the seed has no block to strip, so
/// the strip computes to itself — the general path with empty input.
#[test]
fn a_workflow_with_no_ceiling_strips_to_itself() {
    let out = authored(SHIPPED, &shim());
    assert!(!out.contains("budgets:"), "{out}");
    assert_eq!(authored(&out, &shim()), out);
}

#[test]
fn authoring_is_a_fixed_point_which_is_the_whole_convergence_test() {
    let once = authored(SHIPPED, &shim());
    assert_eq!(authored(&once, &shim()), once);
}

#[test]
fn a_block_naming_another_shim_is_replaced_not_duplicated() {
    let stale = authored(SHIPPED, Path::new("/old/yog/tools/tool-control"));
    let fresh = authored(&stale, &shim());
    assert_eq!(fresh.matches("tool_control:").count(), 1, "{fresh}");
    assert!(!fresh.contains("/old/yog"), "{fresh}");
    assert!(fresh.contains("trigger: on_flush"), "{fresh}");
    assert_eq!(fresh.matches(BUDGETS_MARK).count(), 1, "{fresh}");
}

#[test]
fn a_top_level_key_after_the_block_survives_its_removal() {
    let base = "tool_control:\n  command: /old\n  extra: x\nretry:\n  max_attempts: 3\n";
    let out = authored(base, &shim());
    assert!(out.contains("max_attempts: 3"), "{out}");
    assert!(!out.contains("/old"), "{out}");
    assert!(!out.contains("extra: x"), "{out}");
}

#[test]
fn a_workspace_with_no_config_commit_drifts_nothing() {
    let dir = tempdir().unwrap();
    let ws = dir.path().join("ws");
    std::fs::create_dir_all(ws.join("repo.git")).unwrap();
    assert_eq!(committed(&ws, WORKFLOW_YAML), None);
    assert!(
        workflow_drift(&ws, &shim()).is_none(),
        "nothing to author onto is not an error"
    );
}

#[test]
fn a_tip_lacking_the_block_drifts_the_whole_workflow() {
    let dir = tempdir().unwrap();
    let ws = dir.path().join("ws");
    crate::test_support::workspace::seed_workspace_workflow(&ws, SHIPPED);
    assert_eq!(committed(&ws, WORKFLOW_YAML).as_deref(), Some(SHIPPED));
    let draft = workflow_drift(&ws, &shim()).expect("a tip without the block drifts");
    assert_eq!(draft.rel_path, WORKFLOW_YAML);
    // The drafted file is the WHOLE workflow: a fragment would truncate policy.
    let text = String::from_utf8(draft.bytes).unwrap();
    assert!(text.contains("events:"), "{text}");
    assert!(
        text.contains("command: /data/yog/world/tools/tool-control"),
        "{text}"
    );
}

#[test]
fn a_tip_that_already_names_this_shim_drifts_nothing() {
    let dir = tempdir().unwrap();
    let ws = dir.path().join("ws");
    crate::test_support::workspace::seed_workspace_workflow(&ws, &authored(SHIPPED, &shim()));
    assert!(workflow_drift(&ws, &shim()).is_none());
}
