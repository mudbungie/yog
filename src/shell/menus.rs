//! The §11 context-menu surface: one place where every object's secondary-click
//! menu is attached, and one dispatch from a [`Verb`] to its effect.
//!
//! The roster itself is the tested table in [`crate::nav::menu`]; this file is
//! its placement. Two properties every menu keeps (§11 doctrine, bl-ef89):
//!
//! - **Right-click is not the §6 gesture.** Opening a row's menu neither focuses
//!   nor acknowledges it — nothing here calls `focus_*`, and every verb acts on
//!   the [`Target`] the attach site resolved from the row under the pointer,
//!   never on a target re-derived from the focus. That is the accelerator's
//!   value beyond the click it saves: acting on an object without leaving the
//!   one being read. (It is also load-bearing: focusing acknowledges, so a
//!   focusing right-click would silently clear the row's §6 attention.)
//! - **A destructive verb reached here opens its confirmation**, exactly as its
//!   visible carrier does — [`fire`] calls the *same function* the visible
//!   affordance calls, so the menu can never reach past the dialog.
//!
//! Adding a seat is three edits: the [`Seat`] variant and its entries in
//! [`crate::nav::menu`], one [`attach`] caller beside the widget, and the verb's
//! arm in [`fire`] — pointing at the same function its visible carrier calls.

use crate::AppModel;
use crate::cli_outbound::Cli;
use crate::nav::menu::{self, Entry, Seat, Verb};
use crate::nav::tabs::Tab;
use std::path::PathBuf;

use super::ShellState;

/// The object a menu's verbs act on, resolved **at the attach site** from the
/// row under the pointer (§11: the menu is pointer-targeted where the visible
/// affordances are selection-targeted). Owned, so no borrow rides it.
pub(super) enum Target {
    Tab(Tab),
    Conversation { ws: PathBuf, agent: String },
    Ball(BallRef),
}

/// The three facts every §8.2 `bl` verb needs of a ball row: the project it runs
/// in, the ball, and the claimant it stamps `--as` (§3.2 — empty for a ball no
/// workspace holds yet, which only Assign can act on).
pub(super) struct BallRef {
    /// The project's §5.1 #1 wire **name** — what a `bl` verb's gesture takes,
    /// and what the answered ball row carries since bl-b4b5. It was the clone's
    /// path, resolved back to a name at each fire; the answer says the name, so
    /// nothing here resolves anything.
    pub project: String,
    pub id: String,
    pub owner: String,
}

/// Paint one seat's entries into `response`'s context menu, firing each on
/// click. An empty roster attaches nothing — no seat, no popup.
pub(super) fn attach(
    response: &egui::Response,
    seat: Seat,
    target: &Target,
    model: &mut AppModel,
    state: &mut ShellState,
    lernie: &Cli,
) {
    let entries = menu::entries(seat);
    if entries.is_empty() {
        return;
    }
    response.context_menu(|ui| paint(ui, &entries, target, model, state, lernie));
}

/// Render the roster: one button per entry (§11 — the roster is flat).
fn paint(
    ui: &mut egui::Ui,
    entries: &[Entry],
    target: &Target,
    model: &mut AppModel,
    state: &mut ShellState,
    lernie: &Cli,
) {
    for entry in entries {
        // §11 discoverability: an accelerator says what it accelerates, and the
        // roster already records that — `Entry::carrier` is the doctrine's own
        // claim that this verb exists elsewhere, so the hover is read from it
        // rather than written a second time here.
        let hint = accelerates(&entry.carrier);
        if ui.button(&entry.label).on_hover_text(hint).clicked() {
            fire(&entry.verb, target, model, state, lernie);
            ui.close_menu();
        }
    }
}

/// What a menu row does, said off the roster's own carrier claim: the same verb
/// as the visible affordance, aimed at the row under the pointer rather than at
/// whatever is selected (§11 context-menu doctrine).
fn accelerates(carrier: &str) -> String {
    format!(
        "Does the same as {carrier}, aimed at this row instead of the selection. A menu \
         is not on the Tab path, so that carrier is this verb's keyboard spelling."
    )
}

/// One menu verb's effect — the same call its visible carrier makes, never a
/// second implementation (the doctrine's teeth: delete every context menu and
/// the UI loses clicks, never capabilities).
fn fire(verb: &Verb, target: &Target, model: &mut AppModel, state: &mut ShellState, lernie: &Cli) {
    match (verb, target) {
        (Verb::DeleteWorkspace, Target::Tab(tab)) => super::delete::open(model, state, &tab.name),
        (Verb::Unpin, Target::Tab(tab)) => model.toggle_pin(&tab.name),
        (Verb::Stop { children }, Target::Conversation { ws, agent }) => {
            super::dispatch::stop_agent(model, ws, agent, *children);
        }
        (Verb::Flush, Target::Conversation { ws, .. }) => {
            super::dispatch::scan_ws(model, ws);
        }
        (Verb::DeleteAgent, Target::Conversation { ws, agent }) => {
            super::delete_agent::open(state, lernie, ws, agent);
        }
        (Verb::Assign(to), Target::Ball(ball)) => {
            super::ball_bar::assign_ball(model, &ball.project, &ball.id, to);
        }
        (Verb::Release, Target::Ball(ball)) => {
            super::ball_bar::release_ball(model, &ball.project, &ball.id, &ball.owner);
        }
        (Verb::CloseBall, Target::Ball(ball)) => {
            super::ball_bar::close_ball(model, &ball.project, &ball.id, &ball.owner);
        }
        // The roster pairs each verb with its own seat's target, so no other
        // combination is reachable; ignoring it is the panic-free way to say so.
        _ => {}
    }
}
