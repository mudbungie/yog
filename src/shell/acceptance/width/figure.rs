//! **The widest row the column carries** (§11 rules 1/1d, bl-0424): the §3.5
//! spend figure, driven in a real resizable panel and asserted in every
//! direction the defect had.
//!
//! Split from [`super`] at §12's budget on the seam the two halves already
//! had — that file asks what a *panel* does under wide content, this one asks
//! what the widest *row* does inside one. The figure is the row the operator's
//! column actually carries: five parts on one line, of which the counter clause
//! alone is ~300 pt, so under rule 1's `Truncate` the greedy part takes the
//! whole width and the three laid after it each paint a bare `…` and allocate
//! it past the panel's edge — which is the rect egui stores as next frame's
//! panel width.

use super::super::super::{row, seat};
use crate::budgets::BudgetSpend;
use crate::paint_probe;
use crate::spend::{Attribution, Cost, Figure};
use crate::ui_state::Panel;

/// Frames enough for a ratchet to be a ratchet: egui stores a rect for the
/// *next* frame, so one frame can only ever look innocent — the measured walk
/// was ~15 pt a frame.
const FRAMES: usize = 6;

/// The window the row-level beats are laid in — wide enough that the column
/// opens at its full 260 pt default and the ceiling is not what is doing the
/// clamping, so what the beat measures is the row.
const WINDOW: f32 = 1150.0;

/// A figure with every part populated and every part long: the four counters at
/// provider scale, a priced sum with an unpriced remainder behind it, and the
/// §3.5 attribution clause. Five parts on one row is ~750 pt of content, and
/// the column is 260.
fn wide_figure() -> Figure {
    Figure {
        tokens: BudgetSpend {
            input_tokens: 1_234_567,
            output_tokens: 987_654,
            cache_read_tokens: 55_555_555,
            cache_write_tokens: 4_444_444,
        },
        cost: Some(Cost {
            micro_usd: 12_345_678,
            unpriced_tokens: 987_654,
        }),
        attribution: Attribution::Workspace,
    }
}

/// The board's own seat for a figure (`shell::board::rows`): the weak `spend:`
/// prefix and the figure beside it.
fn spend_row(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.weak("spend:");
        crate::spend::render(ui, &wide_figure());
    });
}

/// Drive `body` in a side panel built exactly as `shell::render` builds the
/// navigator — same floor, same ceiling, the same `row::bounded` at its root and
/// the same `seat` around its content — and hand back the width egui stored for
/// the panel on every frame, plus the last frame's output.
fn column(window: (f32, f32), body: &mut dyn FnMut(&mut egui::Ui)) -> (Vec<f32>, egui::FullOutput) {
    let (w, h) = window;
    let ctx = egui::Context::default();
    let mut widths = Vec::new();
    let mut last = None;
    for _ in 0..FRAMES {
        let out = ctx.run(paint_probe::screen_sized(w, h), |ctx| {
            egui::SidePanel::left("conversations")
                .resizable(true)
                .default_width(Panel::Conversations.default_size())
                .width_range(Panel::Conversations.min_size()..=Panel::Conversations.max_size(w))
                .show(ctx, |ui| {
                    row::bounded(ui);
                    seat(ui, &mut *body);
                });
            egui::CentralPanel::default().show(ctx, |_| {});
        });
        widths.push(
            egui::containers::panel::PanelState::load(&ctx, egui::Id::new("conversations"))
                .expect("the panel stores its rect")
                .rect
                .width(),
        );
        last = Some(out);
    }
    (widths, last.expect("FRAMES is not zero"))
}

