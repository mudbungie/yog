//! What the mark is made of, and the hues its circles take (DESIGN §11).
//!
//! **Flat, and that is structural.** Every primitive here is one flat fill, so
//! the three emissions of the mark's walk — raster, SVG and painter — are
//! *the same picture* rather than three approximations of one: they walk one
//! list in one order, with no light model and no gradient in between to drift.

use super::{ARMS, CIRCLES, HYDRA, arc};

/// One cross-section of a ribbon — the two points its edges pass through.
#[derive(Clone, Copy)]
pub(super) struct Rib {
    pub out: (f32, f32),
    pub back: (f32, f32),
}

/// Everything the mark is made of. Three primitives, one flat fill each; every
/// rendering walks one list of these in one order.
///
/// A [`Trace`](Shape::Trace) is named by its **centreline and width**, not by
/// its edges. That is what it is — a constant-width band down a path — and
/// naming it that way is what lets each renderer say it in its own primitives:
/// the two edge-walking emissions ask [`Shape::ribs`] for the outline, while the
/// painter hands egui the path and the width and lets its own stroker do the
/// joins and the antialiasing. Storing the edges instead would have forced the
/// painter to reconstruct the centreline it was built from.
pub(super) enum Shape {
    /// A constant-width band down the middle of a path.
    Trace {
        path: Vec<(f32, f32)>,
        width: f32,
        fill: egui::Color32,
    },
    /// The crescent between two arcs on one pair of endpoints — a ribbon whose
    /// width varies, so it is named by its ribs and cannot be a stroke.
    Lune { ribs: Vec<Rib>, fill: egui::Color32 },
    /// A flat circle.
    Disc {
        cx: f32,
        cy: f32,
        radius: f32,
        fill: egui::Color32,
    },
}

impl Shape {
    /// The ribbon's cross-sections — computed for a trace, stored for a lune,
    /// absent for a disc. The one doorway the edge-walking renderers use, so
    /// the raster and the SVG bound the same region by construction.
    pub(super) fn ribs(&self) -> Vec<Rib> {
        match self {
            Self::Trace { path, width, .. } => arc::ribs(path, *width),
            Self::Lune { ribs, .. } => ribs.clone(),
            Self::Disc { .. } => Vec::new(),
        }
    }

    /// The one flat colour this shape paints.
    pub(super) fn fill(&self) -> egui::Color32 {
        match self {
            Self::Trace { fill, .. } | Self::Lune { fill, .. } | Self::Disc { fill, .. } => *fill,
        }
    }
}

/// How many node circles the mark has — the seats an agent can take. Derived
/// from the walk (three arms of three circles), never restated.
pub const NODE_SEATS: usize = ARMS as usize * CIRCLES as usize;

/// The palette hue each of the mark's circles is driven from: the eye, then the
/// nine nodes in the walk's own order — arm by arm from bottom dead centre, each arm's
/// circles inner to outer.
///
/// **Hues, not colours.** Every seat goes through `deep` at the phosphor drive like
/// the mark always has, so the five states read as one family rather than five
/// pasted-in swatches, and [`Tints::rest`] — every seat [`HYDRA`] — is the logo
/// yog has always painted, to the byte. Rest is therefore not a case in the
/// walk; it is the walk with its default argument.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Tints {
    pub eye: egui::Color32,
    pub nodes: [egui::Color32; NODE_SEATS],
}

impl Tints {
    /// The mark as identity: every circle the one hue it is built from.
    pub fn rest() -> Self {
        Self {
            eye: HYDRA,
            nodes: [HYDRA; NODE_SEATS],
        }
    }
}
