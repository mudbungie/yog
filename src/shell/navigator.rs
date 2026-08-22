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
///
/// **The column is a band stack, not a flow** (§11 rules 5 and 6 as extended by
/// bl-86a5). Written top-down the conversation list came first, and a
/// `ScrollArea` sizes itself from what is available and shrinks only for
/// content *smaller* than that — so a list taller than the panel took the whole
/// column and everything declared after it laid out past the panel's bottom
/// edge, where egui's own panel clip made it invisible AND un-clickable. That
/// cost the operator the ⚙ Config entry, which is the only visible door to the
/// §3.6 workspace danger row: a wall with a long enough list could not be
/// deleted through the window at all. Read bottom-up, the column is the same
/// budget the conversation pane divides ([`crate::layout`], `shell::pane`):
///
/// ```text
/// conversation list   — the column's own content, keeps half of it
/// balls + clients     — one budgeted band, scrolling in its own room
/// entries             — the doors, on the column's bottom edge
/// ```
///
/// egui creates docked panels outermost-first, so the code order below is the
/// reverse of the reading order and the band at the bottom edge claims first —
/// which is the point: the doors are what a starved column must shed **last**.
/// That is the conversation pane's priority inverted, and one rule covers both:
/// a band holds back whatever must outlive it, and nothing below the doors
/// does.
pub fn side_panel(ui: &mut egui::Ui, model: &mut AppModel, state: &mut ShellState, lernie: &Cli) {
    // Every row here truncates rather than extends (bl-9669): a row that
    // overflows widens the panel's `min_rect`, and egui *stores that rect as
    // the panel width*, so one long title ratchets the left column wider every
    // frame and the splitter can no longer shrink it. Truncation makes the
    // width the operator sets the width they get.
    ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);
    // The extent the stack is a share of, read ONCE before the first band is
    // created, exactly as `shell::pane` reads the conversation pane's.
    let column = ui.available_height();
    // **The doors claim first and hold nothing back**, because nothing below
    // them has to survive: this column's priority runs the other way from the
    // conversation pane's, where the bottom band holds back the goal box's
    // floor. They are all-or-nothing at `DOORS` — a band that can seat one of
    // two entries is not a smaller band, it is the §11 delete door missing —
    // and the balls band is nested inside that answer, so a column too starved
    // to seat the doors never seats the section above them either.
    if crate::layout::share(column, column, 0.0).is_some_and(|cap| cap >= DOORS) {
        entries(ui, model, state);
        if let Some(cap) = crate::layout::share(column, ui.available_height(), 0.0) {
            sections(ui, cap, model, state, lernie);
        }
    }
    super::conv_list::conversations(ui, model, state, lernie);
}

/// **The doors' own floor**: two entry rows, plus the panel frame's margins
/// above and below them — a row's worth between them, so three.
/// [`crate::layout::ROW`] is the floor of *a* row and a floor is per band, so a
/// budget that cannot seat the whole door band buys nothing: a panel handed a
/// ceiling under its content does not shrink to it, it lays out at its natural
/// height wherever it was seated (§11 rule 5's *"an accessory the container
/// cannot pay does not paint"*), which is the overlap again one level down.
const DOORS: f32 = 3.0 * crate::layout::ROW;

/// The entries that focus the center's Config and Login tabs (§11, §8.3), on
/// the column's bottom edge. They **focus**, never toggle: since bl-1ca2 both
/// are tab focuses — named peers of the conversation — so the entry lights up
/// with the tab it names and pressing it again is not a way out. The way out is
/// the strip, another entry, or Escape.
fn entries(ui: &mut egui::Ui, model: &mut AppModel, state: &mut ShellState) {
    egui::TopBottomPanel::bottom("navigator-entries").show_inside(ui, |ui| {
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
    });
}

/// The balls section and the workspace's registered clients (REMOTE §5,
/// bl-4e08 — who participates in it, who is connected right now, and what each
/// offers), as one budgeted band above the entries. They share a band because
/// both answer *what does this workspace have*, and both paint nothing at all
/// where there is nothing to say.
///
/// `cap` is the band's ceiling, already divided by the column: the panel is
/// sized by its content and a `ScrollArea` by what is available, so the two
/// lock each other at whatever the first frame happened to be unless the ui is
/// handed the cap outright — `shell::settings`'s own note, one door over.
fn sections(
    ui: &mut egui::Ui,
    cap: f32,
    model: &mut AppModel,
    state: &mut ShellState,
    lernie: &Cli,
) {
    egui::TopBottomPanel::bottom("navigator-sections").show_inside(ui, |ui| {
        ui.set_max_height(cap);
        egui::ScrollArea::vertical()
            .id_salt("navigator-sections")
            .max_height(cap)
            .show(ui, |ui| {
                // The width axis of the same clamp (bl-0424): this band holds
                // the board's spend rows and the `CollapsingHeader`s that lay
                // their own text `Extend` whatever the panel's wrap mode says,
                // so it is the band most able to widen the column through the
                // scroll's own outward-following rect.
                super::row::shown_width(ui);
                balls_section(ui, model, state, lernie);
                super::clients::section(ui, model);
            });
    });
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
        for ball in nav::balls::roster(&super::chrome::focused_balls(model)) {
            bound_ball_row(ui, model, state, lernie, &ball);
        }
    });
}

/// One bound ball row (§3.5): `id · badge`, weak, and the §11 ball-row
/// accelerator menu — Release / Close, each gated by the same §8.2
/// predicate the composer's own button is gated by. Sensing clicks is what makes
/// the row right-clickable; nothing here reacts to a primary click, so the row
/// stays a read and the menu stays pointer-targeted.
fn bound_ball_row(
    ui: &mut egui::Ui,
    model: &mut AppModel,
    state: &mut ShellState,
    lernie: &Cli,
    ball: &crate::nav::BoundBall,
) {
    let text = match &ball.badge {
        Some(b) => format!("{} · {b}", ball.id),
        None => ball.id.clone(),
    };
    let row = ui
        .add(egui::Label::new(egui::RichText::new(text).weak()).sense(egui::Sense::click()))
        .on_hover_text(
            "a ball this workspace has claimed. Right-click for its actions — Close \
             and Release. Each is also a key or a line: (c), (r), `/close [id]`, \
             `/release [id]`.",
        );
    let seat = Seat::BallRow {
        state: ball.state,
        assign_to: model.focused_ws_name(),
    };
    let target = Target::Ball(BallRef {
        project: ball.project.clone(),
        id: ball.id.clone(),
        owner: ball.owner.clone(),
    });
    super::menus::attach(&row, seat, &target, model, state, lernie);
}
