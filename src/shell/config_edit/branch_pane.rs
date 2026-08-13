//! The §9.3 per-workspace config-branch pane: pick the lineage, pick the file,
//! load what it actually holds, edit it — typed where yog has a grammar for the
//! file (§9.5), raw where it does not — and drive `lernie config`, the only
//! lawful `config/*` writer.
//!
//! Two things this pane used to be: a free-text branch name and a free-text
//! path over an empty body, so a config commit was authored blind against a
//! file nobody had read. Both are now chosen from what the workspace actually
//! has. The git reads happen on the **Browse** and **Load** gestures, never per
//! frame (§7.2, bl-ee0a — the listing used to spawn `for-each-ref` every frame).
//!
//! Coverage-excluded glue: the staging, the plan, the drive and the typed form
//! are tested in `config_edit`; this file only wires widgets. Since bl-3f46 the
//! Send **is** the boundary's
//! [`ApplyConfig`](crate::boundary::Action::ApplyConfig) variant on a
//! [`Branch`](crate::boundary::config::ConfigFile::Branch) destination — the
//! click-glue constructs it and calls the chokepoint, so the lineage write has
//! one implementation and a headless spelling.

use super::{ConfigState, form_ui, status};
use crate::AppModel;
use crate::boundary::Action;
use crate::boundary::config::ConfigFile;
use crate::cli_outbound::Cli;
use crate::config_edit::branch::edit::EditOrigin;
use crate::config_edit::branch::{config_branches, config_file, config_tree};
use crate::config_edit::form::{self, Schema};
use status::status_line;
use std::path::{Path, PathBuf};

/// The dropdown's escape: a lineage the workspace does not have yet, named in
/// the text field the choice reveals. Every list ends in its own escape (§9.4).
const NEW_LINEAGE: &str = "new lineage…";

/// The lineage choice (§9.3) — one phrase for the dropdown and the name field
/// it reveals (§11 rule 4: a phrase worn twice is a named const).
const LINEAGE_HINT: &str = "Which config branch to write. `default` is the one new conversations in \
     this workspace read; any other name is a lineage of its own. Typed, the lineage is \
     `/config branch|fork|orphan <name> <text…>`.";

/// The file choice (§9.3) — worn by the dropdown and the free-path field.
const FILE_HINT: &str = "Which file inside the config branch to edit — `providers.yaml`, for \
     instance. It is created if the branch has no such file. Typed, the file rides the \
     same `/config` line.";

/// Per-workspace config branches (§9.3): the lineage, the file, its settings,
/// and the staged edit that drives `lernie config`.
pub(super) fn render(
    ui: &mut egui::Ui,
    model: &mut AppModel,
    config: &mut ConfigState,
    provider_rows: &[String],
    lernie: &Cli,
    bl: &Cli,
) {
    ui.heading("workspace config branches");
    let Some(ws) = model.focused_workspace().map(PathBuf::from) else {
        ui.weak("focus a workspace to edit its config branches");
        return;
    };
    // The browse half (§9.3, §5.1 #18): what lineages exist and where each tip
    // sits. The selector below chooses among them; this says what they are.
    for b in &config.branches {
        ui.monospace(format!("config/{} @ {}", b.name, b.tip_short_oid));
    }
    lineage(ui, config, &ws);
    file_row(ui, config, &ws);
    match schema_of(&config.cb_path) {
        Some(schema) => settings(ui, config, &schema, provider_rows),
        None => {
            ui.weak(form_ui::NO_READER);
        }
    }
    egui::CollapsingHeader::new("raw file body")
        .show(ui, |ui| {
            form_ui::raw_editor(ui, &mut config.cb_body);
        })
        .header_response
        .on_hover_text(
            "Open the file's whole contents, exactly as they will be committed — \
             the raw text behind the settings above. No key of its own: Tab reaches \
             it, Space presses it.",
        );
    if ui
        .button("Send (stage + lernie config)")
        .on_hover_text(
            "Stage this file and run `lernie config` — the only thing allowed to \
             write a config branch. New conversations in this workspace read the \
             result; running ones stay on the commit they started from. Typed, it is \
             `/config branch <name> <text…>`.",
        )
        .clicked()
    {
        config.cb_status = send_edit(model, config, &ws, lernie, bl);
        // The pane caused the advance, so the pane re-reads it (§7.2).
        reread(config, Some(&ws));
    }
    status_line(ui, &config.cb_status);
}

/// Read the workspace's lineages and the selected one's file listing — the
/// §9 read-on-demand rule extended to git: the pane's own open gesture fills
/// this, and the pane refills it after a `lernie config` it caused (§7.2 — the
/// frame marks what it changed; it never polls). Both answers come from one
/// pass, so the listing and the tree can never be of different commits.
pub(super) fn reread(config: &mut ConfigState, workspace: Option<&Path>) {
    let Some(ws) = workspace else {
        config.branches.clear();
        config.cb_files.clear();
        return;
    };
    config.branches = config_branches(ws).unwrap_or_default();
    config.cb_files = tree(config, ws);
}

/// The selected lineage's tree, or nothing when the selection is a lineage the
/// workspace does not have yet (there is no ref to read).
fn tree(config: &ConfigState, ws: &Path) -> Vec<String> {
    if !config.branches.iter().any(|b| b.name == config.cb_name) {
        return Vec::new();
    }
    config_tree(ws, &format!("config/{}", config.cb_name)).unwrap_or_default()
}

