//! Branch list, tree listing, file reads, and the `for-each-ref` line parser
//! (§9.3 / §5.1 #18). Real-git fixtures; parser error arms driven directly.

use super::super::{config_branches, config_file, config_tree, parse_branches};
use crate::git_tree::GitTreeError;
use crate::git_tree::tests::fixture::Fixture;

#[test]
fn config_branches_lists_default_with_tip_metadata() {
    let fx = Fixture::new();
    let branches = config_branches(&fx.path).unwrap();
    assert_eq!(branches.len(), 1);
    let d = &branches[0];
    assert_eq!(d.name, "default");
    assert_eq!(d.tip_oid.len(), 40);
    assert_eq!(Some(d.tip_short_oid.as_str()), d.tip_oid.get(..8));
    assert!(d.tip_timestamp_unix > 0);
}

#[test]
fn config_branches_are_listed_in_ref_name_order() {
    let fx = Fixture::new();
    fx.config_off("strict", "config/default");
    fx.orphan_config("island");
    let names: Vec<String> = config_branches(&fx.path)
        .unwrap()
        .into_iter()
        .map(|b| b.name)
        .collect();
    assert_eq!(names, vec!["default", "island", "strict"]);
}

#[test]
fn config_tree_lists_the_files_in_a_commit_tree() {
    let fx = Fixture::new();
    fx.commit_other("workflow.yaml", "events: {}\n");
    let tip = config_branches(&fx.path).unwrap()[0].tip_oid.clone();
    let files = config_tree(&fx.path, &tip).unwrap();
    assert!(files.contains(&"version".to_string()));
    assert!(files.contains(&"workflow.yaml".to_string()));
}

#[test]
fn config_file_reads_raw_bytes_and_errors_on_a_missing_path() {
    let fx = Fixture::new();
    // Fixture seeds `version` = "1\n" on the first config commit.
    let raw = config_file(&fx.path, "config/default", "version").unwrap();
    assert_eq!(raw, b"1\n");
    let err = config_file(&fx.path, "config/default", "no-such-file").unwrap_err();
    assert!(matches!(err, GitTreeError::Git { .. }), "{err}");
}

#[test]
fn parse_branches_rejects_a_line_missing_fields() {
    let err = parse_branches(b"config/default\n").unwrap_err();
    assert!(matches!(err, GitTreeError::LogFormat(_)), "{err}");
}

#[test]
fn parse_branches_rejects_a_non_numeric_timestamp() {
    let err = parse_branches(b"config/default deadbeef not-a-number\n").unwrap_err();
    assert!(matches!(err, GitTreeError::LogFormat(_)), "{err}");
}
