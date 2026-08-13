//! The picker's three choices (§9.4): which brazen provider row a role is
//! routed through, which model, and — while the pane is open — which role is
//! being changed at all. Coverage-excluded glue like the rest of `src/shell/*`;
//! the judgements are [`default_row`](crate::model_pick::default_row) and
//! [`plan`](crate::model_pick::plan)'s two refusals.
//!
//! **The two dropdowns ARE the settings row** (bl-cd2a): the whole line becomes
//! `<provider> - <model>` and nothing else. [`pair_row`] paints exactly that — two combo boxes and
//! the separator between them — so the pair is chosen where it is read, with no
//! sentence in front of it and no *change…* to press first.
//!
//! **Two dropdowns, sourced from brazen, dissolve the unknown-provider class
//! (bl-bd89).** Before them the roster was asked of the row the role was
//! *already* on, so a role stranded on a row brazen no longer has — `codex`,
//! after the operator renamed it to `openai-chatgpt` — got `unknown provider`
//! back and no candidates at all: a dead end at exactly the moment the picker
//! exists for. Now the provider is chosen from brazen's own effective table and
//! the models from that row's live roster, so an unroutable pair cannot be
//! expressed. Each list carries one escape at the bottom — a route to the §9.1
//! editor to add a row, and a free-entry id for a model brazen does not list —
//! so neither dropdown is itself a dead end.
//!
//! **Selection is the gesture (bl-fb6b).** There is no Set button: the model
//! dropdown's click is the write, scoped to whichever role the row reports
//! (`worker`, or whatever the pane's strip has re-scoped it to). The free-entry
//! id is the one thing that does not commit as it is chosen — it commits on
//! confirm, because a half-typed id is not a choice.

use super::{Marked, PickerState};
use crate::model_pick::{ModelRow, WORKER_ROLE, default_row};

/// The provider list's last entry: not a row, a route to the §9.1 brazen
/// `config.toml` editor, which is the one place a row is authored.
const ADD_PROVIDER: &str = "add a provider…";
/// The model list's last entry: a free-entry id, for a model brazen does not
/// list (a preview, a local tag). The row is still brazen's, so this can
/// declare an unserved model but never an unroutable one.
const CUSTOM_MODEL: &str = "custom model id…";

/// The provider dropdown, in operator terms (§9.4).
const PROVIDER_HINT: &str = "Which of brazen's providers this role is routed through. Choosing one \
     asks it for its models and refills the list beside it. Typed, the pick is \
     the middle word of `/model <role> <provider> <model-id>`.";

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

/// What the row asked for this frame. The widgets choose; the caller fires the
/// roster, writes the pick and routes the surface, so this file stays a set of
/// controls over [`PickerState`] and nothing else.
pub(super) struct PairChoice {
    /// The provider row the pair is currently scoped to — what the model list
    /// is asked of.
    pub(super) provider: String,
    /// The operator chose *add a provider…*: a route to the §9.1 editor, not a
    /// row.
    pub(super) add_provider: bool,
    /// A model id chosen **this frame** — a click in the list, or a custom id
    /// confirmed. `Some` on exactly the one frame, so a repaint never re-writes
    /// what a click already wrote.
    pub(super) chosen: Option<String>,
    /// The model list is open, so its candidates must be live: the caller fires
    /// the roster off this rather than on sight (bl-cd2a).
    pub(super) list_open: bool,
}

