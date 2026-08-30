//! The §3.6 agent-delete **window** itself, split off [`super`] at §12's budget
//! on the doors-versus-dialog seam — the same seam `super::super::delete` is
//! split on, and for the same reason: `super` holds the RAM, the census fetched
//! at open and the two carriers that open it (the inspector's Config-tab danger
//! row and the conversation row's menu), and this holds what they open — the
//! re-derived confirmation, the blast-radius-scaled arming, the posted delete
//! and the receipt that closes the dialog or keeps its reason.
//!
//! Nothing here decides anything: the chokepoint re-derives the gate
//! fail-closed at fire time (REMOTE §9.8), so every answer painted here is an
//! affordance and may be an ask period behind.

use crate::AppModel;
use crate::boundary::Action;
use crate::delete::agent::AgentConfirmation;
use crate::theme;

use super::{DeleteAgentState, ShellState};

/// The confirmation window. The gate is re-derived from the model every frame,
/// so a driver that wakes while the dialog sits open re-arms the refusal.
pub(crate) fn dialog(ctx: &egui::Context, model: &mut AppModel, state: &mut ShellState) {
    if state.delete_agent.target.is_none() {
        return;
    }
    settle(model, state);
    window(ctx, model, state);
    // Dismissed by any door — the ✕, a clean removal, or the workspace
    // vanishing — hands the keyboard back to the composer (§11).
    if state.delete_agent.target.is_none() {
        crate::shell::focus::request(state);
    }
}

/// The window itself, painted while a target stands.
fn window(ctx: &egui::Context, model: &mut AppModel, state: &mut ShellState) {
    let Some((ws, root)) = state.delete_agent.target.clone() else {
        return;
    };
    // The workspace left the named roster (deleted here or elsewhere): the
    // dialog has no subject left, so it closes rather than naming a ghost.
    //
    // **Folded off two landed answers** (REMOTE §9.7, bl-b4b5): the enumeration
    // for the §3.6 scope, the descent forest for the name and the live members.
    // The chokepoint re-derives the gate fail-closed at fire (§9.8), so this
    // copy paints the affordance and never decides.
    let name = model.snap.ws_name(&ws);
    if !crate::nav::tabs::is_named(&crate::shell::chrome::ws_rows(model), &name) {
        state.delete_agent = DeleteAgentState::default();
        return;
    }
    let rows = crate::shell::convs::of(model, name)
        .value
        .unwrap_or_default();
    let confirm = crate::delete::agent::confirmation_of_rows(&rows, &root);
    let mut shown = true;
    egui::Window::new(format!("delete conversation {}", confirm.name))
        .collapsible(false)
        .resizable(false)
        // Centred for `super::delete`'s reason (bl-86a5): egui's automatic
        // cascade seats an unanchored window wherever the frame's other areas
        // leave room, which for a tall dialog is with its own fire button off
        // the bottom edge.
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
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
    // The descendant list is unbounded, so it scrolls in its own room and the
    // arming row below it cannot be pushed off the screen — `super::delete`'s
    // [`census_room`](crate::shell::delete::census_room) is the one definition of that
    // rule (bl-86a5). The deposit count is one line and stays outside it.
    crate::shell::delete::census_room(ui, "delete-agent-census", &mut |ui| {
        ui.label("children that die with it:");
        if census.descendants.is_empty() {
            ui.weak("(none)");
        }
        for id in &census.descendants {
            ui.weak(id);
        }
    });
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
/// way — a declined `litany delete` rides back as the executor's non-zero
/// outcome, which [`crate::shell::act::trouble`] spells).
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
    match crate::shell::act::trouble(&landed) {
        Some(reason) => state.delete_agent.error = reason,
        None => state.delete_agent = DeleteAgentState::default(),
    }
}
