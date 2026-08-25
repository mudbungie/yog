//! The §3.6 confirmation **window** itself, split off [`super`] at §12's budget
//! on the doors-versus-dialog seam that file's own doc states: `super` holds the
//! RAM and the two carriers that open it (the config-mode danger row and the
//! workspace tab's menu), and this holds what they open — the frame's re-derived
//! census, the typed-name arming, the posted unmaking and the receipt that
//! closes the dialog or keeps its reason.
//!
//! Nothing here decides anything: the gate is re-derived fail-closed inside the
//! chokepoint at fire time whichever frontend fires (REMOTE §9.8), so every
//! answer painted here is an affordance and may be an ask period behind.

use crate::AppModel;
use crate::boundary::Action;
use crate::delete::Confirmation;
use crate::theme;

use super::{DeleteState, ShellState, census_room};

/// The typed-name confirmation window (§3.6). Re-derived every frame from the
/// model, so a driver that wakes while it sits open re-arms the refusal.
pub(crate) fn dialog(ctx: &egui::Context, model: &mut AppModel, state: &mut ShellState) {
    if state.delete.target.is_none() {
        return;
    }
    settle(model, state);
    window(ctx, model, state);
    // Dismissed by *any* of its three doors — the ✕, a clean unmaking, or the
    // subject vanishing under us — hands the keyboard back to the composer
    // (§11 focus discipline). Read as one edge here rather than restated at
    // each door, so a fourth door could not forget it.
    if state.delete.target.is_none() {
        crate::shell::focus::request(state);
    }
}

/// The window itself, painted while a target stands.
fn window(ctx: &egui::Context, model: &mut AppModel, state: &mut ShellState) {
    let Some(ws) = state.delete.target.clone() else {
        return;
    };
    // The workspace vanished (deleted here, or by the other instance): the
    // dialog has no subject left, so it closes rather than naming a ghost.
    //
    // **Folded off three landed answers** (REMOTE §9.7, bl-b4b5): the
    // enumeration says whether this is one of yog's own, the forest says what
    // dies and what is live, the balls listing says what is released. The
    // engine re-derives all of it fail-closed at fire (§9.8), so this copy is
    // the painted affordance and may be an ask period behind.
    let name = model.snap.ws_name(&ws);
    if !crate::nav::tabs::is_named(&crate::shell::chrome::ws_rows(model), &name) {
        state.delete = DeleteState::default();
        return;
    }
    let rows = crate::shell::convs::of(model, name.clone())
        .value
        .unwrap_or_default();
    let balls = crate::shell::chrome::balls(model, &name)
        .value
        .unwrap_or_default();
    let confirm = crate::delete::confirmation_of_rows(&name, &rows, &balls);
    let mut shown = true;
    egui::Window::new(format!("delete workspace {}", confirm.name))
        .collapsible(false)
        .resizable(false)
        // **A modal owns the frame, so it opens in the middle of one** (§11,
        // bl-d921; seated by bl-86a5). Unanchored, egui gives a new window an
        // *automatic cascade* position derived from the areas already on
        // screen — measured at 2560x1700 that put this dialog's title 1190 pt
        // down, with its own fire button below the bottom edge, and its
        // `constrain` then walked it back up one step per frame while the
        // operator watched. A destructive confirmation may not be reachable
        // only after it has finished crawling.
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .open(&mut shown)
        .show(ctx, |ui| body(ui, model, state, &confirm));
    if !shown {
        state.delete = DeleteState::default();
    }
}

/// The dialog body: what dies, what is released, and either the refusal or the
/// typed-name arming (§3.6 — the dialog *states* irrecoverability, since no
/// archival verb exists to offer until `lernie bundle` lands, §8.3).
fn body(ui: &mut egui::Ui, model: &mut AppModel, state: &mut ShellState, confirm: &Confirmation) {
    ui.colored_label(
        theme::ICHOR,
        "this destroys the workspace and everything inside it — irrecoverably",
    );
    census_room(ui, "delete-workspace-census", &mut |ui| {
        ui.label("conversations that die:");
        enumerate(ui, &confirm.conversations, "(none)");
        ui.label("balls released:");
        enumerate(ui, &confirm.ball_ids(), "(none)");
    });
    ui.separator();
    if confirm.refused() {
        ui.colored_label(
            theme::ICHOR,
            format!("live: {} — stop them first", confirm.live.join(", ")),
        );
        return;
    }
    ui.horizontal(|ui| {
        ui.label(format!("type “{}” to confirm:", confirm.name));
        ui.text_edit_singleline(&mut state.delete.typed)
            .on_hover_text(
                "Type the workspace's name exactly as shown to arm the delete — the \
                 same name `/delete-workspace <typed name>` carries.",
            );
    });
    // Disarmed while one is outstanding (§9.8 ruling 2: the fire writes what a
    // clean landing means and marks it until the receipt lands) — a second
    // press would post a second unmaking of a workspace the first one is
    // already taking down.
    let armed = confirm.armed(&state.delete.typed) && state.delete.ticket.is_none();
    if ui
        .add_enabled(armed, egui::Button::new("Delete workspace"))
        .on_hover_text(
            "Destroy this workspace and every conversation listed above, and release \
             the balls it holds. There is no undo and no archive. Typed, it is \
             `/delete-workspace <typed name>`.",
        )
        .on_disabled_hover_text("type the workspace's name above to arm this")
        .clicked()
    {
        fire(model, state, &confirm.name);
    }
    if state.delete.ticket.is_some() {
        ui.weak("taking the wall down …");
    }
    if !state.delete.error.is_empty() {
        ui.colored_label(theme::ICHOR, &state.delete.error);
    }
}

/// One list section, or the empty-state word — the dialog names concretely.
fn enumerate(ui: &mut egui::Ui, items: &[String], empty: &str) {
    if items.is_empty() {
        ui.weak(empty);
    }
    for item in items {
        ui.weak(item);
    }
}

/// Post the unmaking (REMOTE §1.2) and mark the dialog as waiting. The `typed`
/// name rides the gesture: it is what arms the executor's own re-derived gate,
/// which is fail-closed and does not trust this dialog.
fn fire(model: &mut AppModel, state: &mut ShellState, name: &str) {
    let action = Action::DeleteWorkspace {
        workspace: name.to_owned(),
        typed: state.delete.typed.clone(),
    };
    state.delete.ticket = Some(model.post_act(&action));
}

/// Fold the unmaking's receipt: the dialog closes on a clean removal and keeps
/// the refusal otherwise (the trail's own record is the `ops.jsonl` line either
/// way). The convergence runs on **both** arms, for the reason it always did —
/// the releases that did land are already real.
fn settle(model: &mut AppModel, state: &mut ShellState) {
    let (Some(ticket), Some(ws)) = (state.delete.ticket, state.delete.target.clone()) else {
        return;
    };
    let Some(landed) = model.act_receipt(ticket) else {
        return;
    };
    state.delete.ticket = None;
    model.deleted_workspace(&ws);
    if let Some(reason) = crate::shell::act::trouble(&landed) {
        state.delete.error = reason;
        return;
    }
    // The sphere's settings die with the sphere (§16.2 removes its wall
    // directory); its RAM dies on the same terms, or a workspace created later
    // under the same §3.1 name would inherit this one's box.
    state.forget_wall(&ws);
    state.delete = DeleteState::default();
}
