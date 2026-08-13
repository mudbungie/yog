//! The §9.2 lernie-global surface as controls (§9.5): `models.yaml`'s settings
//! typed one row per field, `workflows/*.yaml` on the raw fallback, and one
//! Apply for the file — because the draft, not the field, is what the §9 hash
//! guard is taken against.
//!
//! Coverage-excluded glue: the schema, the reads, the writes and every refusal
//! are [`crate::config_edit::form`]'s and are tested there; this file chooses
//! widgets.

use super::{ConfigState, NEW_WORKFLOW_HINT, RELOAD_HINT, form_ui, status};
use crate::config_edit::form::{self, Schema};
use crate::config_edit::lernie_global::Editor;
use crate::model_pick::grammar::declare_model;
use crate::theme;
use status::{describe_saved, reload_status, status_line};
use std::path::PathBuf;

/// The declare-model id field (§9.2) — what the drafted entry is.
const DECLARE_HINT: &str = "The model id to declare in models.yaml, on the provider row picked \
     beside it. Declare drafts the entry; Apply writes it. Typed, the same write is \
     `/model <role> <provider> <model-id>`.";

/// The lernie global config (§9.2/§9.5): the open file's settings as controls
/// where yog has a reader for it, the raw text where it does not, the file
/// switcher, and Apply — gated on brazen's provider rows exactly as before.
pub(super) fn render(ui: &mut egui::Ui, config: &mut ConfigState, provider_rows: &[String]) {
    ui.heading(egui::RichText::new("lernie global config").color(theme::integration_hue("lernie")));
    ui.monospace(config.lernie_editor.path().display().to_string());
    if config.lernie_editor.is_new() {
        ui.weak("(new file)");
    }
    match schema(config) {
        Some(schema) => settings(ui, config, &schema, provider_rows),
        None => {
            ui.weak(form_ui::NO_READER);
        }
    }
    raw(ui, config);
    ui.horizontal(|ui| {
        if ui
            .button("Apply")
            .on_hover_text(
                "Write this file, refusing it if it names a model whose provider \
                 brazen does not have — which is the failure you would otherwise \
                 only meet mid-conversation. Typed, it is `/config models <text…>`.",
            )
            .clicked()
        {
            config.lernie_status =
                describe_saved(config.lernie_editor.apply(provider_rows, &config.io));
        }
        if ui.button("Reload").on_hover_text(RELOAD_HINT).clicked() {
            config.lernie_status = reload_status(config.lernie_editor.reload(&config.io));
        }
        if ui
            .button("models.yaml")
            .on_hover_text(
                "Load lernie's global models.yaml into this editor. `/config models` \
                 with no text reads the same bytes.",
            )
            .clicked()
        {
            open(config, config.lernie.models());
        }
    });
    workflow_list(ui, config);
    ui.horizontal(|ui| {
        ui.label("new workflow:").on_hover_text(NEW_WORKFLOW_HINT);
        ui.text_edit_singleline(&mut config.new_workflow)
            .on_hover_text(NEW_WORKFLOW_HINT);
        if ui
            .button("Create")
            .on_hover_text(
                "Open an empty editor for a workflow file of that name. Nothing is \
                 written to disk until you press Apply — `/config workflow <name> \
                 <text…>` writes one outright.",
            )
            .clicked()
        {
            new_workflow(config);
        }
    });
    status_line(ui, &config.lernie_status);
}

/// The open file's schema, keyed by its basename — `None` is the raw fallback.
fn schema(config: &ConfigState) -> Option<Schema> {
    let name = config.lernie_editor.path().file_name()?;
    form::schema_for(&name.to_string_lossy())
}

/// Every setting the open file declares, as its control. An edit rewrites the
/// draft through the anchored grammar; a file the grammar cannot hold declines
/// loudly into the status line rather than being guessed at.
fn settings(
    ui: &mut egui::Ui,
    config: &mut ConfigState,
    schema: &Schema,
    provider_rows: &[String],
) {
    let groups = form::read(schema, config.lernie_editor.draft(), provider_rows);
    if let Some((row, value)) = form_ui::render(ui, "lernie", &groups, provider_rows) {
        match form::write(schema, config.lernie_editor.draft(), &row, &value) {
            Ok(text) => {
                config.lernie_editor.set_draft(text);
                config.lernie_status =
                    format!("{}.{} drafted — Apply to write", row.entry, row.field);
            }
            Err(e) => config.lernie_status = e.to_string(),
        }
    }
    declare(ui, config, schema, provider_rows);
}

