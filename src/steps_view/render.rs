//! egui widget: the §11 Altitude-2 Steps tab — a headed per-step table plus a
//! tabbed drill-in.
//!
//! A pure function of the [`StepsView`] view-model, the caller's selection
//! (which step, which tab — viewport ephemera, §5.3, passed in), and the
//! drill-in [`StepDetail`] the caller built for the selected step. The one
//! interaction inside the tab is jsonview's collapse toggle (which owns its own
//! click); step- and tab-selection clicks are shell glue, exactly as the
//! transcript tab's Raw toggle is — so this fn stays a headless
//! shape-walk-tested pure render.
//!
//! The list is a **real table**: an [`egui::Grid`] whose header row is
//! [`super::columns::COLUMNS`], one cell per column per step, so a value always
//! sits under the word for it (bl-3ffc). The drill-in tier lives in
//! [`super::drill`].

use std::collections::HashSet;

use super::columns::{COLUMNS, Cell};
use super::{StepDetail, StepSummary, StepsView};
use crate::git_tree::Framing;
use crate::theme;

/// Which drill-in tab is showing. Selection is viewport ephemera the caller
/// owns (§5.3); the widget renders the chosen tab, and tab-switch clicks live
/// in the shell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepTab {
    Meta,
    Request,
    Staging,
    Response,
    Tools,
}

/// Render the step table, then — when the caller supplies the selected step's
/// [`StepDetail`] — the tabbed drill-in. `collapsed` is the caller-owned
/// jsonview collapse state (§5.3), threaded into every rendered tree; `raw` is
/// the §11 Raw toggle, which flips the drill-in's records from jsonview trees
/// to the record file's bytes unaltered (S7-T1). The table itself is a
/// projection of those same record files, so Raw belongs to the drill-in: the
/// bytes behind a row are the bytes of the step it names.
pub fn render(
    ui: &mut egui::Ui,
    view: &StepsView,
    selected: Option<usize>,
    detail: Option<&StepDetail>,
    tab: StepTab,
    collapsed: &mut HashSet<String>,
    raw: bool,
) {
    // §11 tail idiom, stated exactly: sit on the bottom while the bottom *is*
    // the newest step. A drill-in hangs below the table, so with one open the
    // body's bottom is the end of that detail, not the tail — and the idiom
    // comes off whole. Riding that bottom would scroll the table out of reach
    // the moment a step is picked; bottom-aligning a short one would push the
    // same rows down the viewport for the same wrong reason.
    crate::tail::scroll(ui, detail.is_none(), |ui| {
        if view.steps.is_empty() {
            ui.label("(no steps yet)");
        } else {
            render_table(ui, view, selected);
        }
        if let Some(detail) = detail {
            ui.separator();
            super::drill::render_detail(ui, detail, tab, collapsed, raw);
        }
    });
}

/// The headed table. Headers first — each naming its field and carrying the
/// one-line explanation on hover — then one aligned row per step. Absent
/// values still take their cell, so every column stays under its own heading.
fn render_table(ui: &mut egui::Ui, view: &StepsView, selected: Option<usize>) {
    egui::Grid::new("steps-table").striped(true).show(ui, |ui| {
        for column in COLUMNS {
            ui.label(egui::RichText::new(column.header).strong())
                .on_hover_text(column.hint);
        }
        ui.end_row();
        for (i, step) in view.steps.iter().enumerate() {
            for column in COLUMNS {
                paint_cell(ui, (column.cell)(step, Some(i) == selected));
            }
            ui.end_row();
        }
    });
}

fn paint_cell(ui: &mut egui::Ui, cell: Cell) {
    match cell {
        Cell::Colored(color, text) => ui.colored_label(color, text),
        Cell::Mono(text) => ui.monospace(text),
        Cell::Plain(text) => ui.label(text),
        Cell::Weak(text) => ui.weak(text),
        Cell::Empty => ui.label(""),
    };
}

/// The row's badge — glyph, hue, and the outcome **in words**. The §7.3
/// no-response wound outranks the framing read: framing alone classifies it
/// `Killed`, which paints the ash "stopped" badge a mid-stream kill gets —
/// exactly the quiet-step misreading the wound state exists to correct — so the
/// wound's own sentence ([`super::NO_RESPONSE`], bl-7f2e) is what the row says,
/// in the badge's one seat rather than a second label further along the row.
pub(super) fn summary_badge(step: &StepSummary) -> (&'static str, egui::Color32, &'static str) {
    if step.wound.wounded() {
        ("✖", theme::ICHOR, super::NO_RESPONSE)
    } else {
        framing_badge(step.framing)
    }
}

/// Glyph + colour + **the outcome said in words** for each §4.4 framing — the
/// same visual grammar as the git_tree state badges (✔ good / ✖ error / ■ dead),
/// and the §11 badge-seat pattern's one home for all three carriers of one fact:
/// the match is exhaustive, so a new framing cannot ship glyph-only, and no
/// renderer invents its own wording any more than it restates an RGB triple.
pub(super) fn framing_badge(framing: Framing) -> (&'static str, egui::Color32, &'static str) {
    match framing {
        Framing::Complete => ("✔", theme::HYDRA, "complete"),
        Framing::Failed => ("✖", theme::ICHOR, "failed"),
        // A kill, a crash, and a call in flight are indistinguishable on disk
        // (§2.9), so the words claim only what is known: it never ended clean.
        Framing::Killed => ("■", theme::ASH, "no clean end"),
    }
}
