//! The composer's **line** seat (§8.5): a draft that starts with `/` is a
//! command, not something to say. Coverage-excluded glue like the rest of
//! `src/shell/*` — the reading is [`crate::boundary::line`]'s and the doing is
//! the boundary's; this file only supplies the seat and shows the answer.
//!
//! **The context is the focus, plus the one fact the model does not hold**: the
//! pending [`Prepared`](crate::start::Prepared) is start-flow RAM (§5.3), so it
//! is folded in here and nowhere else.
//!
//! **The start family goes through the frame's own doors** ([`AppModel::
//! prepare_start`], [`AppModel::fire_prompt`]) rather than the raw dispatch
//! match — not a second implementation (both doors *are* the chokepoint's typed
//! entrances, §8.5) but the frame-only aftermath beside it: the §3.4 workspace
//! adoption, the held start claim, and the §3.3 mint seed a landed fire spends.
//! A headless consumer must not do those; a window must.
//!
//! Every answer renders as the reply's own JSON ([`reply::encode`]) — the same
//! bytes the deposit's reply file carries, because a line typed at the window
//! and one deposited from a terminal earn the same answer, and inventing a
//! second phrasing for it here would be the second implementation this whole
//! boundary exists to prevent.

use crate::AppModel;
use crate::boundary::{Action, Gesture, help, line, reply::Reply, reply::encode};
use crate::cli_outbound::Cli;

use super::ShellState;

/// The one box's other Enter (§8.5): the drafted line, run as a gesture.
const RUN_HINT: &str = "Run this slash command — the same gesture the buttons and keys fire, typed \
     instead of clicked. Its answer, or the reason it was refused, appears below. \
     Enter runs it without leaving the box.";

/// Run the drafted command. Returns whether the draft clears — a refusal keeps
/// it, so the operator fixes the line they typed instead of retyping it (§5.3:
/// a draft is RAM until *sent*).
pub(super) fn run(
    model: &mut AppModel,
    state: &mut ShellState,
    lernie: &Cli,
    bl: &Cli,
    typed: &str,
) -> bool {
    let ctx = line::Context {
        prepared: state.start.pending.clone(),
        ..model.line_context()
    };
    match line::parse(typed, &ctx) {
        Err(reason) => {
            state.slash = Some(reason);
            false
        }
        Ok(Gesture::Ask(query)) => {
            let deps = model.boundary_deps(lernie, bl);
            let answer = model.answer(&deps, &query, super::now_unix());
            note(state, answer)
        }
        Ok(Gesture::Act(action)) => act(model, state, lernie, bl, &action),
    }
}

/// Fire one action at this seat and report what came back.
fn act(
    model: &mut AppModel,
    state: &mut ShellState,
    lernie: &Cli,
    bl: &Cli,
    action: &Action,
) -> bool {
    let ts = super::now_ts();
    match action {
        // The §8.1 start family through the frame's doors (see the module doc).
        Action::Prepare { workspace, payload } => {
            let prepared = model.prepare_start(lernie, bl, workspace, payload, &ts);
            if let Ok(ready) = prepared.as_ref() {
                // The prepared start takes the composer's seat (bl-6ad8), so
                // `/prompt` — or the goal box — fires exactly what was prepared.
                state.start.pending = Some(ready.clone());
            }
            note(state, prepared.map(Reply::Prepared))
        }
        Action::Prompt { prepared, goal } => {
            let seed = state.start.mint_seed;
            let fired = model.fire_prompt(lernie, bl, prepared, goal, seed, &ts);
            if fired.is_ok() {
                // The prediction this seed backed is a stamp now (bl-28ba), and
                // the prepared start it fired is spent.
                state.start.spend_mint();
                state.start.pending = None;
            }
            note(
                state,
                fired.map(|conversation| Reply::Started { conversation }),
            )
        }
        _ => {
            let deps = model.boundary_deps(lernie, bl);
            let result = model.dispatch(&deps, &ts, action);
            // The after-verb refresh the buttons make, chosen by the action's
            // own answer to which substrate it touched (§8.2).
            match action.project() {
                Some(project) => model.after_bl_verb(&project),
                None => model.after_lernie_verb(),
            }
            note(state, result)
        }
    }
}

