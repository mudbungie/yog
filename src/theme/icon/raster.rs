//! The mark rasterized (DESIGN §11) — arithmetic only, so yog carries no image
//! decoder for a picture it can compute. Every shape is a flat fill, so this is
//! nothing but coverage and source-over: no light model, no gradient, nothing
//! the vector emitter could disagree with.

use super::EDGE_PX;
use super::shape::{Rib, Shape};

/// A shape with its ribbon edges already built. The walk is resolved **once**,
/// before the pixel loop: a trace names a centreline and a width (§11), and
/// turning that into ribs per pixel would do it four thousand times over for
/// one answer that never changes.
enum Flat {
    Ribbon {
        ribs: Vec<Rib>,
        fill: egui::Color32,
    },
    Disc {
        cx: f32,
        cy: f32,
        radius: f32,
        fill: egui::Color32,
    },
}

/// `shapes` rasterized `size`×`size` as straight-alpha RGBA8, row-major from
/// the top-left — winit's icon format.
pub(super) fn rgba(shapes: &[Shape], size: u16) -> Vec<u8> {
    let side = f32::from(size);
    let edge = EDGE_PX / side;
    let flats: Vec<Flat> = shapes.iter().map(Flat::of).collect();
    let mut out = Vec::with_capacity(usize::from(size) * usize::from(size) * 4);
    for y in 0..size {
        for x in 0..size {
            let point = sample(
                &flats,
                (f32::from(x) + 0.5) / side,
                (f32::from(y) + 0.5) / side,
                edge,
            );
            out.extend_from_slice(&point);
        }
    }
    out
}

/// A premultiplied source-over accumulator — the only compositing yog does.
#[derive(Default)]
struct Paint {
    red: f32,
    green: f32,
    blue: f32,
    alpha: f32,
}

impl Paint {
    /// Lay `fill` over what is already there at `cover` alpha.
    fn over(&mut self, fill: egui::Color32, cover: f32) {
        let keep = 1.0 - cover;
        let mix =
            |channel: u8, under: f32| (f32::from(channel) / 255.0).mul_add(cover, under * keep);
        self.red = mix(fill.r(), self.red);
        self.green = mix(fill.g(), self.green);
        self.blue = mix(fill.b(), self.blue);
        self.alpha = cover + self.alpha * keep;
    }

    /// Un-premultiply to the straight-alpha RGBA8 winit wants.
    fn straight(self) -> [u8; 4] {
        if self.alpha <= 0.0 {
            return [0, 0, 0, 0];
        }
        let byte = |value: f32| (value.clamp(0.0, 1.0) * 255.0) as u8;
        [
            byte(self.red / self.alpha),
            byte(self.green / self.alpha),
            byte(self.blue / self.alpha),
            byte(self.alpha),
        ]
    }
}

/// One pixel of the mark at unit point (`px`, `py`), every shape laid down in
/// order. `edge` is the antialias width in unit-square terms.
fn sample(shapes: &[Flat], px: f32, py: f32, edge: f32) -> [u8; 4] {
    let mut paint = Paint::default();
    for shape in shapes {
        let cover = shape.cover(px, py, edge);
        if cover > 0.0 {
            paint.over(shape.fill(), cover);
        }
    }
    paint.straight()
}

impl Flat {
    /// One walked shape with its edges resolved.
    fn of(shape: &Shape) -> Self {
        match shape {
            Shape::Disc {
                cx,
                cy,
                radius,
                fill,
            } => Self::Disc {
                cx: *cx,
                cy: *cy,
                radius: *radius,
                fill: *fill,
            },
            ribbon => Self::Ribbon {
                ribs: ribbon.ribs(),
                fill: ribbon.fill(),
            },
        }
    }

    /// The flat colour this shape paints.
    fn fill(&self) -> egui::Color32 {
        match self {
            Self::Ribbon { fill, .. } | Self::Disc { fill, .. } => *fill,
        }
    }

    /// Coverage at (`px`, `py`). A ribbon is the union of the quads between its
    /// consecutive ribs — neighbouring quads share a rib exactly, so their
    /// union is seamless and the best of them *is* the ribbon. Both ribbon
    /// primitives arrive here as one [`Flat::Ribbon`], so a trace and a lune are
    /// one rasterization.
    fn cover(&self, px: f32, py: f32, feather: f32) -> f32 {
        match self {
            Self::Disc { cx, cy, radius, .. } => {
                ((radius - (px - cx).hypot(py - cy)) / feather).clamp(0.0, 1.0)
            }
            Self::Ribbon { ribs, .. } => {
                let mut best: f32 = 0.0;
                for (near, far) in ribs.iter().zip(ribs.iter().skip(1)) {
                    best = best.max(quad(
                        (px, py),
                        [near.out, far.out, far.back, near.back],
                        feather,
                    ));
                }
                best
            }
        }
    }
}

/// Coverage of a convex quad, corners wound `near_out, far_out, far_back,
/// near_back`. Inside it, the distance to the nearest edge is the smallest of
/// the four half-plane distances; feathering that gives an antialiased edge,
/// and zero outside. Each edge is oriented by the quad's own centroid, so
/// either winding works.
///
/// **Only the even edges — the two long sides — are feathered.** The odd ones
/// are the ribs a quad shares with its neighbours: soften those and every join
/// paints at half coverage, striping the band with seams the union was meant
/// to erase. A shared rib is an interior line, not a silhouette, so it cuts
/// hard and the quads tile exactly.
fn quad(point: (f32, f32), corners: [(f32, f32); 4], feather: f32) -> f32 {
    let mid = (
        corners.iter().map(|corner| corner.0).sum::<f32>() / 4.0,
        corners.iter().map(|corner| corner.1).sum::<f32>() / 4.0,
    );
    let mut inside = f32::MAX;
    for (edge, (from, to)) in corners
        .iter()
        .zip(corners.iter().cycle().skip(1))
        .enumerate()
    {
        let (ex, ey) = (to.0 - from.0, to.1 - from.1);
        let span = ex.hypot(ey);
        let side = |at: (f32, f32)| ((at.0 - from.0) * ey - (at.1 - from.1) * ex) / span;
        let signed = side(point) * side(mid).signum();
        inside = inside.min(if edge % 2 == 0 {
            signed / feather
        } else if signed >= 0.0 {
            f32::INFINITY
        } else {
            f32::NEG_INFINITY
        });
    }
    (inside + 0.5).clamp(0.0, 1.0)
}
