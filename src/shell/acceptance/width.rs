//! **The column's width is the operator's, under content that does not fit**
//! (§11 rules 1/1b/2/5, bl-0424) — the half [`super::geometry`] could not ask,
//! because no fixture in the suite carried content wide enough to ask it with.
//!
//! One root cause, two symptoms the operator reported. A row laid past the
//! panel's edge widens the rect egui stores *as the panel's width*, so the
//! column walks back out from under a splitter drag — and while the stored rect
//! is wider than the clamped panel, egui paints the panel's fill to the panel
//! and starts the centre at the **content's** edge, leaving an interval painted
//! by nobody that shows the frame's clear colour and flickers as the content
//! width moves.
//!
//! So the beats here are the two halves of one rect. The content half:
//! [`crate::spend::render`], the widest row the column carries, driven in a real
//! resizable panel and asserted in both directions — nothing past the edge, no
//! run elided to a bare `…`, and the same figure demonstrably wider than the
//! column when nothing bounds it, or the beat would pass on a figure that fit.
//! The panel half: the whole window over [`world_wide`], whose rows pair a name
//! that fills the row with a preview laid after it — the column opens where it
//! is told and stays there at every size the audit renders, no run in it is
//! elided to a bare `…`, and a boundary dragged to a sliver is still there
//! frames later instead of walking back out.
//!
//! Read through [`crate::paint_probe`] like every other paint claim here:
//! `Galley::text()` is the string that went in, and elision is exactly what
//! these beats are about.

/// **The widest row the column carries**, in its own file per §12's budget —
/// the same claim one altitude in.
mod figure;

use super::super::render;
use super::fixture::World;
use super::fixture::wide::{ROWS, world_wide};
use crate::cli_outbound::Cli;
use crate::paint_probe;
use crate::ui_state::Panel;

/// Frames enough for a ratchet to be a ratchet and then to overrun the ceiling
/// — one frame can only ever look innocent.
const RATCHET_FRAMES: usize = 14;

/// One settled frame of the real window at `w` x `h`, and the width egui stored
/// for the conversation panel afterwards.
fn settled(ctx: &egui::Context, world: &mut World, w: f32, h: f32) -> f32 {
    let (lernie, bl, bz) = (
        Cli::new("/yog-absent-lernie"),
        Cli::new("/yog-absent-bl"),
        Cli::new("/yog-absent-bz"),
    );
    let _ = ctx.run(paint_probe::screen_sized(w, h), |ctx| {
        render(ctx, &mut world.model, &mut world.state, &lernie, &bl, &bz);
    });
    world.settle();
    egui::containers::panel::PanelState::load(ctx, egui::Id::new("conversations"))
        .expect("the panel stores its rect")
        .rect
        .width()
}

/// **The column opens where it is told and stays there, at every size the audit
/// renders** (§11 rules 2 and 5, and the seam bl-0424 was filed for).
///
/// Nothing drags a boundary here, so the panel's answer is arithmetic: the
/// default width, or the rule 5 ceiling where the window is too small for it.
/// Any other number is content having written a width — and the stored number
/// *is* the content rect, which is the seam: egui clips a panel's fill to the
/// panel and starts the centre at the content's edge, so every point of
/// difference between the two is an interval painted by nobody.
///
/// Both directions: the wide fixture's rows must have reached the column, or a
/// panel that stayed narrow proves only that nothing was painted in it.
#[test]
fn the_conversation_column_opens_where_it_is_told_at_any_window_size() {
    let mut report = Vec::new();
    for (w, h) in super::SIZES {
        let mut world = world_wide();
        let ctx = egui::Context::default();
        let ceiling = Panel::Conversations.max_size(w);
        let opened = Panel::Conversations.default_size().min(ceiling);
        // Enough frames for a ratchet to be a ratchet: the walk was ~15 pt a
        // frame, so one frame can only ever look innocent.
        let mut widths = Vec::new();
        for _ in 0..RATCHET_FRAMES {
            widths.push(settled(&ctx, &mut world, w, h));
        }
        if widths.iter().any(|width| (width - opened).abs() > 1.0) {
            report.push(format!(
                "at {w:.0}x{h:.0} the column left its {opened} pt opening (ceiling \
                 {ceiling}): {widths:?}"
            ));
        }
        let out = frame(&mut world, &ctx, w, h);
        if !painted_a_wide_row(&out) {
            report.push(format!(
                "at {w:.0}x{h:.0} no wide row reached the column — the fixture is \
                 not being rendered and nothing here is being tested"
            ));
        }
        // Rule 1d in the seat that produced it: a preview laid after a title
        // that already fills the row is laid into nothing, and what lands is an
        // ellipsis with no run in front of it (bl-0424). It is also the
        // allocation that widened the panel, so the two halves are one reading.
        for seen in paint_probe::seen_of(&out) {
            if seen.text.trim() == "…" && seen.shown.right() <= opened + 1.0 {
                report.push(format!(
                    "at {w:.0}x{h:.0} a run in the column says only `…`, at {:?}",
                    seen.laid
                ));
            }
        }
    }
    // Every size at once: a width defect is a shape across sizes, and stopping
    // at the first hides which of them a fix actually moved.
    assert!(report.is_empty(), "{}", report.join("\n"));
}

/// One more frame, laid out against the settled ones — the frame a beat reads,
/// exactly as the operator's eye reads the repaint after the answer.
fn frame(world: &mut World, ctx: &egui::Context, w: f32, h: f32) -> egui::FullOutput {
    let (lernie, bl, bz) = (
        Cli::new("/yog-absent-lernie"),
        Cli::new("/yog-absent-bl"),
        Cli::new("/yog-absent-bz"),
    );
    ctx.run(paint_probe::screen_sized(w, h), |ctx| {
        render(ctx, &mut world.model, &mut world.state, &lernie, &bl, &bz);
    })
}

/// Whether the wide fixture's rows are on the glass at all — the head of a name
/// is enough, elision being the other rule's question.
fn painted_a_wide_row(out: &egui::FullOutput) -> bool {
    paint_probe::seen_of(out).into_iter().any(|seen| {
        ROWS.iter().any(|(_, name, _)| {
            // A head, because elision is a different rule with its own guard:
            // the column shows what it can of a name and the `…` is lawful.
            let head: String = name.chars().take(12).collect();
            !head.is_empty() && seen.text.starts_with(&head)
        })
    })
}

/// **The regression the operator reported**: a boundary dragged narrow is still
/// narrow four frames later (§11 rule 2). Before the containment the column
/// walked back out ~15 pt a frame from the moment the pointer came up, until it
/// sat pinned at half the window — so this asserts the frames *after* the drop,
/// not the drop itself.
#[test]
fn a_boundary_dragged_to_a_sliver_stays_there_under_content_wider_than_the_column() {
    let mut world = world_wide();
    let ctx = egui::Context::default();
    for _ in 0..5 {
        settled(&ctx, &mut world, 1150.0, 760.0);
    }
    // The stored rect a splitter drag to the panel's own floor leaves behind.
    ctx.data_mut(|d| {
        d.insert_persisted(
            egui::Id::new("conversations"),
            egui::containers::panel::PanelState {
                rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(40.0, 760.0)),
            },
        );
    });
    let dropped = settled(&ctx, &mut world, 1150.0, 760.0);
    assert!(
        dropped < 96.0,
        "the drag must take: the column settled at {dropped}"
    );
    for _ in 0..4 {
        let width = settled(&ctx, &mut world, 1150.0, 760.0);
        assert!(
            (width - dropped).abs() < 1.0,
            "and it must stay dropped rather than walking back out: \
             {dropped} → {width}"
        );
    }
}