/// The lineage row: the branches the open gesture found, the escape that names
/// a new one, and the advance/orphan origin. Choosing a lineage re-reads its
/// tree — one git call on the gesture that changed the answer, never per frame.
fn lineage(ui: &mut egui::Ui, config: &mut ConfigState, ws: &Path) {
    let before = config.cb_name.clone();
    let known = config.branches.iter().any(|b| b.name == config.cb_name);
    ui.horizontal(|ui| {
        ui.label("lineage:");
        let shown = if known {
            config.cb_name.clone()
        } else {
            NEW_LINEAGE.to_string()
        };
        egui::ComboBox::from_id_salt("config-branch")
            .selected_text(shown)
            .show_ui(ui, |ui| {
                for b in &config.branches {
                    ui.selectable_value(
                        &mut config.cb_name,
                        b.name.clone(),
                        format!("config/{} @ {}", b.name, b.tip_short_oid),
                    )
                    .on_hover_text(LINEAGE_HINT);
                }
                ui.selectable_value(&mut config.cb_name, String::new(), NEW_LINEAGE)
                    .on_hover_text(
                        "Name a config branch this workspace does not have yet, in the \
                         field this choice reveals — the name `/config orphan <name>` \
                         would carry.",
                    );
            })
            .response
            .on_hover_text(LINEAGE_HINT);
        if !known {
            ui.text_edit_singleline(&mut config.cb_name)
                .on_hover_text(LINEAGE_HINT);
        }
        ui.selectable_value(&mut config.cb_origin, EditOrigin::Advance, "advance")
            .on_hover_text(
                "Commit on top of the branch as it stands, keeping everything already \
                 on it. Typed, it is `/config branch <name> <text…>`.",
            );
        ui.selectable_value(&mut config.cb_origin, EditOrigin::Orphan, "orphan")
            .on_hover_text(
                "Start the branch over from nothing — only the file below survives, \
                 and its previous history is left behind. Typed, it is \
                 `/config orphan <name> <text…>`.",
            );
    });
    if config.cb_name != before {
        config.cb_files = tree(config, ws);
    }
}

/// The file row: the paths the lineage's commit actually holds, and the Load
/// that fills the body from it — an edit is over what is there, never a blank.
fn file_row(ui: &mut egui::Ui, config: &mut ConfigState, ws: &Path) {
    ui.horizontal(|ui| {
        ui.label("file:").on_hover_text(FILE_HINT);
        egui::ComboBox::from_id_salt("config-branch-file")
            .selected_text(&config.cb_path)
            .show_ui(ui, |ui| {
                for path in &config.cb_files.clone() {
                    ui.selectable_value(&mut config.cb_path, path.clone(), path)
                        .on_hover_text(FILE_HINT);
                }
            })
            .response
            .on_hover_text(FILE_HINT);
        if config.cb_files.is_empty() {
            ui.text_edit_singleline(&mut config.cb_path)
                .on_hover_text(FILE_HINT);
        }
        if ui
            .button("Load")
            .on_hover_text(
                "Read this file out of the selected lineage's tip into the editor \
                 below, so the edit starts from what is actually there. \
                 `/config branch <name>` with no text reads the same bytes.",
            )
            .clicked()
        {
            config.cb_status = load(config, ws);
        }
    });
}

/// Read the selected file out of the selected lineage's tip into the body.
fn load(config: &mut ConfigState, ws: &Path) -> String {
    let refspec = format!("config/{}", config.cb_name);
    match config_file(ws, &refspec, &config.cb_path) {
        Ok(bytes) => {
            config.cb_body = String::from_utf8_lossy(&bytes).into_owned();
            format!("loaded {refspec}:{}", config.cb_path)
        }
        Err(e) => format!("load: {e}"),
    }
}

/// The schema for the selected path's basename, or `None` — the raw fallback.
fn schema_of(path: &str) -> Option<Schema> {
    form::schema_for(path.rsplit('/').next()?)
}

/// The loaded file's settings as controls; an edit rewrites the body in RAM and
/// `lernie config` remains the writer.
fn settings(
    ui: &mut egui::Ui,
    config: &mut ConfigState,
    schema: &Schema,
    provider_rows: &[String],
) {
    let groups = form::read(schema, &config.cb_body, provider_rows);
    if let Some((row, value)) = form_ui::render(ui, "config-branch", &groups, provider_rows) {
        match form::write(schema, &config.cb_body, &row, &value) {
            Ok(text) => {
                config.cb_body = text;
                config.cb_status = format!("{}.{} drafted — Send to commit", row.entry, row.field);
            }
            Err(e) => config.cb_status = e.to_string(),
        }
    }
}

/// Send the drafted file through the boundary (§8.5): the variant carries the
/// destination — this workspace, this lineage, this path, this origin — and the
/// full staged text, and the chokepoint stages it and drives `lernie config`.
fn send_edit(
    model: &mut AppModel,
    config: &ConfigState,
    ws: &Path,
    lernie: &Cli,
    bl: &Cli,
) -> String {
    let deps = model.boundary_deps(lernie, bl);
    let action = Action::ApplyConfig {
        file: ConfigFile::Branch {
            workspace: ws.to_path_buf(),
            lineage: config.cb_name.clone(),
            origin: config.cb_origin.clone(),
            path: config.cb_path.clone(),
        },
        text: config.cb_body.clone(),
    };
    match model.dispatch(&deps, &crate::shell::now_ts(), &action) {
        // The exit in words, through the one projection (bl-afa9): a piped
        // drive can land on the §4.2 `-1` sentinel, and "exit -1" reads as
        // a signal death rather than "ran, status not observable".
        Ok(crate::boundary::reply::Reply::Outcome(outcome)) => format!(
            "lernie config: {}",
            crate::opslog::exit::ExitKind::of(outcome.exit, "lernie").label()
        ),
        Ok(other) => format!("unexpected reply: {other:?}"),
        Err(e) => e,
    }
}