/// Add a model entry to `models.yaml` — the same [`declare_model`] write the
/// §9.4 picker makes, so a hand-declared model and a picked one are one shape.
/// Only offered for the file that has a `models:` block to add to.
fn declare(ui: &mut egui::Ui, config: &mut ConfigState, schema: &Schema, provider_rows: &[String]) {
    if schema.block != crate::model_pick::grammar::MODELS {
        return;
    }
    ui.horizontal(|ui| {
        ui.label("declare model:");
        ui.text_edit_singleline(&mut config.new_model)
            .on_hover_text(DECLARE_HINT);
        egui::ComboBox::from_id_salt("declare-model-row")
            .selected_text(&config.new_model_row)
            .show_ui(ui, |ui| {
                for name in provider_rows {
                    ui.selectable_value(&mut config.new_model_row, name.clone(), name)
                        .on_hover_text(
                            "The provider row this model calls through — one of the \
                             rows brazen actually has. Typed, the pick is the middle \
                             word of `/model <role> <provider> <model-id>`.",
                        );
                }
            })
            .response
            .on_hover_text(
                "Which provider row the new model calls through — one of the rows \
                 brazen actually has. Typed, the pick is the middle word of \
                 `/model <role> <provider> <model-id>`.",
            );
        if ui
            .button("Declare")
            .on_hover_text(
                "Draft a `models:` entry for that id on the picked provider row — \
                 the same write the model picker makes. Nothing lands until Apply; \
                 `/model <role> <provider> <model-id>` does both halves at once.",
            )
            .clicked()
        {
            config.lernie_status = declared(config);
        }
    });
}

/// Draft the new entry, or say why not. An id already on the picked row is
/// nothing to write, which is a value and not a failure.
fn declared(config: &mut ConfigState) -> String {
    match declare_model(
        config.lernie_editor.draft(),
        config.new_model.trim(),
        config.new_model_row.trim(),
    ) {
        Ok(Some(text)) => {
            config.lernie_editor.set_draft(text);
            "declared — Apply to write".to_string()
        }
        Ok(None) => "already declared on that row".to_string(),
        Err(e) => e.to_string(),
    }
}

/// The raw text of the open file — the escape hatch every typed surface keeps
/// (§9.5): the settings above are the fields yog has a grammar for, and this is
/// everything else the file may carry. Folded away so it is the exception.
fn raw(ui: &mut egui::Ui, config: &mut ConfigState) {
    // The fold is keyed by the FILE, not by its label (bl-9551):
    // `CollapsingHeader` derives its id from its text, and the lernie and
    // yog surfaces both head their raw escape "raw text" — one id, so the
    // two folds opened and shut together and egui painted its id-clash
    // warning straight across the Apply row beneath them. The word the
    // operator reads is one thing; the seat it names is another.
    egui::CollapsingHeader::new("raw text")
        .id_salt("raw-models")
        .show(ui, |ui| {
            form_ui::raw_editor(ui, config.lernie_editor.draft_mut());
        })
        .header_response
        .on_hover_text(
            "Open the file's raw text — everything it carries beyond the settings \
             above. Edits here change nothing until Apply. No key of its own: Tab \
             reaches it, Space presses it.",
        );
}

/// The existing `workflows/*.yaml` as open buttons, listed at the open gesture
/// (§7.2 — never a readdir per frame).
fn workflow_list(ui: &mut egui::Ui, config: &mut ConfigState) {
    let files = config.workflows.clone();
    ui.horizontal_wrapped(|ui| {
        for file in files {
            let label = file.file_name().map(|n| n.to_string_lossy().into_owned());
            if ui
                .button(label.unwrap_or_default())
                .on_hover_text(
                    "Load this workflow file into this editor. \
                     `/config workflow <name>` with no text reads the same bytes.",
                )
                .clicked()
            {
                open(config, file);
            }
        }
    });
}

/// Seed a new-workflow editor from the drafted name (§9.2), or surface why the
/// name is unsafe.
fn new_workflow(config: &mut ConfigState) {
    match config.lernie.new_workflow(&config.new_workflow) {
        Ok(path) => {
            config.lernie_editor = Editor::seeded(path, b"");
            config.lernie_status = "new workflow ready".to_string();
        }
        Err(e) => config.lernie_status = e.to_string(),
    }
}

/// Open a lernie-global file into the editor (its Conflict recovery is Reload).
fn open(config: &mut ConfigState, path: PathBuf) {
    match Editor::load(path.clone(), &config.io) {
        Ok(editor) => config.lernie_editor = editor,
        Err(_) => config.lernie_editor = Editor::seeded(path, b""),
    }
}
