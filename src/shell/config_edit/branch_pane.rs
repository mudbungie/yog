//! The §9.3 per-workspace config-branch pane: pick the lineage, pick the file,
//! load what it actually holds, edit it — typed where yog has a grammar for the
//! file (§9.5), raw where it does not — and drive `litany config`, the only
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
//!
//! The two rows that say *which file* — the lineage strip and the file strip —
//! are `branch_pane/pick`.

use super::{ConfigState, form_ui, status};
use crate::AppModel;
use crate::config_edit::branch::config_branches;
use crate::config_edit::form::{self, Schema};
use status::status_line;
use std::path::Path;

mod pick;

/// Per-workspace config branches (§9.3): the lineage, the file, its settings,
/// and the staged edit that drives `litany config`.
pub(super) fn render(
    ui: &mut egui::Ui,
    model: &mut AppModel,
    config: &mut ConfigState,
    provider_rows: &[String],
) {
    ui.heading("workspace config branches");
    let Some(ws) = model.focused_workspace() else {
        ui.weak("focus a workspace to edit its config branches");
        return;
    };
    // The browse half (§9.3, §5.1 #18): what lineages exist and where each tip
    // sits. The selector below chooses among them; this says what they are.
    for b in &config.branches {
        ui.monospace(format!("config/{} @ {}", b.name, b.tip_short_oid));
    }
    // The send below is a post (REMOTE §9.8, bl-4841), so its receipt is folded
    // here, on the frame it lands — and the re-read it triggers happens then
    // too: until the engine answers, the pre-write branch is still the branch.
    super::send::settle(model, config, &ws);
    pick::lineage(ui, config, &ws);
    pick::file_row(ui, config, &ws);
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
        .button("Send (stage + litany config)")
        .on_hover_text(
            "Stage this file and run `litany config` — the only thing allowed to \
             write a config branch. New conversations in this workspace read the \
             result; running ones stay on the commit they started from. Typed, it is \
             `/config branch <name> <text…>`.",
        )
        .clicked()
    {
        super::send::edit(model, config, &ws);
    }
    status_line(ui, &config.cb_act.line());
}

/// Read the workspace's lineages and the selected one's file listing — the
/// §9 read-on-demand rule extended to git: the pane's own open gesture fills
/// this, and the pane refills it after a `litany config` it caused (§7.2 — the
/// frame marks what it changed; it never polls). Both answers come from one
/// pass, so the listing and the tree can never be of different commits.
pub(super) fn reread(config: &mut ConfigState, workspace: Option<&Path>) {
    let Some(ws) = workspace else {
        config.branches.clear();
        config.cb_files.clear();
        return;
    };
    config.branches = config_branches(ws).unwrap_or_default();
    config.cb_files = pick::tree(config, ws);
}

/// The schema for the selected path's basename, or `None` — the raw fallback.
fn schema_of(path: &str) -> Option<Schema> {
    form::schema_for(path.rsplit('/').next()?)
}

/// The loaded file's settings as controls; an edit rewrites the body in RAM and
/// `litany config` remains the writer.
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
                config.cb_act.say(format!(
                    "{}.{} drafted — Send to commit",
                    row.entry, row.field
                ));
            }
            Err(e) => config.cb_act.say(e.to_string()),
        }
    }
}
