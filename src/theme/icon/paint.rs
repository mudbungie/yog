//! The mark on an egui painter (DESIGN §11) — the **third** emission of the one
//! walk in [`super`], and the only one that runs every frame.
//!
//! The other two build an artifact once: the window-icon raster and the
//! checked-in SVG both state the mark at rest. This one states it *live* — one
//! circle per agent, hue = what that agent is doing — so it is walked with a
//! [`Tints`] the caller assembled, and it must be cheap enough to run at the
//! §7.2 repaint cadence.
//!
//! **Each primitive is handed to egui as the thing it is.** A trace names a
//! centreline and a width, so it goes out as a stroked polyline and egui's own
//! tessellator does the joins and the antialiasing; the lune is a lens — convex,
//! because its two arcs bow apart — so it goes out as one filled polygon; a disc
//! is a circle. Nothing here re-derives an edge or re-decides a hue: the walk
//! settled both, and this file only chooses egui's word for each shape.
//!
//! Why not the rasterizer's pixels, uploaded as a texture: it is a
//! shapes×pixels sweep, tens of milliseconds for one 64 px image, and a live
//! mark would pay that on the render thread every time an agent changed what it
//! was doing — the one thing §7.2 does not allow. The vector path is the cheap
//! one *and* the crisp one at the ~28 pt this is drawn at.

use super::mark_with;
use super::shape::{Shape, Tints};

/// Paint the mark into `rect`, every circle at the hue `tints` gives it.
///
/// The unit square the walk works in maps to the largest centred square `rect`
/// holds, so a seat that hands over a non-square rect gets the mark centred in
/// it rather than stretched — the mark is round and a stretched one is a
/// different picture.
pub fn paint(painter: &egui::Painter, rect: egui::Rect, tints: &Tints) {
    let side = rect.width().min(rect.height());
    let origin = rect.center() - egui::vec2(side, side) / 2.0;
    let at = |(x, y): (f32, f32)| origin + egui::vec2(x * side, y * side);
    for shape in mark_with(tints) {
        painter.add(match &shape {
            Shape::Trace { path, width, fill } => egui::Shape::line(
                path.iter().map(|point| at(*point)).collect(),
                egui::Stroke::new(width * side, *fill),
            ),
            Shape::Lune { ribs, fill } => {
                // Out along one arc, home along the other — the ribs' own two
                // edges, which meet at both ends and so close the lens with no
                // seam to hide.
                let mut points: Vec<egui::Pos2> = ribs.iter().map(|rib| at(rib.out)).collect();
                points.extend(ribs.iter().rev().map(|rib| at(rib.back)));
                egui::Shape::convex_polygon(points, *fill, egui::Stroke::NONE)
            }
            Shape::Disc {
                cx,
                cy,
                radius,
                fill,
            } => egui::Shape::circle_filled(at((*cx, *cy)), radius * side, *fill),
        });
    }
}

#[cfg(test)]
mod tests;
