//! The §3.6 workspace-deletion surface: the typed-name confirmation dialog and
//! the two §11 carriers' shared entry into it.
//!
//! Coverage-excluded glue like the rest of `shell/*`: the confirmation, the
//! gate, the plan and the executor are the tested [`crate::delete`] /
//! [`crate::boundary`] surface; this file paints them.
//!
//! **The unmaking is posted, not run** (REMOTE §9.8, bl-1747). It was the last
//! §3.6 act still dispatched in process, because its answer closed the dialog
//! in the same frame. Now the dialog holds a [`Ticket`] beside the sentence it
//! already held: it says so while the engine has not answered, closes on a
//! clean receipt and keeps the reason otherwise. The gate is unmoved — it is
//! re-derived at fire time and fail-closed inside the chokepoint, whichever
//! frontend fires — and the `ui.json` prune is the **engine's** write now,
//! adopted back by §7.1 like any external change rather than made against this
//! window's own copy.
//!
//! **One dialog, two doors.** The visible carrier is [`danger_row`] — the
//! worded, ichor `delete this workspace…` row on config mode's per-workspace
//! surface (§9.3, the settings-danger-zone convention). The accelerator is the
//! workspace tab's context menu (`super::menus`). Both call [`open`], so the
//! menu reaches the *dialog*, never past it (§11 doctrine), and **no key opens
//! it or arms it** — §3.6: a destructive verb takes no binding, ever. The one
//! key it answers is Escape, which *dismisses* it (`super::modal`, bl-d921):
//! backing out of a destructive dialog is the opposite of firing one, and every
//! modal owes the operator that door (§11).

use crate::AppModel;
use crate::boundary::Action;
use crate::delete::Confirmation;
use crate::theme;
use crate::wire::post::Ticket;
use std::path::{Path, PathBuf};

use super::ShellState;

/// The dialog's RAM (§5.3's unsubmitted-input carve-out): which workspace is
/// being confirmed, the typed arming name, and the last refusal to render.
/// Nothing here is durable — closing the window forgets all of it.
#[derive(Default)]
pub struct DeleteState {
    pub target: Option<PathBuf>,
    pub typed: String,
    pub error: String,
    /// The posted unmaking this dialog is waiting on (REMOTE §9.8): `Some`
    /// disarms the button and marks the line, and the receipt it names either
    /// closes the dialog or leaves its reason here.
    pub ticket: Option<Ticket>,
}

/// Open the §3.6 confirmation on `ws` — the one entry **both** carriers call.
pub(super) fn open(state: &mut ShellState, ws: &Path) {
    state.delete = DeleteState {
        target: Some(ws.to_path_buf()),
        ..DeleteState::default()
    };
}

/// The visible carrier (§11 roster, workspace-tab row): the worded, ichor
/// `delete this workspace…` row at the foot of config mode's per-workspace
/// surface. Rendered only where the verb exists — a yog-named focused workspace
/// (§3.6 scope), which is exactly what a `None` confirmation says.
pub(super) fn danger_row(ui: &mut egui::Ui, model: &AppModel, state: &mut ShellState) {
    let Some(ws) = model.focused_workspace().map(Path::to_path_buf) else {
        return;
    };
    if model.delete_confirmation(&ws).is_none() {
        return;
    }
    if ui
        .button(egui::RichText::new("delete this workspace…").color(theme::ICHOR))
        .on_hover_text(
            "takes the sphere wall down — irrecoverable. Typed, it is \
             `/delete-workspace <typed name>`.",
        )
        .clicked()
    {
        open(state, &ws);
    }
}

/// The typed-name confirmation window (§3.6). Re-derived every frame from the
/// model, so a driver that wakes while it sits open re-arms the refusal.
pub(super) fn dialog(ctx: &egui::Context, model: &mut AppModel, state: &mut ShellState) {
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
        super::focus::request(state);
    }
}

/// The window itself, painted while a target stands.
fn window(ctx: &egui::Context, model: &mut AppModel, state: &mut ShellState) {
    let Some(ws) = state.delete.target.clone() else {
        return;
    };
    // The workspace vanished (deleted here, or by the other instance): the
    // dialog has no subject left, so it closes rather than naming a ghost.
    let Some(confirm) = model.delete_confirmation(&ws) else {
        state.delete = DeleteState::default();
        return;
    };
    let mut shown = true;
    egui::Window::new(format!("delete workspace {}", confirm.name))
        .collapsible(false)
        .resizable(false)
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
    ui.label("conversations that die:");
    enumerate(ui, &confirm.conversations, "(none)");
    ui.label("balls released:");
    enumerate(ui, &confirm.ball_ids(), "(none)");
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
    if let Some(reason) = super::act::trouble(&landed) {
        state.delete.error = reason;
        return;
    }
    // The sphere's settings die with the sphere (§16.2 removes its wall
    // directory); its RAM dies on the same terms, or a workspace created later
    // under the same §3.1 name would inherit this one's box.
    state.forget_wall(&ws);
    state.delete = DeleteState::default();
}