/// **The widest row the column carries cannot move the column, at any size the
/// audit renders** (§11 rules 1/1d/2/5). Three claims of the same frame, because
/// the defect shows in three ways and a fix for one is not a fix for the others:
/// the stored width never leaves the width the panel opened at — which is also
/// the seam, since egui clips a panel's fill to the panel and starts the centre
/// at the **content** rect it stores — no glyph is laid past the panel's edge,
/// and no run is elided until `…` is all that is left (rule 1d: a part laid at
/// zero available width still allocates its ellipsis, which is *how* the
/// overflow happened).
///
/// Every size at once, and every failure at once: a width defect is a shape
/// across sizes, and stopping at the first hides which of them a fix moved.
#[test]
fn the_widest_board_row_cannot_ratchet_the_conversation_column_at_any_size() {
    let mut report = Vec::new();
    for (w, h) in super::super::SIZES {
        let ceiling = Panel::Conversations.max_size(w);
        let opened = Panel::Conversations.default_size().min(ceiling);
        let (widths, out) = column((w, h), &mut spend_row);
        if widths.iter().any(|width| (width - opened).abs() > 1.0) {
            report.push(format!(
                "at {w:.0}x{h:.0} the column left its {opened} pt opening: {widths:?}"
            ));
        }
        for seen in paint_probe::seen_of(&out) {
            if seen.laid.right() > opened + 1.0 {
                report.push(format!(
                    "at {w:.0}x{h:.0} `{}` is laid past the column's right edge at {}",
                    seen.text,
                    seen.laid.right()
                ));
            }
            if seen.text.trim() == "…" {
                report.push(format!(
                    "at {w:.0}x{h:.0} a part of the figure was elided until nothing \
                     was left, at {:?}",
                    seen.laid
                ));
            }
        }
    }
    assert!(report.is_empty(), "{}", report.join("\n"));
}

/// **And the row really is wider than the column**, or the beat above asserts
/// nothing at all. The same figure in a seat that bounds nothing lays past
/// 500 pt — twice the column — which is the width the column would have been
/// dragged out to.
#[test]
fn the_figure_under_test_is_wider_than_the_column_when_nothing_bounds_it() {
    let painted = paint_probe::painted_settled(2000.0, 400.0, |ui| {
        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
        ui.horizontal(|ui| {
            ui.weak("spend:");
            for part in parts() {
                ui.weak(part);
            }
        });
    });
    let widest = painted
        .iter()
        .fold(0.0_f32, |right, (_, rect)| right.max(rect.right()));
    assert!(
        widest > 2.0 * Panel::Conversations.default_size(),
        "the fixture figure must not fit the column, or nothing here is being \
         tested: it laid {widest} pt"
    );
}

/// Every part of [`wide_figure`], as the seat says them.
fn parts() -> Vec<String> {
    let figure = wide_figure();
    let spend = figure.tokens;
    let mut parts = vec![
        format!("budget {} tok", spend.total_tokens()),
        format!(
            "(in {} · out {} · cache r {} · w {})",
            spend.input_tokens,
            spend.output_tokens,
            spend.cache_read_tokens,
            spend.cache_write_tokens
        ),
    ];
    if let Some(cost) = figure.cost {
        parts.push(cost.usd());
        parts.push(format!("+{} tok unpriced", cost.unpriced_tokens));
    }
    if let Some(note) = figure.attribution.note() {
        parts.push(note.label.clone());
    }
    parts
}

/// **The centre settings seat still says the whole figure** (§3.5). Containing
/// a row is only a fix if the row still carries its facts where there is room
/// for them: every part is on the glass as itself, glyph for glyph, in the
/// centre the 1150x760 half-screen leaves.
#[test]
fn every_part_of_a_figure_is_painted_whole_in_the_centre_seat() {
    let painted =
        paint_probe::painted_settled(WINDOW - Panel::Conversations.default_size(), 600.0, |ui| {
            row::bounded(ui);
            crate::spend::render(ui, &wide_figure());
        });
    let said: Vec<&str> = painted.iter().map(|(text, _)| text.as_str()).collect();
    for part in parts() {
        assert!(
            said.iter().any(|seen| *seen == part),
            "`{part}` is not on the glass whole in the centre seat: {said:?}"
        );
    }
}