/// Show the boundary's own answer, and say whether the line landed.
fn note(state: &mut ShellState, result: Result<Reply, String>) -> bool {
    match result {
        // Help renders as help (§8.5): the typed rows are the answer, and this
        // seat prints them the way a reader reads them — the same rendering the
        // terminal prints, not this pane's own phrasing. Every other reply is
        // its JSON, which is what those answers *are*.
        Ok(Reply::Help(rows)) => {
            state.slash = Some(help::render(&rows));
            true
        }
        // The §11 **Search tab** *is* the search answer (§8.5): its rows are
        // addresses to open, and JSON is not clickable. Same rule help follows
        // — a seat renders an answer the way that seat reads answers. Asking is
        // what focuses the tab (bl-1ca2): the ask is the operator's one
        // gesture, so the answer must not need a second one to be seen, and
        // the tab is already on offer this frame because the ask is
        // outstanding. An empty query clears the answer, the tab goes with it,
        // and the center falls back home — the vanishing is the dismissal.
        Ok(Reply::Search(_)) => {
            state.slash = None;
            super::focus::center(state, crate::keymap::CenterTab::Search);
            true
        }
        Ok(reply) => {
            state.slash = Some(
                serde_json::to_string_pretty(&encode(&reply))
                    .unwrap_or_else(|e| format!("unreadable reply: {e}")),
            );
            true
        }
        Err(reason) => {
            state.slash = Some(reason);
            false
        }
    }
}

/// The composer's line seat (§8.5): the one button, re-labelled, and the Enter
/// that fires it. Returns whether the draft **was** a command — the composer's
/// own verbs are what happens when it was not.
pub(super) fn seat(
    ui: &mut egui::Ui,
    model: &mut AppModel,
    state: &mut ShellState,
    lernie: &Cli,
    bl: &Cli,
    ctx: &super::verb_row::VerbCtx,
) -> bool {
    if !line::is_command(&ctx.text) {
        return false;
    }
    let clicked = ui
        .horizontal(|ui| ui.button("Run").on_hover_text(RUN_HINT).clicked())
        .inner;
    if clicked || ctx.entered {
        // A run keeps the keyboard (§11 focus discipline), asked on the attempt
        // and not the outcome — a refused line is fixed where it was typed.
        super::focus::request(state);
        if run(model, state, lernie, bl, &ctx.text) {
            state.actions.drafts.set(ctx.key.clone(), String::new());
        }
    }
    true
}

/// What the last line said back: a reply's own JSON, or the refusal. Bounded,
/// scrolling, and replaced by the next line — a `/balls` answer must not eat
/// the pane it was asked from.
///
/// **Preformatted, so it scrolls on both axes** (bl-5410). The reply is JSON:
/// its indentation is part of what it says, so neither of a row's two answers
/// fits it — truncating it to the pane's width leaves `{…` and nothing else
/// (§11 rule 1's `Truncate` keeps one row, and the note is many), and wrapping
/// it destroys the structure that made it readable. A blob is the case rule 6
/// answers instead: lay it whole and let the viewport reach it. Stated here
/// rather than inherited, in both directions — the wrap mode and the axis —
/// because the panel above now says `Truncate` and this seat means the opposite.
pub(super) fn note_ui(ui: &mut egui::Ui, state: &ShellState) {
    let Some(note) = state.slash.clone() else {
        return;
    };
    egui::ScrollArea::both()
        .id_salt("slash-note")
        .max_height(160.0)
        .show(ui, |ui| {
            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
            ui.label(egui::RichText::new(note).monospace());
        });
}
