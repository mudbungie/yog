//! The §11 `new` tab's one-field name form — where the operator names a sphere
//! wall (DESIGN §3.1 as amended at bl-df65, §3.4).
//!
//! Coverage-excluded glue like the rest of `shell/*`: the validation
//! ([`AppModel::validate_workspace_name`], §3.1) and the raise itself
//! ([`AppModel::new_workspace_inputs`] into `start::prepare`) are the tested
//! surface; this file paints one text box and reports what validation said.
//!
//! **A refusal is a sentence beside the field, never a wound.** Nothing has
//! spawned when a name is refused — no ops row, no banner: the operator retypes
//! (§3.1: "no suffixing, no prompt-loop"). The empty field is that same refusal
//! with nothing typed yet, so it only disables Create rather than shouting.
//!
//! **The form answers all three of a dialog's keys** (§11, bl-d921): Return
//! submits a name that validates, Escape dismisses (through `super::modal`,
//! which owns that verb for both dialogs), and nothing beneath the form is
//! reachable while it stands.

use crate::AppModel;
use crate::cli_outbound::Cli;
use crate::theme;

use super::ShellState;

/// The form's RAM (§5.3's unsubmitted-input carve-out): whether it is open and
/// the name being typed. Closing the window forgets both.
#[derive(Default)]
pub struct NewWsState {
    pub open: bool,
    pub typed: String,
}

/// Open the form — the one entry the `new` tab and the `w` / Ctrl+Shift+N
/// binding share (§11).
pub(super) fn open(state: &mut ShellState) {
    state.new_ws = NewWsState {
        open: true,
        typed: String::new(),
    };
}

/// The §11 name form, painted last over every panel like the §3.6 dialog.
/// Submit (the button or Enter in the box) raises the workspace through the one
/// planner and focuses it (§3.4).
pub(super) fn dialog(
    ctx: &egui::Context,
    model: &mut AppModel,
    state: &mut ShellState,
    lernie: &Cli,
    bl: &Cli,
) {
    if !state.new_ws.open {
        return;
    }
    let mut shown = true;
    egui::Window::new("new workspace")
        .collapsible(false)
        .resizable(false)
        .open(&mut shown)
        .show(ctx, |ui| body(ui, model, state, lernie, bl));
    if !shown {
        // Dismissed by the ✕ — the same verb Escape and the scrim spend
        // (`super::modal`, bl-d921): the draft dies and the keyboard goes back
        // to the message composer (§11 focus discipline). Create does not come
        // through here — it hands the keyboard over itself, at the end of the
        // raise (`super::start_pane::run_prepare`), once the composer it lands
        // in is aimed at the sphere just raised.
        super::modal::dismiss(state);
    }
}

/// The body: the invitation, the field, §3.1's verdict inline, and Create.
fn body(ui: &mut egui::Ui, model: &mut AppModel, state: &mut ShellState, lernie: &Cli, bl: &Cli) {
    ui.label("name this sphere — a client, an employer, personal vs. work:");
    let edit = ui
        .add(
            egui::TextEdit::singleline(&mut state.new_ws.typed)
                .desired_width(220.0)
                .hint_text("ops"),
        )
        .on_hover_text(
            "The name for this sphere of work. It must not be empty and must not \
             collide with a workspace you already have; the reason appears here if \
             it is refused. The form itself opens on (w) / Ctrl+Shift+N, and Enter \
             submits it.",
        );
    let verdict = model.validate_workspace_name(&state.new_ws.typed);
    // Enter submits (§3.1, bl-d921). Read here, **before** the focus claim
    // below, and that order is the whole bug it fixes: `lost_focus` is "had it
    // last frame and does not now", so the claim — which runs the instant the
    // box surrenders focus to the Enter — used to hand it straight back and
    // make this read `false`, every time. Enter was therefore unreachable and
    // the pointer on Create was the only way through.
    let entered = edit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
    let submitted = entered && verdict.is_ok();
    // The form is one field and a button, so it takes the keyboard the moment it
    // appears and the operator types with no click. **Claimed, not held**: an
    // unconditional per-frame `request_focus` made the form the one place
    // Ctrl+I could not reach — the §11 "jump to the composer from anywhere"
    // binding would set its request and this line would out-shout it, every
    // frame, forever. Asking only when the keyboard is unclaimed keeps the
    // grab-on-open and lets the operator leave — and a *refused* Enter comes
    // back through here, so a malformed name leaves the operator typing rather
    // than hunting for the box that just let go.
    if !submitted && ui.memory(|m| m.focused().is_none()) {
        edit.request_focus();
    }
    // The reason, verbatim from the refusal — but not while the box is still
    // empty, which is nothing typed rather than something wrong. "Empty" is
    // `names::normalize`'s reading, the same one validation and §3.6's arming use.
    if let Err(e) = &verdict
        && !crate::names::normalize(&state.new_ws.typed).is_empty()
    {
        ui.colored_label(theme::ICHOR, e.to_string());
    }
    let clicked = ui
        .add_enabled(verdict.is_ok(), egui::Button::new("Create workspace"))
        .on_hover_text(
            "Raise the wall and focus it: a separate world of conversations, config \
             and claimed balls that touches nothing in your other workspaces. Enter \
             does the same from the name box; Escape drops the form.",
        )
        .on_disabled_hover_text("the name above is empty or already taken")
        .clicked();
    if let Ok(name) = verdict
        && (clicked || submitted)
    {
        let inputs = model.new_workspace_inputs(&name);
        state.new_ws = NewWsState::default();
        super::start_pane::run_prepare(model, state, lernie, bl, inputs);
    }
}
