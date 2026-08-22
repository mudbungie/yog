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
use std::path::PathBuf;

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

/// Open the §3.6 confirmation on the workspace `name` addresses — the one
/// entry **both** carriers call, and one of bl-7407's **doors**: what a
/// pointer-targeted tab menu and the config-mode danger row both hand over is a
/// §3.1 name, and the dialog holds the path it resolves to for as long as it is
/// open. A name the enumeration does not answer opens nothing — there is no
/// workspace to unmake.
pub(super) fn open(model: &AppModel, state: &mut ShellState, name: &str) {
    state.delete = DeleteState {
        target: model.workspace_path(name),
        ..DeleteState::default()
    };
}

/// The visible carrier (§11 roster, workspace-tab row): the worded, ichor
/// `delete this workspace…` row at the foot of config mode's per-workspace
/// surface. Rendered only where the verb exists — a yog-named focused workspace
/// (§3.6 scope), which is exactly what a `None` confirmation says.
pub(super) fn danger_row(ui: &mut egui::Ui, model: &mut AppModel, state: &mut ShellState) {
    let Some(name) = model.focused_ws_name() else {
        return;
    };
    // The §3.6 scope, off the landed enumeration (bl-b4b5): the verb exists for
    // yog's own named workspaces and nowhere else, which is what a `named` row
    // says. A frame the engine has not answered offers nothing, which is the
    // collapsed-pane rule and never a delete offered on a guess.
    if !crate::nav::tabs::is_named(&super::chrome::ws_rows(model), &name) {
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
        open(model, state, &name);
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
    //
    // **Folded off three landed answers** (REMOTE §9.7, bl-b4b5): the
    // enumeration says whether this is one of yog's own, the forest says what
    // dies and what is live, the balls listing says what is released. The
    // engine re-derives all of it fail-closed at fire (§9.8), so this copy is
    // the painted affordance and may be an ask period behind.
    let name = model.snap.ws_name(&ws);
    if !crate::nav::tabs::is_named(&super::chrome::ws_rows(model), &name) {
        state.delete = DeleteState::default();
        return;
    }
    let rows = super::convs::of(model, name.clone())
        .value
        .unwrap_or_default();
    let balls = super::chrome::balls(model, &name).value.unwrap_or_default();
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

/// **A census scrolls in its own room** (§11 rule 6 as extended by bl-86a5) —
/// the one definition, shared with [`super::delete_agent`]'s dialog because
/// both have the same shape and the same defect.
///
/// §3.6 mandates the concrete enumeration and nothing bounds how long one is: a
/// dialog window is `resizable(false)` and sized by its content, so a wall with
/// enough conversations in it laid its own arming field and fire button past the
/// bottom of the screen, where they are clipped away and unreachable — a
/// destructive dialog that cannot be fired, and (worse) cannot be read before
/// firing. The enumeration is therefore a bounded viewport and everything the
/// operator must *act* on stays outside it, below, where the census cannot move
/// it. Half the screen is the ceiling every other sized surface divides
/// ([`crate::layout::panel_ceiling`]) — one home, so a dialog and a panel
/// cannot disagree about what half means.
pub(super) fn census_room(ui: &mut egui::Ui, salt: &str, rows: &mut dyn FnMut(&mut egui::Ui)) {
    let cap = crate::layout::panel_ceiling(ui.ctx().screen_rect().height());
    // **The cap is handed to the ui, not only to the scroll** — `shell::settings`
    // one door over, and for the same lock. A `ScrollArea` takes the *available*
    // size at most, and inside a window sized by its own content that is last
    // frame's height, so the two settle on each other wherever the first frame
    // happened to land: measured, the census took every point the window had
    // and the arming row below it was clipped away for good. A scope carrying
    // the cap breaks the loop — the region grows with its content up to half
    // the screen, then scrolls, and what is below it is seated against a height
    // that no longer depends on what was below it last frame.
    ui.scope(|ui| {
        ui.set_max_height(cap);
        egui::ScrollArea::vertical()
            .id_salt(salt)
            .max_height(cap)
            .show(ui, rows);
    });
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
