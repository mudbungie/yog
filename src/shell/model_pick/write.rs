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
        workspace: model.snap.ws_name(ws),
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

/// Fire the §9.4 drift clause's **keeping** exit (bl-2d19) and paint what came
/// back, on the same status line the pick above it writes: one sentence per
/// gesture, in one place, whichever of the two the operator spent.
///
/// The memos are deliberately **not** dropped here. A retarget writes a ref
/// mark and nothing else — the conversation's own executor lands it at its next
/// step — so the freeze this row states is still true at this instant, and
/// re-deriving it would only re-read the same two oids. The clause goes when the
/// branch moves, which is a snapshot change the memo key already answers.
pub(super) fn retarget(
    picker: &mut PickerState,
    model: &mut AppModel,
    ws: &Path,
    clis: (&Cli, &Cli),
    agent: &str,
) {
    let deps = model.boundary_deps(clis.0, clis.1);
    let action = Action::Retarget {
        workspace: model.snap.ws_name(ws),
        agent: agent.to_owned(),
    };
    picker.status = match model.dispatch(&deps, &crate::shell::now_ts(), &action) {
        // lernie's own confirmation is on stderr (the mark is a ref the operator
        // did not name, taking effect at a moment they did not choose), so the
        // receipt says when it lands rather than that it happened.
        Ok(Reply::Outcome(outcome)) if outcome.ok() => {
            format!("{agent} moves onto the current config at its next step")
        }
        Ok(Reply::Outcome(outcome)) => format!(
            "⚠ lernie retarget: {} · {}",
            crate::opslog::exit::ExitKind::of(outcome.exit, "lernie").label(),
            outcome.stderr
        ),
        Ok(other) => format!("⚠ unexpected reply: {other:?}"),
        Err(e) => format!("⚠ {e}"),
    };
}