/// The settings row itself: `<provider> · <model>`, two dropdowns and the
/// separator between them. The pair shown is whatever the operator has chosen
/// this open, falling back to what the config branch tip assigns — so the row
/// reads as the assignment until it is used, and as the choice once it is.
///
/// A provider click **re-scopes; it does not write** (bl-fb6b). The id in hand
/// came from the previous row's roster, so carrying it over would commit a pair
/// the operator never chose; the model is dropped, the list beside it re-fires,
/// and the click there completes the pick.
pub(super) fn pair_row(
    ui: &mut egui::Ui,
    picker: &mut PickerState,
    row: &ModelRow,
    rows: &[String],
    candidates: &[String],
    in_flight: bool,
) -> PairChoice {
    let provider = picker
        .provider
        .clone()
        .unwrap_or_else(|| default_row(&row.provider, rows));
    let shown = if picker.custom.is_some() {
        CUSTOM_MODEL.to_string()
    } else {
        picker.model.clone().unwrap_or_else(|| row.model.clone())
    };
    let mut chosen_provider = provider.clone();
    let mut chosen_model = shown.clone();
    let list_open = ui
        .horizontal(|ui| {
            provider_combo(ui, &provider, rows, &mut chosen_provider);
            ui.weak("·").on_hover_text(&row.hover);
            model_combo(ui, &shown, candidates, in_flight, &mut chosen_model)
        })
        .inner;
    if chosen_provider == ADD_PROVIDER {
        return PairChoice {
            provider,
            add_provider: true,
            chosen: None,
            list_open,
        };
    }
    if chosen_provider != provider {
        picker.provider = Some(chosen_provider.clone());
        picker.model = None;
        picker.custom = None;
        return PairChoice {
            provider: chosen_provider,
            add_provider: false,
            chosen: None,
            list_open,
        };
    }
    PairChoice {
        provider,
        add_provider: false,
        chosen: commit(ui, picker, &shown, chosen_model),
        list_open,
    }
}

/// The provider dropdown over brazen's effective table, plus [`ADD_PROVIDER`].
/// The selection defaults to the assignment's own row while brazen has it and to
/// brazen's first row once brazen does not, so the row never asks a provider
/// that cannot answer; brazen unanswerable (an empty table) offers only the
/// route.
fn provider_combo(ui: &mut egui::Ui, selected: &str, rows: &[String], chosen: &mut String) {
    egui::ComboBox::from_id_salt("model-pick-provider")
        .selected_text(selected)
        .show_ui(ui, |ui| {
            for row in rows {
                ui.selectable_value(chosen, row.clone(), row)
                    .on_hover_text(PROVIDER_HINT);
            }
            ui.selectable_value(chosen, ADD_PROVIDER.to_string(), ADD_PROVIDER)
                .on_hover_text(
                    "Leave for brazen's config.toml editor, where a provider row is \
                     authored — this list only offers rows that already exist. That \
                     editor is `/config brazen <text…>`.",
                );
        })
        .response
        .on_hover_text(PROVIDER_HINT);
}

/// The model dropdown over the selected row's live roster, plus
/// [`CUSTOM_MODEL`]. Returns whether the list is **open** — the roster is fired
/// off that (bl-cd2a), so a conversation you are only looking at spawns nothing,
/// and the first frame of an open paints the pulse where the ids will land.
fn model_combo(
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
fn commit(
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

/// The role strip (§9.4: whatever roles the file declares, not a worker/compactor
/// special case), painted in the pane because a row cannot hold it. It defaults
/// to the role that talks to you; a role whose model is unusable carries the
/// warning glyph, its reason painted below. Choosing another role **re-scopes
/// the row's two dropdowns** onto that role's own assignment — the strip is the
/// scope, not an action (bl-fb6b), so there is no per-role apply.
pub(super) fn select_role(
    ui: &mut egui::Ui,
    picker: &mut PickerState,
    marked: &[Marked],
) -> String {
    let fallback = marked
        .iter()
        .find(|(r, _)| r.role == WORKER_ROLE)
        .or_else(|| marked.first())
        .map_or_else(String::new, |(r, _)| r.role.clone());
    let selected = picker.role.clone().unwrap_or(fallback);
    let mut chosen = selected.clone();
    ui.horizontal(|ui| {
        ui.label("role");
        for (role, fault) in marked {
            let mark = if fault.is_some() { " ⚠" } else { "" };
            let label = format!("{} · {}{mark}", role.role, role.model);
            if ui
                .selectable_label(role.role == selected, label)
                .on_hover_text(
                    "Choose which role the dropdowns above are changing — the model \
                     it runs on now is shown beside its name. Typed, it is the first \
                     word of `/model <role> <provider> <model-id>`.",
                )
                .clicked()
            {
                chosen.clone_from(&role.role);
            }
        }
    });
    if chosen != selected {
        picker.role = Some(chosen.clone());
        picker.forget_choice();
    }
    chosen
}
