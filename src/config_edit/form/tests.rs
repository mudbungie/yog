//! The §9.5 typed-settings view-model: what each control shows, what it faults
//! on, and what it writes back. Every arm is pure text → text, so the whole
//! table is driven without egui, disk or brazen.

use super::{
    CADENCE_SCHEMA, Control, FieldSpec, ROLES_SCHEMA, Row, Schema, read, schema_for, write,
};
use crate::app::cadence::{self, FULL_SWEEP_MS, TEMPLATE};
use crate::config_edit::litany_global::MODELS_YAML;
use crate::model_pick::tests::TEMPLATE_PROVIDERS;

fn rows() -> Vec<String> {
    vec!["codex".to_string(), "anthropic".to_string()]
}

fn row_of(rows: &[Row], entry: &str, field: &str) -> Row {
    rows.iter()
        .find(|r| r.entry == entry && r.name == field)
        .cloned()
        .unwrap_or_else(|| panic!("no {entry}.{field} row"))
}

/// The settings one entry declares, in file order — what a `Group` used to be,
/// derived from the flat answer the way a seat derives it (bl-dc3f).
fn fields_of(rows: &[Row], entry: &str) -> Vec<String> {
    rows.iter()
        .filter(|r| r.entry == entry)
        .map(|r| r.name.clone())
        .collect()
}

/// The schema's own spec for one field — what [`write`] is handed now that a
/// row may have come off a wire.
fn spec_of(schema: &Schema, name: &str) -> FieldSpec {
    *schema
        .fields
        .iter()
        .find(|f| f.name == name)
        .unwrap_or_else(|| panic!("no {name} field"))
}

#[test]
fn schema_is_the_two_files_yog_reads_and_nothing_else() {
    assert_eq!(schema_for("providers.yaml").map(|s| s.block), Some("roles"));
    assert_eq!(schema_for("cadence.yaml").map(|s| s.block), Some("cadence"));
    // A file yog has no reader for keeps the raw editor — `models.yaml` among
    // them since bl-9c8a took its one typed row to the step record.
    assert!(schema_for(MODELS_YAML).is_none());
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
    let rows = read(&schema, cadence::TEMPLATE, &[]);
    assert!(rows.iter().all(|r| r.entry == "watcher"), "{rows:?}");
    assert_eq!(
        fields_of(&rows, "watcher"),
        ["debounce_ms", "cheap_sweep_ms", "full_sweep_ms"],
        "the three periods, each typed"
    );
    assert!(rows.iter().all(|r| r.fault.is_none()));
    let text = write(
        &schema,
        cadence::TEMPLATE,
        "watcher",
        &spec_of(&schema, "cheap_sweep_ms"),
        "5000",
    )
    .unwrap();
    assert_eq!(cadence::parse(&text).cheap_sweep.as_millis(), 5000);
    assert!(
        text.contains("full_sweep_ms: 15000") && text.contains("# yog's clock"),
        "siblings and comments survive the anchored write: {text}"
    );
}

/// `cadence.yaml` reads as one group of bounded numbers, each with the words
/// the pane paints beside it. `models.yaml` read this way too until bl-9c8a
/// deleted its one typed row: a control over a fact nothing consumes is a
/// setting that cannot matter, and the pane shows the settings that exist.
#[test]
fn cadence_yaml_reads_as_one_group_of_bounded_numbers() {
    let rows = read(&CADENCE_SCHEMA, TEMPLATE, &rows());
    assert_eq!(
        fields_of(&rows, cadence::WATCHER),
        vec![cadence::DEBOUNCE_MS, cadence::CHEAP_SWEEP_MS, FULL_SWEEP_MS]
    );
    assert_eq!(rows.len(), 3, "one entry, its three settings: {rows:?}");
    let full = row_of(&rows, cadence::WATCHER, FULL_SWEEP_MS);
    assert_eq!(full.value, "15000");
    assert!(!full.help.is_empty());
}

#[test]
fn providers_yaml_reads_as_one_group_per_role() {
    let rows = read(&ROLES_SCHEMA, TEMPLATE_PROVIDERS, &rows());
    let mut names: Vec<&str> = rows.iter().map(|r| r.entry.as_str()).collect();
    names.dedup();
    assert_eq!(
        names,
        vec!["worker", "compactor"],
        "entry order is file order"
    );
    assert_eq!(
        row_of(&rows, "worker", "tools").value,
        "bash, read_file, load_skill"
    );
    // The compactor declares no `tools:`, so it simply has no such row.
    assert_eq!(fields_of(&rows, "compactor"), vec!["provider", "model"]);
}

