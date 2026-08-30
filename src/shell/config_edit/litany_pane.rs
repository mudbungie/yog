//! The §9.2 litany-global surface as controls (§9.5): `models.yaml`'s settings
//! typed one row per field, `workflows/*.yaml` on the raw fallback, and one
//! Apply for the file — because the draft, not the field, is what the §9 hash
//! guard is taken against.
//!
//! Coverage-excluded glue: the schema, the reads, the writes and every refusal
//! are [`crate::config_edit::form`]'s and are tested there; this file chooses
//! widgets.

use super::{ConfigState, NEW_WORKFLOW_HINT, RELOAD_HINT, form_ui, status};
use crate::config_edit::form::{self, Schema};
use crate::config_edit::litany_global::Editor;
use crate::model_pick::grammar::declare_model;
use crate::theme;
use status::{describe_saved, reload_status, status_line};
use std::path::PathBuf;

/// The declare-model id field (§9.2) — what the drafted entry is.
const DECLARE_HINT: &str = "The model id to declare in models.yaml. Declare drafts the entry; \
     Apply writes it. The entry is the id and its context_window — the \
     denominator of the context-fullness figure, and the one fact anything reads \
     out of this table. Typed, the whole file is `/config models <text…>`.";

/// The litany global config (§9.2/§9.5): the open file's settings as controls
/// where yog has a reader for it, the raw text where it does not, the file
/// switcher, and Apply — which judges nothing since bl-3ffa (§9.2).
pub(super) fn render(ui: &mut egui::Ui, config: &mut ConfigState, provider_rows: &[String]) {
    ui.heading(egui::RichText::new("litany global config").color(theme::integration_hue("litany")));
    ui.monospace(config.litany_editor.path().display().to_string());
    if config.litany_editor.is_new() {
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
                "Write this file: the draft replaces it whole, refused only if it \
                 changed underneath you since it was read. Typed, it is \
                 `/config models <text…>`.",
            )
            .clicked()
        {
            config.litany_status = describe_saved(config.litany_editor.apply(&config.io));
        }
        if ui.button("Reload").on_hover_text(RELOAD_HINT).clicked() {
            config.litany_status = reload_status(config.litany_editor.reload(&config.io));
        }
        if ui
            .button("models.yaml")
            .on_hover_text(
                "Load litany's global models.yaml into this editor. `/config models` \
                 with no text reads the same bytes.",
            )
            .clicked()
        {
            open(config, config.litany.models());
        }
    });
    workflow_list(ui, config);
    // Rule 1b, and the width half of it (bl-7414): the field is greedy and the
    // verb is not, so the verb is laid first and the field takes what is left.
    // Spelled the obvious way, `text_edit_singleline` claims egui's fixed 280 pt
    // `text_edit_width` whatever the pane has — which at 480x1400 was 164 pt
    // MORE than the pane, and an over-wide row does not merely overflow: it
    // ratchets the seat's own `max_rect`, so every row after it truncated to a
    // width the clip then cut, ellipsis and all.
    let create = crate::shell::row::control_last(
        ui,
        |ui| {
            ui.label("new workflow:").on_hover_text(NEW_WORKFLOW_HINT);
            ui.add(
                egui::TextEdit::singleline(&mut config.new_workflow).desired_width(f32::INFINITY),
            )
            .on_hover_text(NEW_WORKFLOW_HINT);
        },
        |ui| {
            ui.button("Create")
                .on_hover_text(
                    "Open an empty editor for a workflow file of that name. Nothing is \
                     written to disk until you press Apply — `/config workflow <name> \
                     <text…>` writes one outright.",
                )
                .clicked()
        },
    )
    .1;
    if create {
        new_workflow(config);
    }
    status_line(ui, &config.litany_status);
}

