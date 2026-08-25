//! The picker's three choices (§9.4): which brazen provider row a role is
//! routed through, which model, and — while the pane is open — which role is
//! being changed at all. Coverage-excluded glue like the rest of `src/shell/*`;
//! the judgements are [`default_row`](crate::model_pick::default_row) and
//! [`plan`](crate::model_pick::plan)'s refusals.
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
//! **What is said above the pair is [`notes`]**, and one of its three sentences
//! is a caveat rather than a refusal (bl-671d): a dialect that declines tools is
//! shown unselectable because `plan` would reject the pick, while a dialect that
//! leaves the context size to the server stays selectable and is merely stated.
//!
//! **Selection is the gesture (bl-fb6b).** There is no Set button: the model
//! dropdown's click is the write, scoped to whichever role the row reports
//! (`worker`, or whatever the pane's strip has re-scoped it to). The free-entry
//! id is the one thing that does not commit as it is chosen — it commits on
//! confirm, because a half-typed id is not a choice. That is the seam this
//! file is split on: the provider list, which only re-scopes, is here, and
//! everything that can commit an assignment is `select/model`.

mod model;
mod notes;

use super::PickerState;
use crate::config_edit::brazen::ProviderRow;
use crate::model_pick::ModelRow;
use model::CUSTOM_MODEL;

/// The provider list's last entry: not a row, a route to the §9.1 brazen
/// `config.toml` editor, which is the one place a row is authored.
const ADD_PROVIDER: &str = "add a provider…";
/// The provider dropdown, in operator terms (§9.4).
const PROVIDER_HINT: &str = "Which of brazen's providers this role is routed through. Choosing one \
     asks it for its models and refills the list beside it. Typed, the pick is \
     the middle word of `/model <role> <provider> <model-id>`.";

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
    rows: &[ProviderRow],
    candidates: &[String],
    in_flight: bool,
) -> PairChoice {
    // Where the dropdown lands, and everything standing that is worth saying
    // about it ([`notes`]) — the strand it was steered off, a protocol that
    // declines tools, a dialect that leaves the context size to the server.
    let provider = notes::scoped_with_notes(ui, picker, row, rows);
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
            model::model_combo(ui, &shown, candidates, in_flight, &mut chosen_model)
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
        chosen: model::commit(ui, picker, &shown, chosen_model),
        list_open,
    }
}

/// The provider dropdown over brazen's effective table, plus [`ADD_PROVIDER`].
/// The selection defaults to the assignment's own row while brazen has it and to
/// brazen's first row once brazen does not, so the row never asks a provider
/// that cannot answer; brazen unanswerable (an empty table) offers only the
/// route.
///
/// **A row whose protocol declines tools is listed and not offered** (bl-3d22):
/// it is shown with the reason, unselectable, because *"nothing offered is
/// unroutable"* is the §9.4 invariant and omitting the row outright would leave
/// an operator reading `bz --list-providers` with no answer at all. The sentence
/// is the row's own
/// ([`ProviderRow::tools_blocked`](crate::config_edit::brazen::ProviderRow::tools_blocked)),
/// never a second phrasing beside `plan`'s refusal.
fn provider_combo(ui: &mut egui::Ui, selected: &str, rows: &[ProviderRow], chosen: &mut String) {
    egui::ComboBox::from_id_salt("model-pick-provider")
        .selected_text(selected)
        .show_ui(ui, |ui| {
            for row in rows {
                if let Some(why) = row.tools_blocked() {
                    ui.weak(format!("{} — {why}", row.name))
                        .on_hover_text(PROVIDER_HINT);
                    continue;
                }
                ui.selectable_value(chosen, row.name.clone(), &row.name)
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
