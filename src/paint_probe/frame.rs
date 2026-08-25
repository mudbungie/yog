//! How a frame is **produced** — the offscreen inputs, the context runs and the
//! two-frame settle — as against [`super`], which is how one is **read**.
//!
//! Split from the walk at §12's budget on the seam the two already had: nothing
//! here traverses a shape. Every driver here runs an egui context and hands the
//! finished output straight to one of the parent's projections, so the walk
//! stays exactly one (`rules/no-hand-rolled-paint-walk.yml`) and this file is
//! the harness around it.

use super::{Painted, painted_of, text_of};

/// An offscreen input of exactly `w` x `h` logical points.
pub(crate) fn screen_sized(w: f32, h: f32) -> egui::RawInput {
    egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(w, h),
        )),
        ..Default::default()
    }
}

/// A screen big enough that every row lays out.
pub(crate) fn screen() -> egui::RawInput {
    screen_sized(1024.0, 4096.0)
}

/// Render `body` into a central panel on a [`screen`]-sized frame and return
/// everything it painted.
pub(crate) fn paint(mut body: impl FnMut(&mut egui::Ui)) -> String {
    let ctx = egui::Context::default();
    let output = ctx.run(screen(), |ctx| {
        egui::CentralPanel::default().show(ctx, &mut body);
    });
    text_of(&output)
}

/// Render `body` as [`paint`] does and return every filled rect's hue.
pub(crate) fn paint_fills(mut body: impl FnMut(&mut egui::Ui)) -> Vec<egui::Color32> {
    let ctx = egui::Context::default();
    let output = ctx.run(screen(), |ctx| {
        egui::CentralPanel::default().show(ctx, &mut body);
    });
    let mut out = Vec::new();
    for clipped in &output.shapes {
        super::collect_fills(&clipped.shape, &mut out);
    }
    out
}

/// The same walk on a screen small enough to scroll, read after the view has
/// settled: a `ScrollArea` learns its content extent *during* a frame, so both
/// halves of §11's tail idiom — the `stick_to_bottom` offset and the top pad
/// that seats an underfull body on the bottom edge (`crate::tail`) — first show
/// on the frame after. Two frames on one context is that settle; what is
/// returned is what a steady-state viewer actually sees, off-screen rows culled.
fn settled_frame(w: f32, h: f32, mut body: impl FnMut(&mut egui::Ui)) -> egui::FullOutput {
    let ctx = egui::Context::default();
    let mut frame = || {
        ctx.run(screen_sized(w, h), |ctx| {
            egui::CentralPanel::default().show(ctx, &mut body);
        })
    };
    let _ = frame();
    frame()
}

/// The settled frame's text — what is on screen.
pub(crate) fn paint_settled(w: f32, h: f32, body: impl FnMut(&mut egui::Ui)) -> String {
    text_of(&settled_frame(w, h, body))
}

/// The settled frame's galleys with their positions — where it sits.
pub(crate) fn painted_settled(w: f32, h: f32, body: impl FnMut(&mut egui::Ui)) -> Vec<Painted> {
    painted_of(&settled_frame(w, h, body))
}

/// The topmost and bottommost painted pixel of a frame — the two edges the §11
/// tail idiom is a claim about. An empty frame yields an inverted span, which
/// no assertion about a real one can pass.
pub(crate) fn span(painted: &[Painted]) -> (f32, f32) {
    painted
        .iter()
        .fold((f32::MAX, f32::MIN), |(top, bottom), (_, rect)| {
            (top.min(rect.top()), bottom.max(rect.bottom()))
        })
}
