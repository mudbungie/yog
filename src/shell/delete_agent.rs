//! The §3.6 agent-delete surface (bl-f17a): the per-conversation confirmation
//! dialog and its two §11 carriers' shared entry.
//!
//! Coverage-excluded glue like the rest of `shell/*`: the confirmation, the
//! census parse, the dispatch gate and the prune are the tested
//! [`crate::delete::agent`] / [`crate::boundary`] surface; this file paints them.
//!
//! **The delete is posted, not run** (REMOTE §9.8, bl-1747), exactly as its
//! workspace-wide twin is (`super::delete`): the dialog holds a [`Ticket`]
//! beside the sentence it already held, says so while the engine has not
//! answered, closes on a clean receipt and keeps the reason otherwise. The gate
//! is unmoved — re-derived at fire time inside the chokepoint, fail-closed —
//! and the `ui.json` prune is the engine's write, adopted back by §7.1.
//!
//! **One dialog, two doors.** The visible carrier is [`danger_row`] — the
//! worded, ichor `delete this conversation…` row at the foot of the
//! inspector's Config tab (the per-conversation settings surface, mirroring
//! the workspace verb's config-mode danger row). The accelerator is the
//! conversation row's context menu (`super::menus`). Both call [`open`], so
//! the menu reaches the *dialog*, never past it (§11 doctrine), and **no key
//! opens it or arms it** (§3.6). Escape dismisses it (`super::modal`).
//!
//! **The dialog enumerates from the substrate's own census** — `lernie delete
//! --children --dry-run`, fetched once at open ([`open`]): the descendants by
//! name and the pending-deposit count come off lernie's `DeleteReport`, never
//! a yog re-derivation. The arming scales with the blast radius (§3.6 as
//! amended): a leaf takes a plain explicit confirm — the row the operator is
//! already pointing at, its name in the title — while a subtree takes the
//! typed name, which is also the only thing that fires `--children`.

use crate::AppModel;
use crate::boundary::Action;
use crate::cli_outbound::Cli;
use crate::delete::agent::{AgentConfirmation, Census};
use crate::theme;
use crate::wire::post::Ticket;
use std::path::{Path, PathBuf};

use super::ShellState;

/// The dialog's RAM (§5.3's unsubmitted-input carve-out): the conversation
/// under confirmation, the census fetched at open, the typed arming name and
/// the last refusal to render. Nothing here is durable.
#[derive(Default)]
pub struct DeleteAgentState {
    pub target: Option<(PathBuf, String)>,
    pub census: Option<Census>,
    pub typed: String,
    pub error: String,
    /// The posted delete this dialog is waiting on (REMOTE §9.8): `Some`
    /// disarms the button and marks the line, and the receipt it names either
    /// closes the dialog or leaves its reason here.
    pub ticket: Option<Ticket>,
}

/// Open the confirmation on `ws`/`root` — the one entry **both** carriers
/// call. The census spawn is the open's own cost (a short dry run, one
/// explicit gesture — never per frame).
pub(super) fn open(state: &mut ShellState, lernie: &Cli, ws: &Path, root: &str) {
    let (census, error) = match crate::delete::agent::census(lernie, ws, root) {
        Ok(census) => (Some(census), String::new()),
        Err(e) => (None, e),
    };
    state.delete_agent = DeleteAgentState {
        target: Some((ws.to_path_buf(), root.to_owned())),
        census,
        error,
        ..DeleteAgentState::default()
    };
}

/// The visible carrier (§11 roster, conversation-row seat): the worded, ichor
/// row at the foot of the inspector's Config tab, aimed at the focused
/// agent's conversation root. Rendered only where the verb exists — a
/// yog-named workspace (§3.6 scope), exactly what a `None` confirmation says.
pub(super) fn danger_row(
    ui: &mut egui::Ui,
    model: &mut AppModel,
    state: &mut ShellState,
    lernie: &Cli,
    ws: &Path,
) {
    // The §2.3 descent root, off the seat's own view of its selection
    // (REMOTE §9.4, bl-1eb0) — the gate is the conversation's, not the member's.
    // Since bl-48ae that view is a selection out of the landed forest, so the
    // row this click arms is aimed at the conversation the operator is looking
    // at rather than at whichever one an answer had caught up to.
    let Some(root) = super::seat::selection(model).map(|seat| seat.root) else {
        return;
    };
    if model.agent_delete_confirmation(ws, &root).is_none() {
        return;
    }
    ui.separator();
    if ui
        .button(egui::RichText::new("delete this conversation…").color(theme::ICHOR))
        .on_hover_text(
            "removes the agent, its children and their pending inbox — irrecoverable. \
             Typed, it is `/delete-agent [typed name]`.",
        )
        .clicked()
    {
        open(state, lernie, ws, &root);
    }
}

/// The confirmation window. The gate is re-derived from the model every frame,
/// so a driver that wakes while the dialog sits open re-arms the refusal.
pub(super) fn dialog(ctx: &egui::Context, model: &mut AppModel, state: &mut ShellState) {
    if state.delete_agent.target.is_none() {
        return;
    }
    settle(model, state);
    window(ctx, model, state);
    // Dismissed by any door — the ✕, a clean removal, or the workspace
    // vanishing — hands the keyboard back to the composer (§11).
    if state.delete_agent.target.is_none() {
        super::focus::request(state);
    }
}