/// The open file's schema, keyed by its basename — `None` is the raw fallback.
fn schema(config: &ConfigState) -> Option<Schema> {
    let name = config.litany_editor.path().file_name()?;
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
    let groups = form::read(schema, config.litany_editor.draft(), provider_rows);
    if let Some((row, value)) = form_ui::render(ui, "litany", &groups, provider_rows) {
        match form::write(schema, config.litany_editor.draft(), &row, &value) {
            Ok(text) => {
                config.litany_editor.set_draft(text);
                config.litany_status =
                    format!("{}.{} drafted — Apply to write", row.entry, row.field);
            }
            Err(e) => config.litany_status = e.to_string(),
        }
    }
    declare(ui, config, schema);
}

/// Add a model entry to `models.yaml` — **the one seat that authors one** since
/// bl-d9cb, the §9.4 picker's half of this write having been deleted with the
/// litany cross-check that justified it. Only offered for the file that has a
/// `models:` block to add to.
///
/// The row it once picked went with bl-3ffa: the entry names no provider row any
/// more, because nothing dispatched through the one it named.
fn declare(ui: &mut egui::Ui, config: &mut ConfigState, schema: &Schema) {
    if schema.block != crate::model_pick::grammar::MODELS {
        return;
    }
    // Rule 1b again (bl-7414): the id field is the greedy half, the verb is not.
    let typed = &mut config.new_model;
    let declare = crate::shell::row::control_last(
        ui,
        |ui| {
            ui.label("declare model:");
            ui.add(egui::TextEdit::singleline(typed).desired_width(f32::INFINITY))
                .on_hover_text(DECLARE_HINT);
        },
        |ui| {
            ui.button("Declare")
                .on_hover_text(
                    "Draft a `models:` entry for that id. Nothing lands until Apply. \
                     The entry declares the context window yog measures fullness \
                     against; picking a model does not write one. Typed, the whole \
                     file is `/config models <text…>`.",
                )
                .clicked()
        },
    )
    .1;
    if declare {
        config.litany_status = declared(config);
    }
}

/// Draft the new entry, or say why not. An id the file already declares is
/// nothing to write, which is a value and not a failure.
///
/// The entry's window is §9.4's declared default under the note that says so:
/// this seat is a typed id with no roster behind it, and since bl-d9cb no seat
/// has one — brazen's served window is a query, never a field seeded here.
fn declared(config: &mut ConfigState) -> String {
    match declare_model(config.litany_editor.draft(), config.new_model.trim()) {
        Ok(Some(text)) => {
            config.litany_editor.set_draft(text);
            "declared — Apply to write".to_string()
        }
        Ok(None) => "already declared".to_string(),
        Err(e) => e.to_string(),
    }
}

/// The raw text of the open file — the escape hatch every typed surface keeps
/// (§9.5): the settings above are the fields yog has a grammar for, and this is
/// everything else the file may carry. Folded away so it is the exception.
fn raw(ui: &mut egui::Ui, config: &mut ConfigState) {
    // The fold is keyed by the FILE, not by its label (bl-9551):
    // `CollapsingHeader` derives its id from its text, and the litany and
    // yog surfaces both head their raw escape "raw text" — one id, so the
    // two folds opened and shut together and egui painted its id-clash
    // warning straight across the Apply row beneath them. The word the
    // operator reads is one thing; the seat it names is another.
    egui::CollapsingHeader::new("raw text")
        .id_salt("raw-models")
        .show(ui, |ui| {
            form_ui::raw_editor(ui, config.litany_editor.draft_mut());
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
    match config.litany.new_workflow(&config.new_workflow) {
        Ok(path) => {
            config.litany_editor = Editor::seeded(path, b"");
            config.litany_status = "new workflow ready".to_string();
        }
        Err(e) => config.litany_status = e.to_string(),
    }
}

/// Open a litany-global file into the editor (its Conflict recovery is Reload).
fn open(config: &mut ConfigState, path: PathBuf) {
    match Editor::load(path.clone(), &config.io) {
        Ok(editor) => config.litany_editor = editor,
        Err(_) => config.litany_editor = Editor::seeded(path, b""),
    }
}
