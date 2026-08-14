//! §3.7 item 5: the shipped default, the committed override, and the opt-out.

use super::*;
use tempfile::tempdir;

#[test]
fn the_shipped_default_is_this_suites_own_convention_and_only_it() {
    assert_eq!(DEFAULT, ["AGENTS.md"]);
}

#[test]
fn a_workspace_with_no_override_reads_the_default() {
    let dir = tempdir().unwrap();
    let ws = dir.path().join("ws");
    std::fs::create_dir_all(ws.join("repo.git")).unwrap();
    assert_eq!(names(&ws), vec!["AGENTS.md".to_owned()]);
    // A config commit that simply carries no such file is the same answer.
    let seeded = dir.path().join("seeded");
    crate::test_support::workspace::seed_workspace_workflow(&seeded, "events: {}\n");
    assert_eq!(names(&seeded), vec!["AGENTS.md".to_owned()]);
}

#[test]
fn a_committed_override_replaces_the_default_in_its_own_order() {
    let dir = tempdir().unwrap();
    let ws = dir.path().join("ws");
    crate::test_support::workspace::seed_workspace_config(
        &ws,
        &[(INSTRUCTIONS_YAML, "- HOUSE.md\n- AGENTS.md\n")],
    );
    assert_eq!(names(&ws), vec!["HOUSE.md", "AGENTS.md"]);
}

#[test]
fn an_override_naming_nothing_is_the_explicit_opt_out() {
    let dir = tempdir().unwrap();
    let ws = dir.path().join("ws");
    crate::test_support::workspace::seed_workspace_config(
        &ws,
        &[(INSTRUCTIONS_YAML, "# we pin nothing here\n")],
    );
    assert!(
        names(&ws).is_empty(),
        "an existing file is authoritative including when it names nothing"
    );
}

#[test]
fn reading_is_total_so_a_line_that_is_not_an_item_is_not_a_name() {
    let text = "---\nnames:\n- AGENTS.md\n  - nested.md\n-\n- \n- ../escape.md\n\
                - docs/RULES.md\n- a\\b.md\n- .\n- ..\n# comment\nplain\n";
    assert_eq!(parse(text), vec!["AGENTS.md", "nested.md"]);
}
