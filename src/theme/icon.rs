//! The application icon — the congeries as a circuit (DESIGN §11).
//!
//! Three circles sit **tangent** to a central one, 120° apart with one at
//! bottom dead centre. From each, an arm runs 60° of arc to a further circle;
//! the circles between ride that arc, joined by a trace of dim casing under a
//! bright phosphor conductor. The centre carries a slit pupil in the void's own
//! black. Two hues, both derived from one palette entry, on transparency.
//!
//! **Both ends of an arm are pinned, and the swell is the only knob.** A start
//! tangent to the middle circle, an end 60° away at [`END_R`] — which is what
//! puts the top-right arm's last circle directly over the main circle — and
//! between them an arc whose sagitta is [`SWELL`]. Everything else falls out of
//! that. The legs come out equal for nothing, because even spacing along a
//! circular arc gives equal chords. The circle seats are *sampled from* the arc
//! rather than placed against it, so the drawn curve passes through every one
//! of them by construction, not by agreement.
//!
//! **What this mark is not.** An earlier one wore purple, gold and green — the
//! carnival tricolour — and read as a jester's hat: bulging lobes ending fat at
//! the rim, with twelve beads scattered there like bells. Three fixes, each
//! load bearing: **two hues** (a third saturated colour was what paid the
//! carnival tax), **arms that terminate rather than bulge**, and **few, large
//! elements**, because 64 px is where a taskbar icon lives and anything smaller
//! than a node is noise there.
//!
//! **Everything is compass work.** No spline, no easing, no tuned curve: an arc
//! is named by two endpoints and the height of its apex above the chord
//! (`arc::arc`), which fixes one circle exactly. The pupil is a *lune*, the
//! sliver between two arcs sharing a pair of endpoints, so it comes to a true
//! point at each end because that is where the arcs meet.
//!
//! **The pupil is filled, not punched.** Cut as negative space it showed what
//! lay beneath — and the traces converge under the eye, so what it showed was
//! them. Filling it with [`VOID_DEEP`] also spares both renderers a
//! coverage-subtraction primitive neither has.
//!
//! **Three emissions, one walk.** [`rgba`] rasterizes it for the window icon,
//! [`svg`](vector::svg) emits the checked-in vector source, and [`paint`] draws
//! it live on an egui layer with one circle per agent (§11 live mark). All
//! three walk the same flat-filled `shape::Shape` list in the same order, so
//! they are the same picture rather than three approximations of one.

use super::{HYDRA, VOID_DEEP};

mod arc;
mod paint;
mod raster;
mod shape;
mod vector;
pub use paint::paint;
pub use shape::{NODE_SEATS, Tints};
pub use vector::svg;

use arc::{arc, lune};
use shape::Shape;

/// Arms, and the degrees between them — the triskele's three-fold turn.
const ARMS: u8 = 3;
const TURN: f32 = 120.0;
/// Points sampled along one arc. Divisible by `CIRCLES - 1`, so the seats fall
/// on sampled points exactly rather than between them.
const STEPS: u16 = 48;
/// Where the first arm's tangent circle sits, in degrees: bottom dead centre.
const BASE: f32 = 90.0;
/// The middle circle, and the radius of every other circle — all equal.
const MAIN_R: f32 = 0.132;
const NODE_R: f32 = 0.048;
/// An arm ends this far out, and this many degrees of arc from where it began.
/// The pair is what puts the top-right arm's last circle over the main circle:
/// its base sits at −30°, and 60° counter-clockwise of that is −90°, straight up.
const END_R: f32 = 0.398;
const SWEEP: f32 = 60.0;
/// Circles an arm, counting the tangent one it starts on and the one it ends on.
const CIRCLES: u8 = 3;
/// **The only free parameter.** The sagitta of an arm's arc, as a fraction of
/// the chord between its two pinned ends: flat near 0, bowed as it climbs.
///
/// At 0.448 the arm leaves its tangent circle at exactly 45° off the radial —
/// which was the ask — and the arc sweeps 167.4°. An arc is a *semicircle*
/// exactly when its sagitta equals the **half** chord, which is `SWELL` 0.5;
/// that sweeps a true 180° but drops the departure to 41.8°. The two cannot
/// both hold, so the swap is this one constant and nothing else.
const SWELL: f32 = 0.448;
/// The dim casing, and the bright conductor laid down its middle.
const CASING_W: f32 = 0.054;
const CONDUCTOR_W: f32 = 0.016;
/// The pupil's half-height and bulge, as fractions of the middle circle.
const PUPIL_LONG: f32 = 0.78;
const PUPIL_BULGE: f32 = 0.30;
/// How hard [`deep`] drives the one hue the mark is built from: the conductor
/// past the hue's own peak until it is phosphor, the casing well under it.
const PHOSPHOR: f32 = 1.15;
const CASING: f32 = 0.45;
/// How much of a palette hue's white component [`deep`] strips.
const WHITE_CUT: f32 = 0.85;
/// Antialiased edge width, in pixels, of any rasterized shape.
const EDGE_PX: f32 = 1.2;

/// The rasterized icon size in pixels. 64 is the window-manager sweet spot:
/// crisp in a 48 px dock, still sharp when a title bar shrinks it to 16.
pub const ICON_PX: u16 = 64;

