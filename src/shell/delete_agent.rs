//! The §3.6 agent-delete surface (bl-f17a): the per-conversation confirmation
//! dialog and its two §11 carriers' shared entry.
//!
//! Coverage-excluded glue like the rest of `shell/*`: the confirmation, the
//! census parse, the dispatch gate and the prune are the tested
//! [`crate::delete::agent`] / [`AppModel`] surface; this file paints them.
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
use crate::cli_outbound::Cli;
use crate::delete::agent::{AgentConfirmation, Census};
use crate::theme;
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
        typed: String::new(),
        error,
    };
}

/// The visible carrier (§11 roster, conversation-row seat): the worded, ichor
/// row at the foot of the inspector's Config tab, aimed at the focused
/// agent's conversation root. Rendered only where the verb exists — a
/// yog-named workspace (§3.6 scope), exactly what a `None` confirmation says.
pub(super) fn danger_row(
    ui: &mut egui::Ui,
    model: &AppModel,
    state: &mut ShellState,
    lernie: &Cli,
    ws: &Path,
) {
    // The §2.3 descent root, off the seat's own view of its selection
    // (REMOTE §9.4, bl-1eb0) — the gate is the conversation's, not the member's.
    let Some(root) = model.focused_conversation().map(|seat| seat.root) else {
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
pub(super) fn dialog(
    ctx: &egui::Context,
    model: &mut AppModel,
    state: &mut ShellState,
    lernie: &Cli,
    bl: &Cli,
) {
    if state.delete_agent.target.is_none() {
        return;
    }
    window(ctx, model, state, lernie, bl);
    // Dismissed by any door — the ✕, a clean removal, or the workspace
    // vanishing — hands the keyboard back to the composer (§11).
    if state.delete_agent.target.is_none() {
        super::focus::request(state);
    }
}

/// The window itself, painted while a target stands.
fn window(
    ctx: &egui::Context,
    model: &mut AppModel,
    state: &mut ShellState,
    lernie: &Cli,
    bl: &Cli,
) {
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
        .show(ctx, |ui| body(ui, model, state, lernie, bl, &confirm));
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
    lernie: &Cli,
    bl: &Cli,
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
    let armed = if census.descendants.is_empty() {
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
    if ui
        .add_enabled(armed, egui::Button::new("Delete conversation"))
        .on_hover_text(
            "Remove this conversation from the workspace, permanently. Typed, it is \
             `/delete-agent [typed name]`.",
        )
        .on_disabled_hover_text("Type the conversation's name above to enable this.")
        .clicked()
    {
        fire(model, state, lernie, bl);
    }
    if !state.delete_agent.error.is_empty() {
        ui.colored_label(theme::ICHOR, &state.delete_agent.error);
    }
}

/// Fire through the one tested entry point; a refusal stays on the dialog
/// (the trail's own record is the `ops.jsonl` line either way).
fn fire(model: &mut AppModel, state: &mut ShellState, lernie: &Cli, bl: &Cli) {
    let Some((ws, root)) = state.delete_agent.target.clone() else {
        return;
    };
    let typed = state.delete_agent.typed.clone();
    match model.delete_agent(lernie, bl, &ws, &root, &typed, &super::now_ts()) {
        Ok(()) => state.delete_agent = DeleteAgentState::default(),
        Err(e) => state.delete_agent.error = e,
    }
}
