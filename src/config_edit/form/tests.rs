//! The §9.5 typed-settings view-model: what each control shows, what it faults
//! on, and what it writes back. Every arm is pure text → text, so the whole
//! table is driven without egui, disk or brazen.

use super::{Control, Group, MODELS_SCHEMA, ROLES_SCHEMA, Row, read, schema_for, write};
use crate::model_pick::grammar::MODELS_YAML;
use crate::model_pick::tests::{SEEDED_MODELS, TEMPLATE_PROVIDERS};

fn rows() -> Vec<String> {
    vec!["codex".to_string(), "anthropic".to_string()]
}

fn row_of(groups: &[Group], entry: &str, field: &str) -> Row {
    groups
        .iter()
        .find(|g| g.entry == entry)
        .and_then(|g| g.rows.iter().find(|r| r.field == field))
        .cloned()
        .unwrap_or_else(|| panic!("no {entry}.{field} row"))
}

#[test]
fn schema_is_the_three_files_yog_reads_and_nothing_else() {
    assert_eq!(schema_for(MODELS_YAML).map(|s| s.block), Some("models"));
    assert_eq!(schema_for("providers.yaml").map(|s| s.block), Some("roles"));
    assert_eq!(schema_for("cadence.yaml").map(|s| s.block), Some("cadence"));
    // A file yog has no reader for keeps the raw editor.
    assert!(schema_for("compact.yaml").is_none());
    assert!(schema_for("").is_none());
}

/// The clock's file (bl-3381): the default template reads as one `watcher`
/// entry whose three periods are bounded numbers, and a period edits through
/// the same anchored write every other row uses.
#[test]
fn cadence_yaml_reads_as_the_watcher_entry_and_writes_in_place() {
    use crate::app::cadence;
    let schema = schema_for(cadence::CADENCE_YAML).unwrap();
    let groups = read(&schema, cadence::TEMPLATE, &[]);
    assert_eq!(groups.len(), 1, "one entry: {groups:?}");
    assert_eq!(groups[0].entry, "watcher");
    let fields: Vec<&str> = groups[0].rows.iter().map(|r| r.field).collect();
    assert_eq!(
        fields,
        ["debounce_ms", "cheap_sweep_ms", "full_sweep_ms"],
        "the three periods, each typed"
    );
    assert!(groups[0].rows.iter().all(|r| r.fault.is_none()));
    let row = row_of(&groups, "watcher", "cheap_sweep_ms");
    let text = write(&schema, cadence::TEMPLATE, &row, "5000").unwrap();
    assert_eq!(cadence::parse(&text).cheap_sweep.as_millis(), 5000);
    assert!(
        text.contains("full_sweep_ms: 15000") && text.contains("# yog's clock"),
        "siblings and comments survive the anchored write: {text}"
    );
}

#[test]
fn models_yaml_reads_as_one_group_per_declared_id() {
    let groups = read(&MODELS_SCHEMA, SEEDED_MODELS, &rows());
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].entry, "gpt-5.4");
    let fields: Vec<&str> = groups[0].rows.iter().map(|r| r.field).collect();
    assert_eq!(
        fields,
        vec!["provider", "model_id", "capabilities", "context_window"]
    );
    // A flow sequence shows its members, not its brackets.
    let caps = row_of(&groups, "gpt-5.4", "capabilities");
    assert_eq!(caps.value, "tool_use_native, streaming");
    assert_eq!(caps.fault, None);
    assert_eq!(row_of(&groups, "gpt-5.4", "context_window").value, "400000");
    assert!(!row_of(&groups, "gpt-5.4", "provider").help.is_empty());
}

#[test]
fn providers_yaml_reads_as_one_group_per_role() {
    let groups = read(&ROLES_SCHEMA, TEMPLATE_PROVIDERS, &rows());
    let names: Vec<&str> = groups.iter().map(|g| g.entry.as_str()).collect();
    assert_eq!(names, vec!["worker", "compactor"]);
    assert_eq!(
        row_of(&groups, "worker", "tools").value,
        "bash, read_file, load_skill"
    );
    // The compactor declares no `tools:`, so it simply has no such row.
    let compactor = groups
        .iter()
        .find(|g| g.entry == "compactor")
        .map(|g| g.rows.iter().map(|r| r.field).collect::<Vec<_>>());
    assert_eq!(compactor, Some(vec!["provider", "model"]));
}

