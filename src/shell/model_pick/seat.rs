//! The §11 settings **seat** both surfaces wear: the pair row, whatever that
//! row's own state earns beneath it, and the pane's extras while it is open.
//! Coverage-excluded glue like the rest of `src/shell/*`.
//!
//! Two seats, one implementation (bl-824e, bl-2e18): an open conversation's
//! settings rows and the §11 birth-config block are the same bottom seat, one
//! branch on the selection apart. Only the scope claim and the drift clause
//! differ, so the two entry points derive those and hand the rest to [`seat`] —
//! a second painter would be a second authority on the same two files.

use super::{
    CenterTab, Cli, ModelRow, ShellState, birth_scope, conversation_scope, lines, marks, pane,
    refresh, roster_fault, row_names, select, settled, write,
};
use crate::AppModel;
use crate::git_tree::CommitNode;
use crate::model_pick::{NEW_CONVERSATION_EXIT, Pick, row_role};
use std::path::Path;

/// The **conversation's** settings seat (§11): the pair its two dropdowns show
/// and write, the drift clause when this conversation has parted from the
/// workspace default, and the pane's extras while it is open. Returns whether
/// the operator took the §9.4 drift exit — the caller owns the composer, so the
/// seat names the request rather than performing it.
pub(crate) fn conversation_seat(
    ui: &mut egui::Ui,
    model: &mut AppModel,
    state: &mut ShellState,
    ws: &Path,
    tip_oid: &str,
    config_tip: Option<&CommitNode>,
    clis: (&Cli, &Cli, &Cli),
) -> bool {
    let Some((frozen_oid, row)) =
        lines::conversation_row_of(ws, tip_oid, config_tip, &mut state.wall.picker)
    else {
        return false;
    };
    let scope = conversation_scope(ws, &frozen_oid);
    seat(ui, model, state, ws, &row, &scope, clis)
}

/// The **birth block's** seat (§11, bl-824e): the same row, asked of the config
/// branch head a conversation started now would fork. Returns whether it painted
/// at all — a workspace whose snapshot carries no config lineage yet has no pair
/// to offer, and says nothing rather than offering an empty one.
pub(crate) fn birth_seat(
    ui: &mut egui::Ui,
    model: &mut AppModel,
    state: &mut ShellState,
    ws: &Path,
    config_tip: Option<&CommitNode>,
    clis: (&Cli, &Cli, &Cli),
) -> bool {
    let Some(row) = lines::birth_row_of(ws, config_tip, &mut state.wall.picker) else {
        return false;
    };
    let scope = birth_scope(ws);
    seat(ui, model, state, ws, &row, &scope, clis);
    true
}

/// The seat both surfaces wear: the pair row, whatever the row's own state earns
/// beneath it (drift clause, roster fault, write receipt), and the pane's extras
/// while it is open. Returns whether the §9.4 drift exit was taken; a row with no
/// drift clause has no exit to take, which is every undrifted conversation and
/// every birth block.
fn seat(
    ui: &mut egui::Ui,
    model: &mut AppModel,
    state: &mut ShellState,
    ws: &Path,
    row: &ModelRow,
    scope: &str,
    clis: (&Cli, &Cli, &Cli),
) -> bool {
    let (bz, lernie_cli, bl) = clis;
    let role = row_role(state.wall.picker.role.as_deref());
    let rows = marks::provider_rows(&mut state.wall.picker);
    let view = settled(&mut state.wall.picker);
    let models = view.as_ref().map(|v| v.models.clone()).unwrap_or_default();
    // The pair, in two dropdowns and nothing else — the whole of the operator's
    // ruling. Everything the vanished sentence said rides `row.hover`.
    let choice = select::pair_row(
        ui,
        &mut state.wall.picker,
        row,
        &row_names(&rows),
        &models,
        view.is_none(),
    );
    // The roster is asked because the list is OPEN, not because the row exists
    // (bl-cd2a): the dropdowns are on screen whenever a conversation is.
    if choice.list_open {
        refresh(&mut state.wall.picker, &choice.provider, bz);
    }
    let mut route = choice.add_provider.then_some(CenterTab::Config);
    if let Some(view) = &view {
        route = route.or(roster_fault(
            ui,
            view,
            rows.iter().find(|r| r.name == choice.provider),
        ));
    }
    // The selection IS the commit (§9.4, bl-fb6b): there is no Set button and no
    // per-role apply. The role the row reports is the scope the click writes to.
    if let Some(chosen) = choice.chosen {
        let pick = Pick {
            role: role.clone(),
            provider: choice.provider,
            model: chosen,
        };
        write::apply(&mut state.wall.picker, model, ws, (lernie_cli, bl), &pick);
    }
    if !state.wall.picker.status.is_empty() {
        ui.label(&state.wall.picker.status);
    }
    let exit = drift_exit(ui, row);
    route = route.or(pane(ui, &mut state.wall.picker, ws, scope, &role));
    if let Some(tab) = route {
        state.wall.picker.toggle();
        // The route spends the one tab-focus gesture (bl-1ca2), which is both
        // the focus and the target pane's freshness — never a flag set beside
        // a read.
        crate::shell::center::focus(model, state, tab);
    }
    exit
}

/// The drift clause and its exit, painted under the pair when — and only when —
/// the workspace default has moved past this conversation (§9.4, bl-9786). A
/// conversation already on the current config has nothing to escape, so it gets
/// the bare pair the operator asked for and nothing else.
fn drift_exit(ui: &mut egui::Ui, row: &ModelRow) -> bool {
    let Some(clause) = &row.drift else {
        return false;
    };
    ui.horizontal(|ui| {
        ui.weak(clause).on_hover_text(&row.hover);
        // The one honest exit (§9.4): a conversation cannot adopt config
        // mid-lineage — a new one forks the current config. The affordance
        // points at the composer's own new-conversation verb rather than
        // growing a second way to start one.
        ui.button(NEW_CONVERSATION_EXIT)
            .on_hover_text(
                "Clear the selection and put the cursor in the composer, so the \
                 next thing you send starts a conversation on the config this \
                 workspace has now. This one cannot move off the commit it \
                 started from (n).",
            )
            .clicked()
    })
    .inner
}
