//! The **role strip** (§9.4) — which role the pane's two dropdowns are
//! changing at all.
//!
//! Its own file beside [`super::select`] at §12's cap, on the seam that file's
//! own doc already draws: the pair row is a *row* (two dropdowns and a
//! separator, chosen where they are read), and this is the **scope** over it —
//! painted in the pane because a row cannot hold it, and choosing here writes
//! nothing (bl-fb6b). Coverage-excluded glue like the rest of `src/shell/*`.

use super::{Marked, PickerState};
use crate::model_pick::WORKER_ROLE;

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
