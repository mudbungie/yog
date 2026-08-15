//! The conversation-list side panel (§11 altitude 0). Coverage-excluded glue:
//! the conversation rows, the balls section's start affordances, and the
//! pin/collapse effects are all tested `AppModel`/`nav` methods; this file only
//! wires widgets. Its sibling [`super::top_bar`] paints the other altitude-0
//! surface, the attention strip + workspace tab bar.

use crate::AppModel;
use crate::cli_outbound::Cli;
use crate::keymap::CenterTab;
use crate::nav::{self, menu::Seat};

use super::ShellState;
use super::menus::{BallRef, Target};

/// The side panel (§11): the focused workspace's conversation list headed by
/// `new conversation`, then the collapsible balls section (start affordances +
/// bound balls), then the entries that focus the center's Config and Login
/// tabs. Nothing here paints a surface of its own — the Login pane used to
/// fold open *inside* this column, which is how ten provider rows came to
/// share a 200 pt panel with the roster (bl-1ca2).
pub fn side_panel(ui: &mut egui::Ui, model: &mut AppModel, state: &mut ShellState, lernie: &Cli) {
    // Every row here truncates rather than extends (bl-9669): a row that
    // overflows widens the panel's `min_rect`, and egui *stores that rect as
    // the panel width*, so one long title ratchets the left column wider every
    // frame and the splitter can no longer shrink it. Truncation makes the
    // width the operator sets the width they get.
    ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);
    super::conv_list::conversations(ui, model, state, lernie);
    ui.separator();
    balls_section(ui, model, state, lernie);
    // The workspace's registered clients (REMOTE §5, bl-4e08): who participates
    // in it, who is connected right now, and what each offers. It sits with the
    // balls section because both answer "what does this workspace have", and it
    // paints nothing at all where nothing is registered.
    super::clients::section(ui, model);
    ui.separator();
    // The entries that focus the center's Config and Login tabs (§11, §8.3).
    // They **focus**, never toggle: since bl-1ca2 both are tab focuses — named
    // peers of the conversation — so the entry lights up with the tab it names
    // and pressing it again is not a way out. The way out is the strip, another
    // entry, or Escape.
    for tab in [CenterTab::Config, CenterTab::Login] {
        let label = format!("{}{}", entry_mark(tab), tab.label());
        if ui
            .selectable_label(state.center == tab, label)
            .on_hover_text(tab.focus_hover())
            .clicked()
        {
            super::center::focus(model, state, tab);
        }
    }
}

/// The glyph an entry wears, or none. `⚙` is the settings glyph everywhere and
/// rides beside its own word, which is the §11 glyph doctrine satisfied; Login
/// has no such glyph and goes bare rather than borrowing a doubtful one.
fn entry_mark(tab: CenterTab) -> &'static str {
    match tab {
        CenterTab::Config => "⚙ ",
        _ => "",
    }
}

/// The minimal collapsible balls section (§11): the start affordances and the
/// focused workspace's bound-ball rows; the full per-project views return in
/// the ball-views wave. The fold is the persisted §4.1 collapse override.
fn balls_section(ui: &mut egui::Ui, model: &mut AppModel, state: &mut ShellState, lernie: &Cli) {
    let collapsed = model.is_collapsed("balls");
    let arrow = if collapsed { "▶" } else { "▼" };
    if ui
        .selectable_label(false, format!("{arrow} balls"))
        .on_hover_text(
            "show or hide the balls section — the ready tasks you can start and the \
             balls this workspace already holds (b)",
        )
        .clicked()
    {
        model.set_collapsed("balls", !collapsed);
    }
    if collapsed {
        return;
    }
    ui.indent("balls", |ui| {
        super::board::board(ui, model, state, lernie);
        // The focused workspace's remaining ball rows (§3.5 claimant join), id +
        // badge: the delivered ones. A *bound* ball is rendered in full by the
        // ▶ Continue row above, so it is not repeated here as a bare id
        // (bl-abbe) — `nav::balls::roster` is the covered partition, now a
        // selection out of the landed listing rather than a second derivation.
        let targets = nav::tabs::move_targets(&super::chrome::ws_rows(model), "");
        for ball in nav::balls::roster(&super::chrome::focused_balls(model)) {
            bound_ball_row(ui, model, state, lernie, &ball, &targets);
        }
    });
}

/// One bound ball row (§3.5): `id · badge`, weak, and the §11 ball-row
/// accelerator menu — Move / Release / Close, each gated by the same §8.2
/// predicate the composer's own button is gated by. Sensing clicks is what makes
/// the row right-clickable; nothing here reacts to a primary click, so the row
/// stays a read and the menu stays pointer-targeted.
fn bound_ball_row(
    ui: &mut egui::Ui,
    model: &mut AppModel,
    state: &mut ShellState,
    lernie: &Cli,
    ball: &crate::nav::BoundBall,
    targets: &[String],
) {
    let text = match &ball.badge {
        Some(b) => format!("{} · {b}", ball.id),
        None => ball.id.clone(),
    };
    let row = ui
        .add(egui::Label::new(egui::RichText::new(text).weak()).sense(egui::Sense::click()))
        .on_hover_text(
            "a ball this workspace has claimed. Right-click for its actions — Close, \
             Release, Move to another workspace. Each is also a key or a line: (c), \
             (r), `/move [id] <to>`.",
        );
    let seat = Seat::BallRow {
        state: ball.state,
        assign_to: model.focused_ws_name(),
        // The destinations minus this row's own holder — the same rule the
        // composer's `move to:` buttons read, folded once for the section.
        move_to: targets
            .iter()
            .filter(|n| **n != ball.owner)
            .cloned()
            .collect(),
    };
    let target = Target::Ball(BallRef {
        project: ball.project.clone(),
        id: ball.id.clone(),
        owner: ball.owner.clone(),
    });
    super::menus::attach(&row, seat, &target, model, state, lernie);
}