/// The window itself, painted while a target stands.
fn window(ctx: &egui::Context, model: &mut AppModel, state: &mut ShellState) {
    let Some((ws, root)) = state.delete_agent.target.clone() else {
        return;
    };
    // The workspace left the named roster (deleted here or elsewhere): the
    // dialog has no subject left, so it closes rather than naming a ghost.
    let Some(confirm) = model.agent_delete_confirmation(&ws, &root) else {
        state.delete_agent = DeleteAgentState::default();
        return;
    };
    let mut shown = true;
    egui::Window::new(format!("delete conversation {}", confirm.name))
        .collapsible(false)
        .resizable(false)
        .open(&mut shown)
        .show(ctx, |ui| body(ui, model, state, &confirm));
    if !shown {
        state.delete_agent = DeleteAgentState::default();
    }
}

/// The dialog body: what dies (the substrate's census), then the refusal or
/// the blast-radius-scaled arming (§3.6 as amended). The target rides in the
/// state that opened the dialog, not as extra parameters.
fn body(
    ui: &mut egui::Ui,
    model: &mut AppModel,
    state: &mut ShellState,
    confirm: &AgentConfirmation,
) {
    ui.colored_label(
        theme::ICHOR,
        "this destroys the conversation — transcript, steps, worktree, marks — irrecoverably",
    );
    let Some(census) = state.delete_agent.census.clone() else {
        // The census dry run was declined: fail closed — no fire button, the
        // decline's own words below.
        ui.colored_label(theme::ICHOR, &state.delete_agent.error);
        return;
    };
    ui.label("children that die with it:");
    if census.descendants.is_empty() {
        ui.weak("(none)");
    }
    for id in &census.descendants {
        ui.weak(id);
    }
    ui.label(format!(
        "pending inbox deposits that die: {}",
        census.pending_deposits
    ));
    ui.separator();
    if confirm.refused() {
        ui.colored_label(
            theme::ICHOR,
            format!("live: {} — stop them first", confirm.live.join(", ")),
        );
        return;
    }
    // The §3.6 arming, scaled to blast radius: typed-name iff the verb
    // destroys objects beyond the one named on screen.
    let confirmed = if census.descendants.is_empty() {
        true
    } else {
        ui.horizontal(|ui| {
            ui.label(format!("type “{}” to confirm:", confirm.name));
            ui.text_edit_singleline(&mut state.delete_agent.typed)
                .on_hover_text(
                    "Retype the conversation's name exactly — deleting its children \
                     too is confirmed by the name, not a checkbox. It is the name \
                     `/delete-agent [typed name]` carries.",
                );
        });
        confirm.subtree_armed(&state.delete_agent.typed)
    };
    // Disarmed while one is outstanding (§9.8 ruling 2), for `super::delete`'s
    // reason: a second press would post a second delete of what the first one
    // is already removing. It is the *button* that disarms and never the form
    // above it — a typed name that vanished mid-flight would read as the dialog
    // having forgotten what the operator confirmed.
    let armed = confirmed && state.delete_agent.ticket.is_none();
    if ui
        .add_enabled(armed, egui::Button::new("Delete conversation"))
        .on_hover_text(
            "Remove this conversation from the workspace, permanently. Typed, it is \
             `/delete-agent [typed name]`.",
        )
        .on_disabled_hover_text("Type the conversation's name above to enable this.")
        .clicked()
    {
        fire(model, state);
    }
    if state.delete_agent.ticket.is_some() {
        ui.weak("removing the conversation …");
    }
    if !state.delete_agent.error.is_empty() {
        ui.colored_label(theme::ICHOR, &state.delete_agent.error);
    }
}

/// Post the delete (REMOTE §1.2) and mark the dialog as waiting. `typed` rides
/// the gesture unchanged: it is what arms the `--children` form, and the
/// executor re-derives its own gate rather than trusting this dialog.
fn fire(model: &mut AppModel, state: &mut ShellState) {
    let Some((ws, root)) = state.delete_agent.target.clone() else {
        return;
    };
    let action = Action::DeleteAgent {
        workspace: model.snap.ws_name(&ws),
        agent: root,
        typed: state.delete_agent.typed.clone(),
    };
    state.delete_agent.ticket = Some(model.post_act(&action));
}

/// Fold the delete's receipt: the dialog closes on a clean removal and keeps
/// the refusal otherwise (the trail's own record is the `ops.jsonl` line either
/// way — a declined `lernie delete` rides back as the executor's non-zero
/// outcome, which [`super::act::trouble`] spells).
fn settle(model: &mut AppModel, state: &mut ShellState) {
    let (Some(ticket), Some((ws, root))) =
        (state.delete_agent.ticket, state.delete_agent.target.clone())
    else {
        return;
    };
    let Some(landed) = model.act_receipt(ticket) else {
        return;
    };
    state.delete_agent.ticket = None;
    model.deleted_agent(&ws, &root);
    match super::act::trouble(&landed) {
        Some(reason) => state.delete_agent.error = reason,
        None => state.delete_agent = DeleteAgentState::default(),
    }
}
