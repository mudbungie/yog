use super::super::shape::Rib;
use super::super::{ARMS, CIRCLES, END_R, MAIN_R, NODE_R, SWEEP, arc, mark, pins, seats, trace};

/// Every ribbon the mark emits, as edges, in order — traces and the lune
/// alike, through the one doorway both edge-walking renderers use.
fn bands() -> Vec<Vec<Rib>> {
    let mut out = Vec::new();
    for shape in mark() {
        // Every shape is asked, and a disc simply has no edges — which is the
        // honest answer for it and keeps this walk free of a shape test.
        let ribs = shape.ribs();
        if !ribs.is_empty() {
            out.push(ribs);
        }
    }
    out
}

/// The bearing of a point about the centre, in degrees.
fn bearing(point: (f32, f32)) -> f32 {
    (point.1 - 0.5).atan2(point.0 - 0.5).to_degrees()
}

/// An arc is named by two endpoints and a sagitta, and must honour both: it
/// starts and ends where it was told, and its apex stands exactly the sagitta
/// off the chord. That is the whole contract everything else is built on.
#[test]
fn an_arc_meets_its_endpoints_and_stands_its_sagitta_off_the_chord() {
    for bulge in [0.05_f32, 0.12, -0.09, 0.40] {
        let (from, to) = ((0.2_f32, 0.3_f32), (0.7_f32, 0.55_f32));
        let points = arc(from, to, bulge);
        let (mut first, mut last) = ((0.0, 0.0), (0.0, 0.0));
        for (step, point) in (0u16..).zip(&points) {
            if step == 0 {
                first = *point;
            }
            last = *point;
        }
        assert!((first.0 - from.0).abs() < 1e-4 && (first.1 - from.1).abs() < 1e-4);
        assert!((last.0 - to.0).abs() < 1e-4 && (last.1 - to.1).abs() < 1e-4);
        let (dx, dy) = (to.0 - from.0, to.1 - from.1);
        let span = dx.hypot(dy);
        let mut stand = 0.0_f32;
        for point in &points {
            let off = ((point.0 - from.0) * dy - (point.1 - from.1) * dx) / span;
            if off.abs() > stand.abs() {
                stand = off;
            }
        }
        assert!(
            (stand.abs() - bulge.abs()).abs() < 1e-3,
            "apex stands {stand} off the chord, sagitta was {bulge}"
        );
    }
}

/// Each arm begins on a circle **tangent** to the middle one: centre to centre
/// is exactly the two radii summed, so the circles touch and never overlap.
#[test]
fn every_arm_starts_on_a_circle_tangent_to_the_middle_one() {
    for turn in 0..ARMS {
        let (start, _) = pins(turn);
        let apart = (start.0 - 0.5).hypot(start.1 - 0.5);
        assert!(
            (apart - (MAIN_R + NODE_R)).abs() < 1e-5,
            "arm {turn} starts {apart} out; tangent is {}",
            MAIN_R + NODE_R
        );
    }
}

/// One arm's tangent circle sits at bottom dead centre, and the arms are three
/// -fold about it.
#[test]
fn the_first_tangent_circle_is_at_bottom_dead_centre() {
    let (start, _) = pins(0);
    assert!((start.0 - 0.5).abs() < 1e-6, "not on the vertical axis");
    assert!(start.1 > 0.5, "not below the centre");
}

/// Each arm covers exactly `SWEEP` of arc, and its far end sits at `END_R` —
/// the two pinned facts the swell is free *within*.
#[test]
fn every_arm_covers_the_swept_angle_and_ends_on_the_far_radius() {
    for turn in 0..ARMS {
        let (start, end) = pins(turn);
        let swept = (bearing(start) - bearing(end) + 540.0) % 360.0 - 180.0;
        assert!(
            (swept - SWEEP).abs() < 1e-3,
            "arm {turn} sweeps {swept}, want {SWEEP}"
        );
        let out = (end.0 - 0.5).hypot(end.1 - 0.5);
        assert!((out - END_R).abs() < 1e-5, "arm {turn} ends at {out}");
    }
}

/// The consequence that pins the composition: the top-right arm's last circle
/// lands directly over the middle circle, on the vertical axis.
#[test]
fn the_top_right_arm_ends_directly_over_the_middle_circle() {
    // Bases run 90°, 210°, 330°; the last is the top-right one.
    let (_, end) = pins(ARMS - 1);
    assert!(
        (end.0 - 0.5).abs() < 1e-5,
        "the last circle sits {} off the axis",
        end.0 - 0.5
    );
    assert!(end.1 < 0.5, "and it must be above the centre, not below");
}

/// The legs come out equal because the seats are spaced evenly along a
/// circular arc, where equal spacing means equal chords. Nothing asks for it.
#[test]
fn the_legs_between_circles_are_equal() {
    for turn in 0..ARMS {
        let seat = seats(turn);
        assert_eq!(seat.len(), usize::from(CIRCLES));
        let mut legs = Vec::new();
        for (near, far) in seat.iter().zip(seat.iter().skip(1)) {
            legs.push((far.0 - near.0).hypot(far.1 - near.1));
        }
        for (index, leg) in (0u8..).zip(&legs) {
            let first = legs.first().copied().unwrap_or(0.0);
            assert!(
                (leg - first).abs() < 1e-4,
                "arm {turn} leg {index} is {leg}, first is {first}"
            );
        }
    }
}

/// The circles are *sampled from* the arc rather than placed against it, so
/// the drawn curve passes through every one of their centres exactly — the
/// curve fits the circles by construction, not by agreement.
#[test]
fn the_drawn_curve_passes_through_every_circle_centre() {
    for turn in 0..ARMS {
        let path = trace(turn);
        for seat in seats(turn) {
            let mut nearest = f32::MAX;
            for point in &path {
                nearest = nearest.min((point.0 - seat.0).hypot(point.1 - seat.1));
            }
            assert!(nearest < 1e-6, "a circle sits {nearest} off the curve");
        }
    }
}

/// A lune closes to a true point at both ends, because that is where its two
/// arcs meet — the pupil's shape is a property of the construction.
#[test]
fn the_pupil_closes_to_a_point_at_both_ends() {
    let mut pupil = Vec::new();
    for ribs in bands() {
        pupil = ribs; // the pupil is the last band emitted
    }
    let last = u16::try_from(pupil.len()).unwrap_or(u16::MAX) - 1;
    let mut ends = Vec::new();
    for (step, rib) in (0u16..).zip(&pupil) {
        if step == 0 || step == last {
            ends.push((rib.out.0 - rib.back.0).hypot(rib.out.1 - rib.back.1));
        }
    }
    assert_eq!(ends.len(), 2, "a band has two ends");
    for width in ends {
        assert!(width < 1e-4, "the pupil ends {width} wide, want a point");
    }
}
