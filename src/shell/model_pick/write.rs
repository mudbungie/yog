//! The picker's write half (§9.4) — since bl-3f46 a single boundary gesture.
//!
//! The pick used to be composed here: plan both files, apply `models.yaml`
//! through the §9.2 pipeline, then stage `providers.yaml` and drive `lernie
//! config`. All of that is now the chokepoint's
//! [`PickModel`](crate::boundary::Action::PickModel) arm, so a click, a
//! `/model` line and a deposit are one implementation and the pane is what it
//! should be: the construction of a variant, and the sentence its reply earns.
//!
//! Coverage-excluded glue like the rest of `src/shell/*`; the gesture it fires
//! is tested in `boundary::config`.

use super::PickerState;
use crate::AppModel;
use crate::boundary::Action;
use crate::boundary::reply::Reply;
use crate::cli_outbound::Cli;
use crate::model_pick::Pick;
use std::path::Path;

/// Fire one pick and paint what came back. Every derived fact the pane holds is
/// dropped afterwards — the branch moved and `models.yaml` may have gained an
/// entry, so the next paint re-reads rather than showing the pre-write world.
pub(super) fn apply(
    picker: &mut PickerState,
    model: &mut AppModel,
    ws: &Path,
    clis: (&Cli, &Cli),
    pick: &Pick,
) {
    let deps = model.boundary_deps(clis.0, clis.1);
    let action = Action::PickModel {
        workspace: ws.to_path_buf(),
        role: pick.role.clone(),
        provider: pick.provider.clone(),
        model: pick.model.clone(),
    };
    picker.status = match model.dispatch(&deps, &crate::shell::now_ts(), &action) {
        // The receipt carries the whole pair, which is what the vanished Set
        // button's label used to say up front (bl-fb6b): the selection commits,
        // so the sentence stating what happened is the only one left to paint.
        Ok(Reply::Outcome(outcome)) if outcome.ok() => {
            format!("set {} → {} · {}", pick.role, pick.provider, pick.model)
        }
        // The exit in words, through the one projection (bl-afa9) — a bare `-1`
        // here read as a signal death rather than "ran, status not observable".
        Ok(Reply::Outcome(outcome)) => format!(
            "⚠ lernie config: {} · {}",
            crate::opslog::exit::ExitKind::of(outcome.exit, "lernie").label(),
            outcome.stderr
        ),
        Ok(other) => format!("⚠ unexpected reply: {other:?}"),
        Err(e) => format!("⚠ {e}"),
    };
    picker.tip_providers = None;
    picker.models_text = None;
    picker.frozen = None;
}