#[test]
fn a_file_with_no_block_has_no_settings() {
    assert!(read(&MODELS_SCHEMA, "steps:\n  - run\n", &rows()).is_empty());
    assert!(read(&ROLES_SCHEMA, "", &rows()).is_empty());
}

#[test]
fn a_provider_row_brazen_lacks_is_faulted_where_it_is_read() {
    let dead = "models:\n  m:\n    provider: gone\n";
    let fault = row_of(&read(&MODELS_SCHEMA, dead, &rows()), "m", "provider").fault;
    assert!(fault.is_some_and(|f| f.contains("`gone`")));
    // An empty table is no answer, so it faults nothing.
    assert_eq!(
        row_of(&read(&MODELS_SCHEMA, dead, &[]), "m", "provider").fault,
        None
    );
}

#[test]
fn an_off_shape_list_and_an_off_range_number_fault_rather_than_render_typed() {
    let odd = "models:\n  m:\n    provider: codex\n    capabilities:\n    context_window: lots\n";
    let groups = read(&MODELS_SCHEMA, odd, &rows());
    let caps = row_of(&groups, "m", "capabilities");
    assert_eq!(caps.value, "");
    assert!(caps.fault.is_some_and(|f| f.contains("inline")));
    let window = row_of(&groups, "m", "context_window");
    assert_eq!(window.value, "lots");
    assert!(window.fault.is_some_and(|f| f.contains("whole number")));
    // Zero is out of range too — the bound is a setting, not a hint.
    let zero = "models:\n  m:\n    context_window: 0\n";
    assert!(
        row_of(&read(&MODELS_SCHEMA, zero, &rows()), "m", "context_window")
            .fault
            .is_some()
    );
}

#[test]
fn writing_a_control_rewrites_one_line_and_nothing_else() {
    let groups = read(&MODELS_SCHEMA, SEEDED_MODELS, &rows());
    let row = row_of(&groups, "gpt-5.4", "provider");
    let out = write(&MODELS_SCHEMA, SEEDED_MODELS, &row, " anthropic ").unwrap();
    assert!(out.contains("    provider: anthropic\n"));
    // Every other byte survives: the comment header and the sibling fields.
    assert!(out.starts_with("# Global config-root models.yaml"));
    assert!(out.contains("    model_id: gpt-5.4\n"));
    assert!(out.contains("    capabilities: [tool_use_native, streaming]\n"));
}

#[test]
fn a_number_is_clamped_and_a_list_is_re_emitted_as_a_flow_sequence() {
    let groups = read(&MODELS_SCHEMA, SEEDED_MODELS, &rows());
    let window = row_of(&groups, "gpt-5.4", "context_window");
    let out = write(&MODELS_SCHEMA, SEEDED_MODELS, &window, "999999999999").unwrap();
    assert!(out.contains("    context_window: 100000000\n"));
    // Unparseable falls to the floor rather than writing a line yog cannot read.
    let out = write(&MODELS_SCHEMA, SEEDED_MODELS, &window, "").unwrap();
    assert!(out.contains("    context_window: 1\n"));
    let caps = row_of(&groups, "gpt-5.4", "capabilities");
    let out = write(&MODELS_SCHEMA, SEEDED_MODELS, &caps, " a , ,b ").unwrap();
    assert!(out.contains("    capabilities: [a, b]\n"));
    let out = write(&MODELS_SCHEMA, SEEDED_MODELS, &caps, "").unwrap();
    assert!(out.contains("    capabilities: []\n"));
}

#[test]
fn writing_a_role_field_declines_loudly_off_grammar() {
    let groups = read(&ROLES_SCHEMA, TEMPLATE_PROVIDERS, &rows());
    let row = row_of(&groups, "worker", "model");
    assert!(write(&ROLES_SCHEMA, TEMPLATE_PROVIDERS, &row, "opus-5").is_ok());
    // The same row against a file whose block is inline: refused, not guessed.
    let err = write(&ROLES_SCHEMA, "roles: {}\n", &row, "opus-5").unwrap_err();
    assert!(err.to_string().contains("inline value"));
}

#[test]
fn control_kinds_compare_by_value() {
    assert_eq!(Control::Provider, Control::Provider);
    assert_ne!(Control::Number { min: 1, max: 2 }, Control::Text);
}
