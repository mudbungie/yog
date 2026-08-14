//! Authoring the control onto a workspace's `config/default`. The *drive* that
//! commits the drift is `start::ensure`'s single convergence (§3.7 item 4), and
//! is tested there; this file owns the fixed point and the drift.

use super::*;
use std::path::PathBuf;
use tempfile::tempdir;

const SHIPPED: &str =
    "events:\n  user_message:\n    - dispatch(worker)\n\nbudgets:\n  max_depth: 4\n";

fn shim() -> PathBuf {
    PathBuf::from("/data/yog/world/tools/tool-control")
}

#[test]
fn authoring_appends_the_block_and_keeps_every_other_default() {
    let out = authored(SHIPPED, &shim());
    assert!(out.contains("events:"), "{out}");
    assert!(out.contains("max_depth: 4"), "{out}");
    assert!(
        out.contains("tool_control:\n  command: /data/yog/world/tools/tool-control\n"),
        "{out}"
    );
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
    assert!(fresh.contains("max_depth: 4"), "{fresh}");
}

#[test]
fn a_top_level_key_after_the_block_survives_its_removal() {
    let base = "tool_control:\n  command: /old\n  extra: x\nbudgets:\n  max_depth: 4\n";
    let out = authored(base, &shim());
    assert!(out.contains("max_depth: 4"), "{out}");
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
