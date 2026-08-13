//! Compass work (DESIGN §11) — the only geometry the mark is made of.
//!
//! Every curve here is named the way you would name it with a compass: two
//! endpoints and a **sagitta**, the height of the arc's apex above the chord
//! between them. That fixes one circle exactly, so nothing in the mark is a
//! tuned spline; each edge is a piece of a circle whose centre and radius fall
//! out of the arithmetic below.
//!
//! Two shapes are built from it. A **trace** runs a constant width along an
//! arc — a trace on a board — and is named by that centreline and that width,
//! never by its edges: [`ribs`] is how a renderer that wants edges gets them.
//! A [`lune`] is the region between *two* arcs drawn on the *same* pair of
//! endpoints: pointed at both ends because the arcs meet there, fat in the
//! middle by exactly the difference of their sagittas. The mark's slit pupil is
//! a lune, and any tapered limb built this way would be one too.

use super::STEPS;
use super::shape::{Rib, Shape};

/// The arc from `from` to `to` whose apex stands `bulge` off the chord —
/// positive to the left of the direction of travel — sampled into [`STEPS`]
/// spans. The radius is `(h² + s²) / 2s` for half-chord `h` and sagitta `s`;
/// the centre sits `radius − s` back from the chord's midpoint; and the swept
/// angle is `2·atan2(h, radius − s)`, which needs no special case for the
/// major arc because `radius − s` simply goes negative there.
pub(super) fn arc(from: (f32, f32), to: (f32, f32), bulge: f32) -> Vec<(f32, f32)> {
    let (side, sag) = (bulge.signum(), bulge.abs());
    let (dx, dy) = (to.0 - from.0, to.1 - from.1);
    let span = dx.hypot(dy);
    let half = span / 2.0;
    let radius = half.mul_add(half, sag * sag) / (2.0 * sag);
    let (nx, ny) = (-dy / span * side, dx / span * side);
    let centre = (
        (from.0 + to.0).mul_add(0.5, -nx * (radius - sag)),
        (from.1 + to.1).mul_add(0.5, -ny * (radius - sag)),
    );
    let start = (from.1 - centre.1).atan2(from.0 - centre.0);
    let sweep = 2.0 * half.atan2(radius - sag);
    (0..=STEPS)
        .map(|step| {
            let along = f32::from(step) / f32::from(STEPS);
            let angle = side.mul_add(-(sweep * along), start);
            (
                radius.mul_add(angle.cos(), centre.0),
                radius.mul_add(angle.sin(), centre.1),
            )
        })
        .collect()
}

/// The two edges of a constant-width band down the middle of `path` —
/// flat-cut at both ends, which is all a trace needs when one end sits under
/// the eye and the other under a node. The edge-walking renderers ask for
/// these; the painter states the centreline and the width to egui instead and
/// never sees a rib.
pub(super) fn ribs(path: &[(f32, f32)], width: f32) -> Vec<Rib> {
    let half = width / 2.0;
    let mut ribs = Vec::new();
    for (index, point) in (0u16..).zip(path) {
        let at = usize::from(index);
        let behind = path.get(at.saturating_sub(1)).copied().unwrap_or(*point);
        let ahead = path.get(at + 1).copied().unwrap_or(*point);
        let (dx, dy) = (ahead.0 - behind.0, ahead.1 - behind.1);
        let span = dx.hypot(dy).max(f32::EPSILON);
        let (nx, ny) = (-dy / span * half, dx / span * half);
        ribs.push(Rib {
            out: (point.0 + nx, point.1 + ny),
            back: (point.0 - nx, point.1 - ny),
        });
    }
    ribs
}

/// The crescent between two arcs on one pair of endpoints. Both are sampled
/// the same way, so rib `i` simply joins their `i`th points — and at the ends
/// those points coincide, which is what brings the shape to a true point
/// rather than a rounded stub.
pub(super) fn lune(
    from: (f32, f32),
    to: (f32, f32),
    out: f32,
    back: f32,
    fill: egui::Color32,
) -> Shape {
    Shape::Lune {
        ribs: arc(from, to, out)
            .into_iter()
            .zip(arc(from, to, back))
            .map(|(out, back)| Rib { out, back })
            .collect(),
        fill,
    }
}
