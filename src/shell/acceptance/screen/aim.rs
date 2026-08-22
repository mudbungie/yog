//! **How a beat names a seat on the glass** (§11): the rect a painted galley
//! landed on, and the coordinate a click is aimed at. Split from [`super`] at
//! §12's budget — the driver there is how a frame is run, this is how a
//! coordinate is found in one.

/// The rect of the galley reading exactly `text` — how a pointer test names a
/// seat without an `egui::Id` it has no way to know, and how a geometry test
/// asks which panel a row landed in. The **first**, in paint order; a word one
/// window paints twice needs [`rects_of`].
pub(in crate::shell::acceptance) fn rect_of(
    shapes: &[egui::epaint::ClippedShape],
    text: &str,
) -> Option<egui::Rect> {
    rects_of(shapes, text).into_iter().next()
}

/// **Every** rect a galley reading exactly `text` landed on. A label is not a
/// seat: `Login` is on this window three times — the navigator entry, the §11
/// tab strip's entry and the §8.3 row's verb — so a beat about the verb has to
/// tell them apart by where they landed, which it cannot do from the first
/// match alone.
pub(in crate::shell::acceptance) fn rects_of(
    shapes: &[egui::epaint::ClippedShape],
    text: &str,
) -> Vec<egui::Rect> {
    shapes
        .iter()
        .flat_map(|clipped| find(&clipped.shape, text))
        .collect()
}

/// The centre of that rect — the coordinate a click is aimed at.
pub(in crate::shell::acceptance) fn locate(
    shapes: &[egui::epaint::ClippedShape],
    text: &str,
) -> Option<egui::Pos2> {
    rect_of(shapes, text).map(|r| r.center())
}

/// Over [`paint_probe::collect`] — the ONE walk — and not a private copy of it.
///
/// This *was* a copy, and it was the copy bl-bc06 fixed and bl-36c3 swept for:
/// it matched on `Galley::text()`, which is the string that went IN. A row egui
/// truncated to `Login (bz browser…` still reports the whole label, so this
/// found it, handed back its rect, and the pointer test clicked confidently at
/// a seat whose painted text was not what it named — the one defect the paint
/// layer is the only witness for, aiming a click instead of reading a dump.
/// Both earlier balls fixed the homes they knew about; this copy was private to
/// the acceptance harness and survived both, which is why the check that
/// forbids the shape now lives in `rules/no-hand-rolled-paint-walk.yml` rather
/// than in anyone's memory (bl-70b8).
fn find(shape: &egui::Shape, text: &str) -> Vec<egui::Rect> {
    let mut painted = Vec::new();
    crate::paint_probe::collect(shape, &mut painted);
    painted
        .into_iter()
        .filter(|(seen, _)| seen == text)
        .map(|(_, rect)| rect)
        .collect()
}
