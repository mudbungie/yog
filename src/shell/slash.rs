//! The composer's **line** seat (§8.5): a draft that starts with `/` is a
//! command, not something to say. Coverage-excluded glue like the rest of
//! `src/shell/*` — the reading is [`crate::boundary::line`]'s and the doing is
//! the boundary's; this file only supplies the seat and shows the answer.
//!
//! **The context is the focus, plus the one fact the model does not hold**: the
//! pending [`Prepared`](crate::start::Prepared) is start-flow RAM (§5.3), so it
//! is folded in here and nowhere else.
//!
//! **The start family needs no door of its own any more** (bl-1747). It had
//! two, because the frame-only aftermath rode their answers — the §3.4
//! workspace adoption, the held start claim, the §3.3 mint seed a landed fire
//! spends — and a headless consumer must not do those while a window must. Over
//! the wire the answer is a **receipt**, and a receipt is already the window's
//! own: `/prepare` and `/prompt` post the ordinary gestures and the aftermath
//! hangs off what came back ([`super::acting`]), beside the note this seat
//! paints. Same two facts, one mechanism, and no second entrance.
//!
//! Every answer renders as the reply's own JSON ([`reply::encode`]) — the same
//! bytes the deposit's reply file carries, because a line typed at the window
//! and one deposited from a terminal earn the same answer, and inventing a
//! second phrasing for it here would be the second implementation this whole
//! boundary exists to prevent.

use crate::AppModel;
use crate::actions::DraftKey;
use crate::boundary::{Action, Gesture, help, line, reply::Reply, reply::encode};
use crate::cli_outbound::Cli;
use std::path::PathBuf;

use super::ShellState;

/// The one box's other Enter (§8.5): the drafted line, run as a gesture.
const RUN_HINT: &str = "Run this slash command — the same gesture the buttons and keys fire, typed \
     instead of clicked. Its answer, or the reason it was refused, appears below. \
     Enter runs it without leaving the box.";

/// Run the drafted command. Returns whether the draft clears **on this frame**
/// — an answered *query* does, the answer being in hand; an **act** never does,
/// because whether it landed is its receipt's to say and the clear rides the
/// ticket instead (REMOTE §9.8, bl-1747). A refusal keeps the line either way,
/// so the operator fixes what they typed instead of retyping it (§5.3: a draft
/// is RAM until *sent*).
pub(super) fn run(
    model: &mut AppModel,
    state: &mut ShellState,
    lernie: &Cli,
    bl: &Cli,
    key: &DraftKey,
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
        Ok(Gesture::Act(action)) => {
            act(model, state, key, &seeded(state, action));
            // The line is RAM until it lands: whether it clears is the
            // receipt's answer, so nothing clears here.
            false
        }
    }
}

/// **This seat's own §3.3 prediction, carried onto the gesture** (bl-1747).
/// [`line::parse`] reads a `/prompt` off the typed text and predicts no name;
/// the composer above this box has been painting one all along, off
/// `start.mint_seed`, so the line fires that seed and the preview and the
/// minted `--name` are one draw. Every other action passes through untouched.
fn seeded(state: &ShellState, action: Action) -> Action {
    match action {
        Action::Prompt { prepared, goal, .. } => Action::Prompt {
            prepared,
            goal,
            seed: Some(state.start.mint_seed),
        },
        other => other,
    }
}

/// Fire one action at this seat: post it, and hold the draft it was typed in
/// (REMOTE §9.8). Every consequence — the note, whether the line clears, and
/// the start family's own adoption/claim/seed — rides the receipt, so this arm
/// makes no decision the buttons do not make.
fn act(model: &mut AppModel, state: &mut ShellState, key: &DraftKey, action: &Action) {
    // The line's address resolved at the seat's own door (REMOTE §8): the
    // aftermath is about a workspace **path**, the gesture carries a name, and
    // a gesture naming none is about none. A name the roster does not carry is
    // one a `/prepare` is about to found, so it resolves the §3.1 way — the
    // same path the §11 raise's own inputs name.
    let ws = action.workspace().map_or_else(PathBuf::new, |name| {
        model
            .snap
            .ws_path(&name)
            .unwrap_or_else(|_| model.new_workspace_inputs(&name).workspace)
    });
    super::acting::line(model, state, key, &ws, action);
}

/// Show the boundary's own answer, and say whether the line landed.
pub(super) fn note(state: &mut ShellState, result: Result<Reply, String>) -> bool {
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
        if run(model, state, lernie, bl, &ctx.key, &ctx.text) {
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
