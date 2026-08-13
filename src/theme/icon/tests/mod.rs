mod artifacts;
mod geometry;

use super::{HYDRA, ICON_PX, PHOSPHOR, VOID_DEEP, deep, icon_data, rgba};

/// One straight-alpha pixel out of a [`rgba`] buffer.
fn pixel(buf: &[u8], size: u16, x: u16, y: u16) -> [u8; 4] {
    let at = (usize::from(y) * usize::from(size) + usize::from(x)) * 4;
    buf.get(at..at + 4)
        .and_then(|slice| <[u8; 4]>::try_from(slice).ok())
        .unwrap_or([0, 0, 0, 0])
}

/// The pixel at unit coordinates (`ux`, `uy`) of a `size`-wide render.
fn at_unit(buf: &[u8], size: u16, ux: f32, uy: f32) -> [u8; 4] {
    let coord = |u: f32| (u * f32::from(size)) as u16;
    pixel(buf, size, coord(ux), coord(uy))
}

const SIZE: u16 = 64;

/// The two hues the mark is built from, and the void its pupil is filled with.
const CONDUCTOR: [u8; 4] = [32, 255, 108, 255];
const CASING: [u8; 4] = [12, 99, 42, 255];

/// A point of iris clear of the pupil, a point of bare casing, a point down the
/// middle of a trace where the conductor rides it, and one of the outer
/// circles. The wire's sample is mid-trace and clear of every circle by more
/// than a circle's radius, so only the trace can be painting it.
const ON_IRIS: (f32, f32) = (0.445, 0.492);
const ON_WIRE: (f32, f32) = (0.782, 0.806);
const ON_CASING: (f32, f32) = (0.711, 0.383);
const ON_NODE: (f32, f32) = (0.133, 0.711);

#[test]
fn the_window_icon_is_a_square_rgba_buffer() {
    let icon = icon_data();
    assert_eq!(icon.width, u32::from(ICON_PX));
    assert_eq!(icon.height, u32::from(ICON_PX));
    assert_eq!(
        icon.rgba.len(),
        usize::from(ICON_PX) * usize::from(ICON_PX) * 4
    );
}

#[test]
fn the_mark_sits_on_transparency_inside_its_frame() {
    let buf = rgba(SIZE);
    // No background plate, and nothing runs off the edge: the whole border is
    // clear, not merely the corners.
    for along in 0..SIZE {
        for (x, y) in [(along, 0), (along, SIZE - 1), (0, along), (SIZE - 1, along)] {
            assert_eq!(pixel(&buf, SIZE, x, y), [0, 0, 0, 0], "edge at {x},{y}");
        }
    }
}

/// The pupil is **filled**, not punched. Cut as negative space it would show
/// whatever lies beneath — and three traces converge directly under the eye,
/// so what it showed was them. An opaque fill is also one flat shape rather
/// than a coverage-subtraction primitive neither renderer has.
#[test]
fn the_pupil_is_opaque_void_and_not_a_hole_onto_the_traces() {
    let buf = rgba(SIZE);
    let [red, green, blue, alpha] = at_unit(&buf, SIZE, 0.5, 0.5);
    assert_eq!(alpha, 255, "the pupil must not be see-through");
    assert_eq!(
        [red, green, blue],
        [VOID_DEEP.r(), VOID_DEEP.g(), VOID_DEEP.b()],
        "the pupil should be the void, not a trace showing through"
    );
}

#[test]
fn the_iris_and_the_circles_are_the_one_phosphor_hue() {
    let buf = rgba(SIZE);
    assert_eq!(at_unit(&buf, SIZE, ON_IRIS.0, ON_IRIS.1), CONDUCTOR);
    assert_eq!(at_unit(&buf, SIZE, ON_NODE.0, ON_NODE.1), CONDUCTOR);
}

/// The conductor lightens the middle of the casing it rides. It is *not*
/// tested for a pure phosphor pixel, because at 64 px it is only 1 px wide and
/// never resolves to one — it reads as a bright core in a dim sleeve, which is
/// exactly what this asserts.
#[test]
fn the_conductor_brightens_the_middle_of_its_casing() {
    let buf = rgba(SIZE);
    let wire = at_unit(&buf, SIZE, ON_WIRE.0, ON_WIRE.1);
    assert_eq!(wire[3], 255, "mid-trace should be opaque");
    assert!(
        wire[1] > CASING[1],
        "mid-trace {wire:?} is no brighter than bare casing {CASING:?}"
    );
    assert!(
        wire[1] < CONDUCTOR[1],
        "and at this size it cannot reach full phosphor"
    );
}

/// The casing is the same hue held well under the conductor it carries — the
/// mark's whole range is one palette entry driven two ways.
#[test]
fn the_casing_is_the_same_hue_held_under_the_conductor() {
    let buf = rgba(SIZE);
    let sleeve = at_unit(&buf, SIZE, ON_CASING.0, ON_CASING.1);
    assert_eq!(sleeve, CASING);
    assert!(
        sleeve[1] < CONDUCTOR[1],
        "the casing must sit under the wire"
    );
    assert!(
        sleeve[1] > sleeve[0] && sleeve[1] > sleeve[2],
        "and still be green"
    );
}

#[test]
fn a_zero_sized_render_is_empty_rather_than_a_panic() {
    assert!(rgba(0).is_empty());
}

/// The badge hues are pastel by design — they must read as small marks on a
/// dark panel. `deep` is how the icon gets saturated colour without restating
/// a single RGB triple of its own, and driven past 1.0 it goes phosphor.
#[test]
fn deepening_a_palette_hue_takes_the_pastel_out_of_it() {
    let full = deep(HYDRA, 1.0);
    assert!(full.r() < HYDRA.r() / 3, "the white component goes");
    assert_eq!(
        deep(HYDRA, PHOSPHOR).g(),
        255,
        "driven past its peak the green goes phosphor"
    );
}
