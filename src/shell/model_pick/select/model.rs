//! **The model half of the pair** (§9.4), split off [`super`] at §12's budget
//! on the seam the two dropdowns already stand on: a provider click *re-scopes*
//! and writes nothing, while a model click **is** the write (bl-fb6b). So the
//! provider list, which only ever changes what is asked, stays in `super`
//! beside the row that lays them out, and everything that can commit an
//! assignment is here — the list over the chosen row's live roster, the click
//! that writes, and the free-entry id that does not.
//!
//! The escape at the list's foot is the one thing here that commits on
//! **confirm** rather than on selection: a half-typed id is not a choice.

use super::PickerState;

/// The model list's last entry: a free-entry id, for a model brazen does not
/// list (a preview, a local tag). The row is still brazen's, so this can
/// declare an unserved model but never an unroutable one.
pub(super) const CUSTOM_MODEL: &str = "custom model id…";

/// The free-entry model id field (§9.4).
const CUSTOM_ID_HINT: &str = "Type the model id exactly as the provider names it, then press Enter — a \
     half-typed id is not a choice, so this one field commits on confirm \
     rather than as you type. The whole gesture is one line: \
     `/model <role> <provider> <model-id>`.";

/// The model dropdown, in operator terms (§9.4).
const MODEL_HINT: &str = "Which model this role runs on — the ids the chosen provider reports. \
     Picking one IS the write: it advances the workspace's config branch at \
     once, for the next conversation. Typed, it is the last word of \
     `/model <role> <provider> <model-id>`. The whole picker — other roles, a \
     custom id, a provider to add — is `m`.";

/// The model dropdown over the selected row's live roster, plus
/// [`CUSTOM_MODEL`]. Returns whether the list is **open** — the roster is fired
/// off that (bl-cd2a), so a conversation you are only looking at spawns nothing,
/// and the first frame of an open paints the pulse where the ids will land.
pub(super) fn model_combo(
    ui: &mut egui::Ui,
    selected: &str,
    candidates: &[String],
    in_flight: bool,
    chosen: &mut String,
) -> bool {
    // The closure runs only while the popup is open, so setting the flag inside
    // it IS the "is the list open" question — and the hover stays on the
    // constructor's own chain, where §11's discoverability scan reads it.
    let mut open = false;
    egui::ComboBox::from_id_salt("model-pick-model")
        .selected_text(selected)
        .show_ui(ui, |ui| {
            open = true;
            if in_flight || candidates.is_empty() {
                let time = ui.ctx().input(|i| i.time);
                ui.colored_label(
                    crate::theme::pulse(crate::theme::SPECTRE, time),
                    "⟳ asking the provider for its models…",
                );
                ui.ctx()
                    .request_repaint_after(crate::theme::PULSE_REPAINT_DELAY);
            }
            for id in candidates {
                ui.selectable_value(chosen, id.clone(), id)
                    .on_hover_text(MODEL_HINT);
            }
            ui.selectable_value(chosen, CUSTOM_MODEL.to_string(), CUSTOM_MODEL)
                .on_hover_text(
                    "Reveal a field for typing a model id the provider did not \
                     list — a preview, or a locally served tag. Typed, any id is \
                     the last word of `/model <role> <provider> <model-id>`.",
                );
        })
        .response
        .on_hover_text(MODEL_HINT);
    open
}

/// Turn the model list's answer into the write (bl-fb6b), or into the free-entry
/// field's own state. `CUSTOM_MODEL` commits nothing by itself: it reveals
/// [`custom_entry`], which commits on confirm.
pub(super) fn commit(
    ui: &mut egui::Ui,
    picker: &mut PickerState,
    shown: &str,
    chosen: String,
) -> Option<String> {
    if chosen != shown {
        if chosen == CUSTOM_MODEL {
            picker.custom = Some(String::new());
            return None;
        }
        picker.model = Some(chosen.clone());
        picker.custom = None;
        return Some(chosen);
    }
    custom_entry(ui, picker)
}

/// The free-entry id field, visible only while [`CUSTOM_MODEL`] is the
/// selection. It commits on **confirm** — Enter, or focus leaving the field
/// with something in it — never per keystroke: writing on the keystroke would
/// declare `g`, `gp`, `gpt`… each one a `models.yaml` entry and a `lernie
/// config` commit. A confirmed id becomes the dropdown's selection, which
/// retires the field and makes a second confirm of the same id impossible.
fn custom_entry(ui: &mut egui::Ui, picker: &mut PickerState) -> Option<String> {
    let typed = picker.custom.as_mut()?;
    let response = ui
        .horizontal(|ui| {
            ui.label("id").on_hover_text(CUSTOM_ID_HINT);
            ui.text_edit_singleline(typed).on_hover_text(CUSTOM_ID_HINT)
        })
        .inner;
    let id = typed.trim().to_string();
    if !response.lost_focus() || id.is_empty() {
        return None;
    }
    picker.model = Some(id.clone());
    picker.custom = None;
    Some(id)
}
