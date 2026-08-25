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
//!
//! This file is the doors and the RAM they fill; what they open is
//! `delete/window`.

use crate::AppModel;
use crate::theme;
use crate::wire::post::Ticket;
use std::path::PathBuf;

use super::ShellState;

mod window;
pub(super) use window::dialog;

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
