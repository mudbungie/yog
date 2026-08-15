//! The §11 settings **seat** both surfaces wear: the pair row, whatever that
//! row's own state earns beneath it, and the pane's extras while it is open.
//! Coverage-excluded glue like the rest of `src/shell/*`.
//!
//! Two seats, one implementation (bl-824e, bl-2e18): an open conversation's
//! settings rows and the §11 birth-config block are the same bottom seat, one
//! branch on the selection apart. They differ in exactly one value — their
//! [`Subject`] — so the two entry points derive that and hand the rest to
//! [`seat`]; a second painter would be a second authority on the same two
//! files.

use super::{
    CenterTab, Cli, ModelRow, ShellState, birth_scope, conversation_scope, lines, marks, pane,
    refresh, roster_fault, row_names, select, settled, write,
};
use crate::AppModel;
use crate::model_pick::{
    ConfigTip, NEW_CONVERSATION_EXIT, Pick, RETARGET_EXIT, RETARGET_HOVER, row_role,
};
use std::path::Path;

/// What a seat is **about**: the scope claim its write states, and the
/// conversation it belongs to when there is one. The birth block has no
/// conversation — which is also why it can never reach the retarget exit, since
/// the clause that offers it is a fact about a conversation's own freeze.
struct Subject {
    scope: String,
    agent: Option<String>,
}

/// Which of the drift clause's two exits the operator took (§9.4, bl-2d19).
/// The clause offers a way out that **discards** and a way out that **keeps**,
/// and they are answered in different places — the composer owns one, the
/// boundary owns the other — so the seat names the choice rather than
/// performing both.
enum Exit {
    None,
    /// Focus the composer's new-conversation verb — the caller's, since the
    /// caller owns the composer.
    NewConversation,
    /// Fire `lernie retarget` on this conversation — the seat's own, since it
    /// is a boundary gesture like the pick above it.
    Retarget,
}

/// The **conversation's** settings seat (§11): the pair its two dropdowns show
/// and write, the drift clause when this conversation has parted from the
/// workspace default, and the pane's extras while it is open. Returns whether
/// the operator took the §9.4 drift exit — the caller owns the composer, so the
/// seat names the request rather than performing it.
///
/// The branch tip this conversation is frozen on is the **selection's own
/// detail** and is asked for here (REMOTE §9.7, bl-48ae): it is a fact about one
/// agent rather than about the §11 list, so it rides the standing
/// `Query::Agent` rather than a row. A frame the engine has not answered has no
/// pair to offer and offers none — the collapsed-pane rule at one row, and the
/// same beat the transcript above it keeps.
pub(crate) fn conversation_seat(
    ui: &mut egui::Ui,
    model: &mut AppModel,
    state: &mut ShellState,
    ws: &Path,
    agent_id: &str,
    config_tip: Option<&ConfigTip>,
    bz: &Cli,
) -> bool {
    let Some(tip) = crate::shell::seat::detail(model, ws, agent_id)
        .value
        .map(|view| view.tip)
    else {
        return false;
    };
    let Some((frozen_oid, row)) =
        lines::conversation_row_of(ws, &tip, config_tip, &mut state.wall.picker)
    else {
        return false;
    };
    let subject = Subject {
        scope: conversation_scope(ws, &frozen_oid),
        agent: Some(agent_id.to_owned()),
    };
    seat(ui, model, state, ws, &row, &subject, bz)
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
    config_tip: Option<&ConfigTip>,
    bz: &Cli,
) -> bool {
    let Some(row) = lines::birth_row_of(ws, config_tip, &mut state.wall.picker) else {
        return false;
    };
    let subject = Subject {
        scope: birth_scope(ws),
        agent: None,
    };
    seat(ui, model, state, ws, &row, &subject, bz);
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
    subject: &Subject,
    bz: &Cli,
) -> bool {
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
        write::apply(&mut state.wall.picker, model, ws, &pick);
    }
    // The receipt, folded once on the frame it lands (REMOTE §9.8): the write
    // above is a post, so what is painted here is the sentence the click wrote,
    // marked while the engine has not answered it.
    write::settle(&mut state.wall.picker, model);
    if !state.wall.picker.act.quiet() {
        ui.label(state.wall.picker.act.line());
    }
    let exit = drift_exit(ui, row);
    // The exit that keeps the conversation is a boundary gesture, so it fires
    // where the pick above it fires — through the one chokepoint, with its
    // receipt in the same sentence the pick writes. `agent` is `Some` exactly
    // when a conversation is selected, which is exactly when a drift clause
    // exists, so the birth block can never reach it.
    if let (Exit::Retarget, Some(agent)) = (&exit, subject.agent.as_deref()) {
        write::retarget(&mut state.wall.picker, model, ws, agent);
    }
    route = route.or(pane(ui, &mut state.wall.picker, ws, &subject.scope, &role));
    if let Some(tab) = route {
        state.wall.picker.toggle();
        // The route spends the one tab-focus gesture (bl-1ca2), which is both
        // the focus and the target pane's freshness — never a flag set beside
        // a read.
        crate::shell::center::focus(model, state, tab);
    }
    matches!(exit, Exit::NewConversation)
}

/// The drift clause and its two exits, painted under the pair when — and only
/// when — the workspace default has moved past this conversation (§9.4,
/// bl-9786). A conversation already on the current config has nothing to
/// escape, so it gets the bare pair the operator asked for and nothing else.
///
/// **The frozen sentence is where the way out belongs** (bl-2d19): the operator
/// reads *this conversation is frozen on …* and the verbs that answer it are
/// the next thing on the row, not a fourth control somewhere else. Keeping the
/// history leads, because discarding it is the larger act.
fn drift_exit(ui: &mut egui::Ui, row: &ModelRow) -> Exit {
    let Some(clause) = &row.drift else {
        return Exit::None;
    };
    // **A strip of peers** (§11 rule 8, [`crate::shell::row::peers`]): the two
    // exits are controls of their own natural width and neither may be dropped,
    // so the row wraps rather than omitting one — which is what a plain
    // `horizontal` does to the second button in the pane a 420x320 window
    // leaves. Beside the sentence wherever there is room, under it where there
    // is not; never absent.
    crate::shell::row::peers(ui, |ui| {
        ui.weak(clause).on_hover_text(&row.hover);
        // The exit that KEEPS (§9.4 as amended, bl-2d19): lernie's `retarget`
        // re-forks this conversation onto the current config at its next step
        // and replays its work on top, so the history survives the move.
        if ui
            .button(RETARGET_EXIT)
            .on_hover_text(RETARGET_HOVER)
            .clicked()
        {
            return Exit::Retarget;
        }
        // The exit that DISCARDS (§9.4, bl-9786): a new conversation forks the
        // current config by the ordinary path. The affordance points at the
        // composer's own new-conversation verb rather than growing a second way
        // to start one.
        if ui
            .button(NEW_CONVERSATION_EXIT)
            .on_hover_text(
                "Clear the selection and put the cursor in the composer, so the \
                 next thing you send starts a conversation on the config this \
                 workspace has now — leaving this one where it is (n).",
            )
            .clicked()
        {
            return Exit::NewConversation;
        }
        Exit::None
    })
}
