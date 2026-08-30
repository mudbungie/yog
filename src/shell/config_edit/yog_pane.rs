//! The yog clock surface (§7.2, §9.5, bl-3381): `cadence.yaml`'s periods as
//! bounded-number controls, over the same [`Editor`] + §9 Apply pipeline every
//! other file surface uses — the file is yog's own, the discipline is not new.
//!
//! Coverage-excluded glue: the schema, the parse, the round-trip and the
//! worker-side adoption are tested in [`crate::app::cadence`] and
//! [`crate::config_edit::form`]; this file chooses widgets.

use super::{ConfigState, RELOAD_HINT, form_ui, status};
use crate::config_edit::form::{self, CADENCE_SCHEMA};
use status::{describe_saved, reload_status, status_line};

/// The clock's settings as controls, the raw escape, and Apply. No provider
/// gate has anything to say about a duration, so Apply passes no rows — an
/// empty set gates nothing, the same contract the workflows files ride.
pub(super) fn render(ui: &mut egui::Ui, config: &mut ConfigState) {
    ui.heading("yog clock");
    ui.monospace(config.cadence_editor.path().display().to_string());
    ui.weak(
        "the watcher-cycle cadence — worker sweeps, debounce, and every rhythm \
         derived from them; delete the file to restore the defaults",
    );
    let groups = form::read(&CADENCE_SCHEMA, config.cadence_editor.draft(), &[]);
    if let Some((row, value)) = form_ui::render(ui, "cadence", &groups, &[]) {
        match form::write(&CADENCE_SCHEMA, config.cadence_editor.draft(), &row, &value) {
            Ok(text) => {
                config.cadence_editor.set_draft(text);
                config.cadence_status =
                    format!("{}.{} drafted — Apply to write", row.entry, row.field);
            }
            Err(e) => config.cadence_status = e.to_string(),
        }
    }
    // The fold is keyed by the FILE, not by its label (bl-9551):
    // `CollapsingHeader` derives its id from its text, and the litany and
    // yog surfaces both head their raw escape "raw text" — one id, so the
    // two folds opened and shut together and egui painted its id-clash
    // warning straight across the Apply row beneath them. The word the
    // operator reads is one thing; the seat it names is another.
    egui::CollapsingHeader::new("raw text")
        .id_salt("raw-cadence")
        .show(ui, |ui| {
            form_ui::raw_editor(ui, config.cadence_editor.draft_mut());
        })
        .header_response
        .on_hover_text(
            "Open the file's raw text — everything it carries beyond the settings \
             above. Edits here change nothing until Apply. No key of its own: Tab \
             reaches it, Space presses it.",
        );
    ui.horizontal(|ui| {
        if ui
            .button("Apply")
            .on_hover_text(
                "Write cadence.yaml to disk. The worker adopts the new rhythms on \
                 its next sweep — no restart needed. Typed, it is \
                 `/config cadence <text…>`.",
            )
            .clicked()
        {
            config.cadence_status = describe_saved(config.cadence_editor.apply(&config.io));
        }
        if ui.button("Reload").on_hover_text(RELOAD_HINT).clicked() {
            config.cadence_status = reload_status(config.cadence_editor.reload(&config.io));
        }
    });
    status_line(ui, &config.cadence_status);
}
