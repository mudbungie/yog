//! The §9.5 control renderer: one [`Row`] → one widget of its [`Control`]'s
//! kind, plus [`raw_editor`] — the fallback control, whose subject is a whole
//! file rather than one setting. Coverage-excluded glue like the rest of
//! `src/shell/*` — which control a setting gets, what it shows, what it faults
//! on and what it writes back all live in the tested
//! [`crate::config_edit::form`]; this file only binds them to egui.
//!
//! **The pane holds no copy of any setting.** egui is immediate-mode, so each
//! widget binds a *local* copy of the row's value for the duration of the
//! frame and a change is handed straight back to the caller, which rewrites the
//! draft text. There is nowhere for a control's value to drift from the file.

use crate::config_edit::form::{Control, Group, Row};
use crate::theme;

/// How tall a raw editor opens. One number for all three raw surfaces (§9.1
/// brazen, §9.2 lernie-global, §9.3 config-branch): the fallback is one thing,
/// so it looks like one thing wherever it is folded open.
const RAW_ROWS: usize = 6;

/// Why a file has no controls, in the operator's words — the §9.5 fallback
/// announcing itself. Two seats say it (the lernie-global pane and the
/// config-branch pane) and it is one fact, so it is one sentence.
pub(super) const NO_READER: &str =
    "yog has no reader for this file's shape — edit it as raw text below";

/// The §9.5 raw fallback, in its one shape: a code editor over the whole file,
/// **as wide as the pane it is in**. egui's default `text_edit_width` is a fixed
/// 280 px column, which wrapped nearly every TOML/YAML line while the config
/// pane had two to three times that free (bl-2622) — hiding the very text the
/// fallback exists to expose. `f32::INFINITY` is egui's idiom for "take the
/// available width": the widget clamps it to `ui.available_width()`.
pub(super) fn raw_editor(ui: &mut egui::Ui, text: &mut String) {
    ui.add(
        egui::TextEdit::multiline(text)
            .desired_rows(RAW_ROWS)
            .desired_width(f32::INFINITY)
            .code_editor(),
    )
    .on_hover_text(
        "The file's whole contents, exactly as they will be written. Editing \
         here changes nothing until the pane's own Apply/Send commits it. No key of \
         its own: Tab reaches it, then type — inside it Tab indents.",
    );
}

/// The §8.5 spelling every control on this form shares (§11 rule 3): a setting
/// is one line of a config file, and a config file is one line at the composer.
/// Worn by each row's control, so it is a named phrase here rather than five
/// copies (rule 4).
const TYPED: &str = "Typed, the whole file is `/config <destination> <text…>`.";

/// Render every group's controls and return the one edit made this frame, as
/// `(row, new value)` — `None` on every frame nothing was touched, so a repaint
/// never re-writes what a gesture already wrote.
pub(super) fn render(
    ui: &mut egui::Ui,
    salt: &str,
    groups: &[Group],
    provider_rows: &[String],
) -> Option<(Row, String)> {
    let mut edit = None;
    for group in groups {
        ui.label(egui::RichText::new(&group.entry).strong());
        ui.indent(format!("{salt}-{}", group.entry), |ui| {
            for row in &group.rows {
                if let Some(value) = control(ui, salt, row, provider_rows) {
                    edit = Some((row.clone(), value));
                }
            }
        });
    }
    edit
}

/// One labelled control, plus the fault glyph when the stored value is not
/// usable (§9.5: judged at the point it is read, not at Apply).
fn control(ui: &mut egui::Ui, salt: &str, row: &Row, provider_rows: &[String]) -> Option<String> {
    let mut out = None;
    ui.horizontal(|ui| {
        ui.label(row.field).on_hover_text(row.help);
        // The value takes what the row has left, less the seat the fault glyph
        // is pinned into (bl-76f8, §11 rule 1 read for a form row): laid the
        // other way round the greedy box consumes the full width and the mark
        // after it lands outside the pane. A number needs no share — a bounded
        // integer is as wide as its digits, and stretching one across 2300 pt
        // would be the dead field G4 names, not the cure for it.
        let trailing = row.fault.as_ref().map_or(0.0, |_| FAULT_SEAT);
        let width = crate::layout::value_width(ui.available_width(), trailing);
        out = match row.control {
            Control::Provider => provider(ui, salt, row, provider_rows, width),
            Control::Number { min, max } => number(ui, row, min, max),
            Control::List | Control::Text => scalar(ui, row, width),
        };
        if let Some(fault) = &row.fault {
            ui.colored_label(theme::ICHOR, "⚠").on_hover_text(fault);
        }
    });
    out
}

/// The room the trailing fault glyph is pinned into — the mark plus the
/// spacing before it, reserved out of the value's share so a faulted row and a
/// clean one lay their boxes to the same right edge (§11 grid, QUALITY G3).
const FAULT_SEAT: f32 = 24.0;

/// A provider reference: brazen's live table, so nothing unroutable is
/// offerable. The current value is listed even when brazen lacks it, or
/// selecting anything would first silently retarget the row.
fn provider(
    ui: &mut egui::Ui,
    salt: &str,
    row: &Row,
    provider_rows: &[String],
    width: f32,
) -> Option<String> {
    let mut chosen = row.value.clone();
    let says = format!("{} {TYPED}", row.help);
    egui::ComboBox::from_id_salt(format!("{salt}-{}-{}", row.entry, row.field))
        .width(width)
        .selected_text(&row.value)
        .show_ui(ui, |ui| {
            if !provider_rows.contains(&row.value) {
                ui.selectable_value(&mut chosen, row.value.clone(), &row.value)
                    .on_hover_text(says.clone());
            }
            for name in provider_rows {
                ui.selectable_value(&mut chosen, name.clone(), name)
                    .on_hover_text(says.clone());
            }
        })
        .response
        .on_hover_text(says);
    (chosen != row.value).then_some(chosen)
}

/// A bounded whole number. The range is the setting's, so a value outside it
/// cannot be entered — only read back faulted from a file edited elsewhere.
fn number(ui: &mut egui::Ui, row: &Row, min: u64, max: u64) -> Option<String> {
    let mut n: u64 = row.value.parse().unwrap_or(min);
    ui.add(egui::DragValue::new(&mut n).range(min..=max))
        .on_hover_text(format!("{} {TYPED}", row.help))
        .changed()
        .then(|| n.to_string())
}

/// A single scalar or flow-list value — scoped to the one setting, never the
/// file. A list shows its member names; the grammar re-emits the brackets.
fn scalar(ui: &mut egui::Ui, row: &Row, width: f32) -> Option<String> {
    let mut value = row.value.clone();
    ui.add(egui::TextEdit::singleline(&mut value).desired_width(width))
        .on_hover_text(format!("{} {TYPED}", row.help))
        .changed()
        .then_some(value)
}

#[cfg(test)]
mod tests;
