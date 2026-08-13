//! The headless egui paint probe: render a widget tree off-screen and read
//! back every galley it painted.
//!
//! Every view module tests its render the same way — lay out into an
//! oversized offscreen rect so nothing scrolls out, run one frame, then walk
//! the emitted shapes concatenating their text. That walk was copy-pasted
//! into eight modules, and under the 100% floor **each copy carried its own
//! coverage obligation**: four of them shipped a byte-identical
//! `collect_text_descends_shape_vec_and_ignores_non_text` purely to reach
//! this file's recursion arm and catch-all. One home, one such test.

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

/// One painted galley: its text and the rect it landed on, in screen points.
pub(crate) type Painted = (String, egui::Rect);

/// Collect every painted galley with its position, descending `Shape::Vec` and
/// ignoring shapes that carry no text. The ONE walk — [`collect_text`] is this
/// one with the positions dropped, so a test that pins *where* content sits and
/// a test that pins *what* is on screen read the same frame the same way.
pub(crate) fn collect(shape: &egui::Shape, out: &mut Vec<Painted>) {
    let mut inked = Vec::new();
    descend(shape, &mut inked);
    out.extend(inked.into_iter().map(|(text, rect, _)| (text, rect)));
}

/// The one recursive walk, before either projection drops what it does not
/// need: glyphs, rect, and the ink they were painted in ([`ink`]).
fn descend(shape: &egui::Shape, out: &mut Vec<(String, egui::Rect, egui::Color32)>) {
    match shape {
        egui::Shape::Text(t) => out.push((
            visible(&t.galley),
            t.galley.rect.translate(t.pos.to_vec2()),
            ink(t),
        )),
        egui::Shape::Vec(shapes) => shapes.iter().for_each(|s| descend(s, out)),
        _ => {}
    }
}

/// The colour a run reached the glass in — its layout section's, or the
/// shape's `fallback_color` where that section defers to it.
///
/// `Color32::PLACEHOLDER` is what a plain `ui.label` lays: the widget declines
/// to name a colour, egui resolves it at paint time from the fallback, and
/// `Ui::set_opacity` dims **that** — so a seat faded by the §11 solidity
/// (`theme::tone_solidity`) is invisible to a reader of the section alone.
/// Both halves here, so "is this run faded?" is one question of the frame
/// rather than a guess about which widget drew it.
fn ink(t: &egui::epaint::TextShape) -> egui::Color32 {
    match t.galley.job.sections.first().map(|s| s.format.color) {
        Some(colour) if colour != egui::Color32::PLACEHOLDER => colour,
        _ => t.fallback_color,
    }
}

/// What a laid-out galley actually **shows**: its glyphs, row by row (bl-bc06).
///
/// `Galley::text()` is the text that went *in* — a galley egui truncated to
/// `…` still reports the whole string, so every assertion made against this
/// dump was blind to elision, the one defect the paint layer is the only
/// witness for. The glyphs are what reached the screen, so `contains("Login")`
/// now fails on a Login button rendered as a bare `…`, which is what makes the
/// dump evidence rather than a restatement of the input.
fn visible(galley: &egui::Galley) -> String {
    let mut out = String::new();
    for row in &galley.rows {
        out.extend(row.glyphs.iter().map(|g| g.chr));
        if row.ends_with_newline {
            out.push('\n');
        }
    }
    out
}

/// Concatenate every painted galley's text, one line each.
pub(crate) fn collect_text(shape: &egui::Shape, out: &mut String) {
    let mut painted = Vec::new();
    collect(shape, &mut painted);
    for (text, _) in painted {
        out.push_str(&text);
        out.push('\n');
    }
}

/// Concatenate the text of every shape in one finished frame.
pub(crate) fn text_of(output: &egui::FullOutput) -> String {
    let mut text = String::new();
    for clipped in &output.shapes {
        collect_text(&clipped.shape, &mut text);
    }
    text
}

/// Every galley of one finished frame, with its position.
pub(crate) fn painted_of(output: &egui::FullOutput) -> Vec<Painted> {
    let mut painted = Vec::new();
    for clipped in &output.shapes {
        collect(&clipped.shape, &mut painted);
    }
    painted
}

/// One galley **as the operator sees it** (bl-36c3): its glyphs, the rect it
/// was laid into, and the part of that rect its clip rect actually let through.
///
/// [`Painted`] answers *what was laid out*; that is not the same question as
/// *what is on the glass*. egui emits shapes it then clips away, and a run laid
/// wider than its container is sliced at the container's edge — mid-glyph, with
/// no ellipsis, because the galley itself was never truncated and so never had
/// one added. Only the two rects together tell that apart from a run that
/// simply fits, which is why both ride here rather than one being dropped.
#[derive(Clone)]
pub(crate) struct Seen {
    /// The laid-out glyphs — [`visible`]'s read, never `Galley::text()`.
    pub(crate) text: String,
    /// Where the run was laid, in screen points.
    pub(crate) laid: egui::Rect,
    /// The part of [`Self::laid`] the clip rect let through.
    pub(crate) shown: egui::Rect,
    /// The colour it was painted in ([`ink`]) — how a faded seat is legible to
    /// a test at all, since §11 solidity is an alpha on the run and nothing
    /// about its rect.
    pub(crate) ink: egui::Color32,
}

