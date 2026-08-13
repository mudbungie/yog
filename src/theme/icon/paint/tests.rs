//! What the painter puts on the layer — read back off a headless frame's own
//! shapes, the image-side sibling of `paint_probe`'s galley walk.

use super::super::{NODE_SEATS, Tints};
use super::paint;
use crate::theme::{HYDRA, ICHOR, SPECTRE};

/// One painted circle: centre, radius, fill.
type Circle = (egui::Pos2, f32, egui::Color32);

/// Every circle in one shape tree, descending `Shape::Vec`.
fn circles_of(shape: &egui::Shape, out: &mut Vec<Circle>) {
    match shape {
        egui::Shape::Circle(circle) => out.push((circle.center, circle.radius, circle.fill)),
        egui::Shape::Vec(shapes) => shapes.iter().for_each(|s| circles_of(s, out)),
        _ => {}
    }
}

/// Paint the mark into a `w`×`h` rect and return every circle it laid down.
fn circles(tints: &Tints, w: f32, h: f32) -> Vec<Circle> {
    let ctx = egui::Context::default();
    let output = ctx.run(crate::paint_probe::screen(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let rect = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(w, h));
            paint(ui.painter(), rect, tints);
        });
    });
    let mut out = Vec::new();
    for clipped in &output.shapes {
        circles_of(&clipped.shape, &mut out);
    }
    out
}

/// Every stroked path in one shape tree, as (point count, stroke width).
fn paths_of(shape: &egui::Shape, out: &mut Vec<(usize, f32)>) {
    match shape {
        egui::Shape::Path(path) if path.stroke.width > 0.0 => {
            out.push((path.points.len(), path.stroke.width));
        }
        egui::Shape::Vec(shapes) => shapes.iter().for_each(|s| paths_of(s, out)),
        _ => {}
    }
}

/// Every trace the painter laid down on a `side`-square rect.
fn traces(side: f32) -> Vec<(usize, f32)> {
    let ctx = egui::Context::default();
    let output = ctx.run(crate::paint_probe::screen(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let rect = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(side, side));
            paint(ui.painter(), rect, &Tints::rest());
        });
    });
    let mut out = Vec::new();
    for clipped in &output.shapes {
        paths_of(&clipped.shape, &mut out);
    }
    out
}

/// A trace reaches egui as **what it is** — its centreline and its width — so
/// egui's own stroker does the joins and the antialiasing rather than this file
/// reconstructing edges. Six of them: three arms of casing, three of conductor,
/// each the walk's own sampled arc scaled to the rect.
#[test]
fn a_trace_goes_out_as_its_centreline_and_its_width() {
    let laid = traces(100.0);
    assert_eq!(laid.len(), 6, "three casings and three conductors");
    // Every trace carries the whole sampled arc: STEPS spans, so STEPS+1 points.
    assert!(laid.iter().all(|(points, _)| *points == 49), "{laid:?}");
    // The widths are the walk's constants scaled by the rect's side — CASING_W
    // laid down first, CONDUCTOR_W over it, never the other way round.
    let widths: Vec<f32> = laid.iter().map(|(_, w)| *w).collect();
    assert!(
        widths.iter().take(3).all(|w| (w - 5.4).abs() < 0.01),
        "{widths:?}"
    );
    assert!(
        widths.iter().skip(3).all(|w| (w - 1.6).abs() < 0.01),
        "{widths:?}"
    );
}

/// The distinct fills of a set of circles.
fn hues(painted: &[Circle]) -> std::collections::BTreeSet<[u8; 4]> {
    painted.iter().map(|(_, _, fill)| fill.to_array()).collect()
}

/// The mark is ten circles — nine node seats and the eye — however it is
/// tinted. That count is the whole reason a seat roster can address them.
#[test]
fn the_painter_lays_down_one_circle_per_seat_plus_the_eye() {
    assert_eq!(circles(&Tints::rest(), 100.0, 100.0).len(), NODE_SEATS + 1);
}

/// At rest every circle is the one hue — so the painted mark is the logo yog
/// has always carried, not a second drawing of it.
#[test]
fn a_rest_tint_paints_one_hue_on_every_circle() {
    assert_eq!(hues(&circles(&Tints::rest(), 100.0, 100.0)).len(), 1);
}

/// The walk's order *is* the seat order. An eye hue and one node hue land on
/// two different circles, the eye's on the largest — the middle circle the
/// pupil rides — and both are **driven** through `deep`, never the raw palette
/// entry pasted in.
#[test]
fn each_seat_takes_its_own_hue_and_the_eye_takes_the_largest_circle() {
    let mut tints = Tints::rest();
    tints.eye = ICHOR;
    if let Some(first) = tints.nodes.first_mut() {
        *first = SPECTRE;
    }
    let painted = circles(&tints, 100.0, 100.0);
    // Rest green, the one blue node, the red eye — three hues on ten circles.
    assert_eq!(hues(&painted).len(), 3);
    let eye = painted
        .iter()
        .copied()
        .reduce(|best, one| if one.1 > best.1 { one } else { best })
        .expect("the mark paints circles");
    assert!((eye.0.x - 50.0).abs() < 1.0 && (eye.0.y - 50.0).abs() < 1.0);
    assert_ne!(eye.2, ICHOR, "the hue is driven, never pasted in");
    assert_ne!(eye.2, HYDRA, "the eye did not take its tint");
}

/// A non-square rect **centres** the mark rather than stretching it: the mark
/// is round, and a stretched one is a different picture. The eye's radius is
/// `MAIN_R` of the short side, and the mark sits in the middle of the long one.
#[test]
fn a_wide_rect_centres_the_mark_instead_of_stretching_it() {
    let painted = circles(&Tints::rest(), 200.0, 40.0);
    let eye = painted
        .iter()
        .copied()
        .reduce(|best, one| if one.1 > best.1 { one } else { best })
        .expect("the mark paints circles");
    assert!((eye.1 - 0.132 * 40.0).abs() < 0.5, "eye radius {}", eye.1);
    assert!((eye.0.x - 100.0).abs() < 1.0 && (eye.0.y - 20.0).abs() < 1.0);
}
