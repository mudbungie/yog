//! The picker's write half (§9.4) — since bl-3f46 a single boundary gesture,
//! and since bl-4841 a posted one.
//!
//! The pick used to be composed here: plan both files, apply `models.yaml`
//! through the §9.2 pipeline, then stage `providers.yaml` and drive `lernie
//! config`. All of that is now the chokepoint's
//! [`PickModel`](crate::boundary::Action::PickModel) arm, so a click, a
//! `/model` line and a deposit are one implementation and the pane is what it
//! should be: the construction of a variant, and the sentence its **receipt**
//! earns.
//!
//! **The receipt lands later** (REMOTE §9.8). The gesture crosses loopback mTLS
//! and a frame may not wait on a socket, so the click writes the sentence a
//! clean landing means and [`settle`] folds what actually came back — the same
//! line, its in-flight mark dropped, or the reason appended. The memos are
//! dropped at the *landing* rather than at the fire, because until the receipt
//! arrives the pre-write world is still the world.
//!
//! Coverage-excluded glue like the rest of `src/shell/*`; the gesture it posts
//! is tested in `boundary::config`.

use super::PickerState;
use crate::AppModel;
use crate::boundary::Action;
use crate::model_pick::Pick;
use std::path::Path;

/// Fire one pick. The sentence is what committing it means — the whole pair,
/// which is what the vanished Set button's label used to say up front (bl-fb6b)
/// — carried in flight and confirmed by the receipt.
pub(super) fn apply(picker: &mut PickerState, model: &mut AppModel, ws: &Path, pick: &Pick) {
    let action = Action::PickModel {
        workspace: model.snap.ws_name(ws),
        role: pick.role.clone(),
        provider: pick.provider.clone(),
        model: pick.model.clone(),
    };
    let said = format!("set {} → {} · {}", pick.role, pick.provider, pick.model);
    picker.act.fire(model, &action, &said);
}

/// Fire the §9.4 drift clause's **keeping** exit (bl-2d19), on the same line the
/// pick above it writes: one sentence per gesture, in one place, whichever of
/// the two the operator spent.
///
/// lernie's own confirmation is on stderr (the mark is a ref the operator did
/// not name, taking effect at a moment they did not choose), so the sentence
/// says when it lands rather than that it happened.
pub(super) fn retarget(picker: &mut PickerState, model: &mut AppModel, ws: &Path, agent: &str) {
    let action = Action::Retarget {
        workspace: model.snap.ws_name(ws),
        agent: agent.to_owned(),
    };
    let said = format!("{agent} moves onto the current config at its next step");
    picker.act.fire(model, &action, &said);
}

/// Fold whatever the picker's act earned, once, on the frame it arrives.
///
/// Every derived fact the pane holds is dropped **here**: the branch moved, so
/// the next paint re-reads rather than showing the pre-write world. A retarget
/// writes a ref mark and nothing
/// else — the conversation's own executor lands it at its next step — so the
/// freeze the drift row states is still true; re-deriving it costs two oid
/// reads and is what keeps this one fold rather than two.
pub(super) fn settle(picker: &mut PickerState, model: &mut AppModel) {
    let Some(landed) = picker.act.landed(model) else {
        return;
    };
    if let Some(why) = crate::shell::act::trouble(&landed) {
        let said = format!("{} — ⚠ {why}", picker.act.line());
        picker.act.say(said);
    }
    picker.tip_providers = None;
    picker.frozen = None;
}