#[test]
fn a_file_with_no_block_has_no_settings() {
    assert!(read(&CADENCE_SCHEMA, "steps:\n  - run\n", &rows()).is_empty());
    assert!(read(&ROLES_SCHEMA, "", &rows()).is_empty());
}

/// The provider control faults where the pointer actually lives —
/// `roles.<r>.provider`, the whole of a role's binding. It read
/// `models.<id>.provider` too until bl-3ffa, which is the field the retired §9.2
/// gate judged; nothing dispatched through it.
#[test]
fn a_provider_row_brazen_lacks_is_faulted_where_it_is_read() {
    let dead = "roles:\n  worker:\n    provider: gone\n    model: m\n";
    let fault = row_of(&read(&ROLES_SCHEMA, dead, &rows()), "worker", "provider").fault;
    assert!(fault.is_some_and(|f| f.contains("`gone`")));
    // An empty table is no answer, so it faults nothing.
    assert_eq!(
        row_of(&read(&ROLES_SCHEMA, dead, &[]), "worker", "provider").fault,
        None
    );
}

#[test]
fn an_off_shape_list_and_an_off_range_number_fault_rather_than_render_typed() {
    // The list control's own file is `providers.yaml` now (bl-3ffa): a role's
    // `tools:` is the one inline flow sequence a surface edits.
    let odd_list = "roles:\n  worker:\n    provider: codex\n    model: m\n    tools:\n";
    let tools = row_of(&read(&ROLES_SCHEMA, odd_list, &rows()), "worker", "tools");
    assert_eq!(tools.value, "");
    assert!(tools.fault.is_some_and(|f| f.contains("inline")));
    let odd = "cadence:\n  watcher:\n    full_sweep_ms: lots\n";
    let period = row_of(
        &read(&CADENCE_SCHEMA, odd, &rows()),
        "watcher",
        FULL_SWEEP_MS,
    );
    assert_eq!(period.value, "lots");
    assert!(period.fault.is_some_and(|f| f.contains("whole number")));
    // Zero is out of range too — the bound is a setting, not a hint.
    let zero = "cadence:\n  watcher:\n    full_sweep_ms: 0\n";
    assert!(
        row_of(
            &read(&CADENCE_SCHEMA, zero, &rows()),
            "watcher",
            FULL_SWEEP_MS
        )
        .fault
        .is_some()
    );
}

#[test]
fn writing_a_control_rewrites_one_line_and_nothing_else() {
    let spec = spec_of(&ROLES_SCHEMA, "model");
    let text = format!("# header\n{TEMPLATE_PROVIDERS}");
    let out = write(&ROLES_SCHEMA, &text, "worker", &spec, " opus-5 ").unwrap();
    assert!(out.contains("  worker:\n    provider: codex\n    model: opus-5\n"));
    // Every other byte survives: the comment header, the fields beside it and
    // the other entry.
    assert!(out.starts_with("# header\n"));
    assert!(out.contains("    tools: [bash, read_file, load_skill]\n"));
    assert!(out.contains("  compactor:\n    provider: codex\n    model: gpt-5.4-mini\n"));
}

#[test]
fn a_number_is_clamped_and_a_list_is_re_emitted_as_a_flow_sequence() {
    let period = spec_of(&CADENCE_SCHEMA, FULL_SWEEP_MS);
    let at = |v| write(&CADENCE_SCHEMA, TEMPLATE, "watcher", &period, v).unwrap();
    assert!(at("999999999999").contains("    full_sweep_ms: 3600000\n"));
    // Unparseable falls to the floor rather than writing a line yog cannot read.
    assert!(at("").contains("    full_sweep_ms: 1000\n"));
    let tools = spec_of(&ROLES_SCHEMA, "tools");
    let at = |v| write(&ROLES_SCHEMA, TEMPLATE_PROVIDERS, "worker", &tools, v).unwrap();
    assert!(at(" a , ,b ").contains("    tools: [a, b]\n"));
    assert!(at("").contains("    tools: []\n"));
}

#[test]
fn writing_a_role_field_declines_loudly_off_grammar() {
    let model = spec_of(&ROLES_SCHEMA, "model");
    assert!(
        write(
            &ROLES_SCHEMA,
            TEMPLATE_PROVIDERS,
            "worker",
            &model,
            "opus-5"
        )
        .is_ok()
    );
    // The same field against a file whose block is inline: refused, not guessed.
    let err = write(&ROLES_SCHEMA, "roles: {}\n", "worker", &model, "opus-5").unwrap_err();
    assert!(err.to_string().contains("inline value"));
}

#[test]
fn control_kinds_compare_by_value() {
    assert_eq!(Control::Provider, Control::Provider);
    assert_ne!(Control::Number { min: 1, max: 2 }, Control::Text);
}