/// A palette hue with its white component stripped and its value scaled. The
/// badge hues are tuned to read as *small marks on a dark panel*, which makes
/// them pastel; a logo wants them saturated, and driven past 1.0 the green goes
/// phosphor. Deriving them keeps the theme module the one home of every hue
/// (§11) — the icon restates no RGB triple of its own.
fn deep(hue: egui::Color32, value: f32) -> egui::Color32 {
    let (red, green, blue) = (f32::from(hue.r()), f32::from(hue.g()), f32::from(hue.b()));
    let high = red.max(green).max(blue);
    let low = red.min(green).min(blue) * WHITE_CUT;
    let pure =
        |channel: f32| (high * (channel - low) / (high - low) * value).clamp(0.0, 255.0) as u8;
    egui::Color32::from_rgb(pure(red), pure(green), pure(blue))
}

/// The unit-square point `radius` out from the centre at `degrees`.
fn polar(radius: f32, degrees: f32) -> (f32, f32) {
    let turn = degrees.to_radians();
    (
        radius.mul_add(turn.cos(), 0.5),
        radius.mul_add(turn.sin(), 0.5),
    )
}

/// One arm's two **pinned** points: a circle tangent to the middle one, and a
/// circle [`SWEEP`] degrees of arc counter-clockwise of it at [`END_R`].
fn pins(turn: u8) -> ((f32, f32), (f32, f32)) {
    let base = BASE + TURN * f32::from(turn);
    (polar(MAIN_R + NODE_R, base), polar(END_R, base - SWEEP))
}

/// The arc between them, bowed by [`SWELL`] of the chord.
fn trace(turn: u8) -> Vec<(f32, f32)> {
    let (from, to) = pins(turn);
    let chord = (to.0 - from.0).hypot(to.1 - from.1);
    arc(from, to, SWELL * chord)
}

/// Where an arm's circles sit: [`CIRCLES`] points spaced evenly along its arc.
/// Even spacing on a circular arc means equal chords, so the legs between them
/// come out equal without anyone asking.
fn seats(turn: u8) -> Vec<(f32, f32)> {
    let path = trace(turn);
    let stride = usize::from(STEPS / u16::from(CIRCLES - 1));
    let mut out = Vec::new();
    for step in 0..CIRCLES {
        if let Some(seat) = path.get(usize::from(step) * stride) {
            out.push(*seat);
        }
    }
    out
}

/// The whole mark at rest — the identity, and what both artifact emissions
/// carry (a window icon states no agent's business).
fn mark() -> Vec<Shape> {
    mark_with(&Tints::rest())
}

/// The whole mark, back to front — **by layer, not by arm**. Every casing goes
/// down, then every conductor, then every circle, then the eye over the place
/// the arms converge. Drawing each arm complete in turn looks equivalent and is
/// not: the arms overlap near the middle, so a later one's casing would paint
/// over an earlier one's conductor.
///
/// The circles take their hues from `tints`; the traces never do. A trace is
/// the mark's own wiring, not an agent's business, and lighting it would leave
/// the reading of any one circle depending on which arm it sat on.
fn mark_with(tints: &Tints) -> Vec<Shape> {
    let (conductor, casing) = (deep(HYDRA, PHOSPHOR), deep(HYDRA, CASING));
    let traces: Vec<Vec<(f32, f32)>> = (0..ARMS).map(trace).collect();
    let mut out = Vec::new();
    for path in &traces {
        out.push(Shape::Trace {
            path: path.clone(),
            width: CASING_W,
            fill: casing,
        });
    }
    for path in &traces {
        out.push(Shape::Trace {
            path: path.clone(),
            width: CONDUCTOR_W,
            fill: conductor,
        });
    }
    // The zip is the seat assignment: nine seats, nine circles, in the walk's
    // order — so nothing outside this file ever indexes the shape list to find
    // out which circle is whose.
    for (seat, hue) in (0..ARMS).flat_map(seats).zip(tints.nodes) {
        out.push(Shape::Disc {
            cx: seat.0,
            cy: seat.1,
            radius: NODE_R,
            fill: deep(hue, PHOSPHOR),
        });
    }
    out.push(Shape::Disc {
        cx: 0.5,
        cy: 0.5,
        radius: MAIN_R,
        fill: deep(tints.eye, PHOSPHOR),
    });
    let reach = MAIN_R * PUPIL_LONG;
    out.push(lune(
        (0.5, 0.5 - reach),
        (0.5, 0.5 + reach),
        MAIN_R * PUPIL_BULGE,
        -MAIN_R * PUPIL_BULGE,
        VOID_DEEP,
    ));
    out
}

/// The window/taskbar icon for `ViewportBuilder::with_icon`. **On Wayland this
/// is never seen** — there is no protocol for a client to set its own window
/// icon there, and the compositor matches `app_id` to a desktop entry instead
/// (§11). It is the X11 path and the raster half of the parity story.
pub fn icon_data() -> egui::IconData {
    // The mark is square, so its side is one fact said once — and one binding
    // rather than two identical const conversions is also one coverage region
    // rather than two the optimizer can fold away unattributed.
    let side = u32::from(ICON_PX);
    egui::IconData {
        rgba: rgba(ICON_PX),
        width: side,
        height: side,
    }
}

/// The mark rasterized `size`×`size` as straight-alpha RGBA8, row-major from
/// the top-left — winit's icon format.
pub fn rgba(size: u16) -> Vec<u8> {
    raster::rgba(&mark(), size)
}

/// The sizes emitted as PNG beside the scalable SVG. The small end is what a
/// fixed-size icon theme installs; the large end is for dropping the mark into
/// a README or a page, where an SVG is not always welcome. The *encoding* is a
/// build-time concern — see `examples/icon.rs` — so nothing here carries a
/// codec into the binary.
pub const PNG_SIZES: [u16; 6] = [16, 32, 48, 64, 128, 256];

#[cfg(test)]
mod tests;