/// Every galley one finished frame put on the glass, each narrowed to what its
/// clip rect let through. Shapes clipped away entirely are dropped — they are
/// not on screen and are not evidence about it.
///
/// The ONE clip walk, beside [`collect`]'s text walk: `acceptance::overlap` asks
/// whether two shown rects share pixels and `acceptance::legible` asks whether a
/// shown rect is narrower than the run laid into it, and neither should carry
/// its own idea of what is visible.
pub(crate) fn seen_of(output: &egui::FullOutput) -> Vec<Seen> {
    let mut out = Vec::new();
    for clipped in &output.shapes {
        let mut here = Vec::new();
        descend(&clipped.shape, &mut here);
        out.extend(here.into_iter().filter_map(|(text, laid, ink)| {
            let shown = laid.intersect(clipped.clip_rect);
            (shown.width() > 0.5 && shown.height() > 0.5).then_some(Seen {
                text,
                laid,
                shown,
                ink,
            })
        }));
    }
    out
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

/// Every filled rect's hue, descending `Shape::Vec` — the one *fill* walk,
/// beside [`collect`]'s text walk. A role stripe (§11) is a rect with no
/// galley, so a hue assertion must read the fills the frame painted.
pub(crate) fn collect_fills(shape: &egui::Shape, out: &mut Vec<egui::Color32>) {
    match shape {
        egui::Shape::Rect(r) => out.push(r.fill),
        egui::Shape::Vec(shapes) => shapes.iter().for_each(|s| collect_fills(s, out)),
        _ => {}
    }
}

/// Render `body` as [`paint`] does and return every filled rect's hue.
pub(crate) fn paint_fills(mut body: impl FnMut(&mut egui::Ui)) -> Vec<egui::Color32> {
    let ctx = egui::Context::default();
    let output = ctx.run(screen(), |ctx| {
        egui::CentralPanel::default().show(ctx, &mut body);
    });
    let mut out = Vec::new();
    for clipped in &output.shapes {
        collect_fills(&clipped.shape, &mut out);
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

#[cfg(test)]
mod tests {
    use super::collect_text;

    #[test]
    fn collect_text_descends_shape_vec_and_ignores_non_text() {
        // Simple widgets don't nest galleys in a `Shape::Vec`, so build one
        // directly to cover the walker's recursion arm and its non-text
        // catch-all. This is the ONE copy of a test that four modules each
        // carried, because each carried its own copy of the walker.
        use egui::{Color32, FontId, Pos2, Stroke};
        let ctx = egui::Context::default();
        let mut nested: Option<egui::Shape> = None;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            // Two rows, so the glyph walk's own line break is exercised beside
            // its single-row case: a galley's rows carry no newline glyph, so
            // the walk re-inserts one where the row says it ended on it.
            let galley = ctx.fonts(|f| {
                f.layout_no_wrap("nested\nrows".into(), FontId::default(), Color32::WHITE)
            });
            nested = Some(egui::Shape::Vec(vec![egui::Shape::Text(
                egui::epaint::TextShape {
                    pos: Pos2::ZERO,
                    galley,
                    underline: Stroke::NONE,
                    fallback_color: Color32::WHITE,
                    override_text_color: None,
                    opacity_factor: 1.0,
                    angle: 0.0,
                },
            )]));
        });
        let mut out = String::new();
        collect_text(nested.as_ref().unwrap(), &mut out);
        collect_text(&egui::Shape::Noop, &mut out);
        assert_eq!(out, "nested\nrows\n");
    }

    #[test]
    fn collect_fills_descends_shape_vec_and_ignores_non_rect() {
        // The fill walk's own recursion arm and catch-all, on a built shape —
        // the same reason the text walker's test builds one (frames rarely
        // nest rects in a `Shape::Vec`).
        let rect = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(3.0, 10.0));
        let nested = egui::Shape::Vec(vec![egui::Shape::rect_filled(
            rect,
            0.0,
            egui::Color32::RED,
        )]);
        let mut out = Vec::new();
        super::collect_fills(&nested, &mut out);
        super::collect_fills(&egui::Shape::Noop, &mut out);
        assert_eq!(out, vec![egui::Color32::RED]);
    }
}
