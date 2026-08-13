//! The §11 tail idiom in one home: a vertical scroll body that sits on its own
//! bottom edge.
//!
//! One rule with two halves, and every tail surface takes both from here rather
//! than restating either.
//!
//! - **Overfull** — `ScrollArea::stick_to_bottom`, egui's own
//!   release-on-scroll-up / re-engage-on-return, so nothing is stored (§13.1).
//! - **Underfull** — egui does *not* bottom-align a body shorter than the
//!   viewport; it draws it at the top and leaves the space below. So the body
//!   is pushed down by a top pad of `viewport − body`, clamped at zero. New
//!   text appears at the bottom edge from its very first line and growth pushes
//!   upward into the empty space, then scrolls: terminal semantics, which is
//!   what the operator asked for (bl-8c13).
//!
//! The pad is measured against **the previous frame's body height**, because a
//! scroll body only learns its own extent while painting — there is no height
//! to pad against until one frame has been drawn. That is the same one-frame
//! settle the anchor itself has ([`crate::paint_probe::paint_settled`] reads
//! frame two for exactly this reason). An unmeasured body is assumed to be
//! **the full viewport**, so the unsettled frame pads nothing and paints
//! exactly what a bare `stick_to_bottom` would: the settle can only ever move
//! content *down*, never scroll a tall body out of its own viewport for a frame
//! (assuming zero instead inflates the content by a viewport on frame one, and
//! an overfull body then spends frame two parked above the top of itself,
//! painting nothing). A height that moved asks for one more frame, so a body
//! that grows while the window is otherwise idle still re-seats itself.
//!
//! `anchored` is the surface's answer to *"is my bottom row my newest
//! content?"*, and `false` takes **both** halves off together — they are one
//! claim, not two knobs. Steps with a drill-in open is the standing case: the
//! body's last pixel is the end of a detail, so riding it would carry the step
//! rows off the top, and padding down to it would push the same rows down the
//! viewport for the same wrong reason (§11).

/// Show `body` in a vertical scroll area obeying the tail idiom above.
///
/// Returns the closure's own value beside the **body height this frame
/// painted** — the same measurement the pad already takes, handed out so a
/// caller whose region *is* its content (the inbox-composer's derived fold
/// line, bl-929d) can derive its extent from the one measurement rather than
/// keeping a second one.
///
/// `pub(crate)`: a bounded closure parameter has no business on the library
/// boundary (AGENTS rule 9), and every caller is a sibling view module.
pub(crate) fn scroll<R>(
    ui: &mut egui::Ui,
    anchored: bool,
    body: impl FnOnce(&mut egui::Ui) -> R,
) -> (R, f32) {
    let shown = egui::ScrollArea::vertical()
        .stick_to_bottom(anchored)
        .show(ui, |ui| {
            let key = ui.id().with("tail-body-height");
            let room = ui.max_rect().height();
            let settled: f32 = ui
                .ctx()
                .memory(|memory| memory.data.get_temp(key))
                .unwrap_or(room);
            if anchored {
                ui.add_space((room - settled).max(0.0));
            }
            let top = ui.cursor().top();
            let out = body(ui);
            let painted = (ui.min_rect().bottom() - top).max(0.0);
            // Sub-pixel drift is not a move; a real one re-seats next frame.
            if (painted - settled).abs() > 0.5 {
                ui.ctx()
                    .memory_mut(|memory| memory.data.insert_temp(key, painted));
                ui.ctx().request_repaint();
            }
            (out, painted)
        });
    shown.inner
}
