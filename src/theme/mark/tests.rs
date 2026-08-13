//! The two mark seats: what the tint assembly lays on which circle, what the
//! roster says in words, and that both seats actually paint.

use super::super::icon::{NODE_SEATS, Tints};
use super::{live_mark, roster, tints, wordmark};
use crate::nav::convs::{Doing, Seat};
use crate::theme::{BRAZEN, HYDRA, ICHOR, SIGIL, SPECTRE};

fn seat(name: &str, doing: Doing) -> Seat {
    Seat {
        name: name.to_owned(),
        doing,
    }
}

/// Nothing open is the mark at rest — the logo, byte for byte.
#[test]
fn no_seats_is_the_mark_at_rest() {
    assert!(tints(&[]) == Tints::rest());
    assert_eq!(roster(&[]), super::NOTHING_OPEN);
}

/// The first seat is the eye and the rest ride the node circles in order —
/// each seat's own hue, none of them borrowed from a neighbour.
#[test]
fn the_first_seat_is_the_eye_and_the_rest_fill_the_nodes_in_order() {
    let laid = tints(&[
        seat("root", Doing::Thinking),
        seat("kid-a", Doing::Tools),
        seat("kid-b", Doing::Waiting),
    ]);
    assert_eq!(laid.eye, SIGIL);
    assert_eq!(laid.nodes.first().copied(), Some(ICHOR));
    assert_eq!(laid.nodes.get(1).copied(), Some(BRAZEN));
    // Every unfilled seat rests: an absent agent and an idle one read alike.
    assert_eq!(laid.nodes.get(2).copied(), Some(HYDRA));
    assert_eq!(laid.nodes.last().copied(), Some(HYDRA));
}

/// An idle agent is green, which is the hue an *empty* seat already wears —
/// the empty case is the general path, not a branch.
#[test]
fn an_idle_agent_and_an_empty_seat_are_the_same_green() {
    let laid = tints(&[seat("root", Doing::Idle), seat("kid", Doing::Idle)]);
    assert_eq!(laid.eye, HYDRA);
    assert!(laid.nodes.iter().all(|hue| *hue == HYDRA));
}

/// The roster names every seat and says what it is doing — the words behind
/// the hues, which may never carry a fact alone (§11 glyph doctrine).
#[test]
fn the_roster_names_every_seat_and_says_what_it_is_doing() {
    let said = roster(&[
        seat("energize", Doing::Inference),
        seat("scribe", Doing::Tools),
    ]);
    assert!(said.contains("energize — inference"), "{said}");
    assert!(said.contains("scribe — tools"), "{said}");
    assert!(!said.contains("are shown"), "nothing was dropped: {said}");
}

/// A conversation with more subagents than the mark has circles says so
/// **outright**: a cap that stays quiet reads as full coverage.
#[test]
fn overflow_past_the_mark_s_circles_is_stated_not_swallowed() {
    let mut seats = vec![seat("root", Doing::Idle)];
    for n in 0..NODE_SEATS + 4 {
        seats.push(seat(&format!("kid-{n}"), Doing::Tools));
    }
    let said = roster(&seats);
    assert!(
        said.contains(&format!("{NODE_SEATS} of {} are shown", NODE_SEATS + 4)),
        "{said}"
    );
    // The lines themselves stop at the circle count — the eye plus nine.
    assert_eq!(said.matches("↳").count(), NODE_SEATS);
}

/// Both seats paint the mark; **only the resting one paints the name.** The
/// wordmark brands a window, so it says "yog"; the live seat sits on a
/// conversation's own headline row, where that word says nothing (bl-d44e).
/// Drives the two widget bodies end to end.
#[test]
fn both_seats_paint_the_mark_and_only_the_resting_one_names_it() {
    for live in [false, true] {
        let ctx = egui::Context::default();
        let output = ctx.run(crate::paint_probe::screen(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                if live {
                    live_mark(ui, &[seat("energize", Doing::Waiting)]);
                } else {
                    wordmark(ui);
                }
            });
        });
        assert_eq!(
            crate::paint_probe::text_of(&output).contains("yog"),
            !live,
            "the name rides the seat, not the mark (live: {live})"
        );
        let mut circles = 0;
        for clipped in &output.shapes {
            count_circles(&clipped.shape, &mut circles);
        }
        assert_eq!(circles, NODE_SEATS + 1, "live: {live}");
    }
}

/// Circles in one shape tree, descending `Shape::Vec`.
fn count_circles(shape: &egui::Shape, out: &mut usize) {
    match shape {
        egui::Shape::Circle(_) => *out += 1,
        egui::Shape::Vec(shapes) => shapes.iter().for_each(|s| count_circles(s, out)),
        _ => {}
    }
}

/// The five hues are five: nothing on the mark reads as anything else on it.
#[test]
fn the_five_states_are_five_distinct_hues() {
    let all = [
        Doing::Idle,
        Doing::Waiting,
        Doing::Thinking,
        Doing::Inference,
        Doing::Tools,
    ];
    let hues: std::collections::BTreeSet<[u8; 4]> = all
        .iter()
        .map(|d| crate::theme::doing_badge(*d).0.to_array())
        .collect();
    assert_eq!(hues.len(), all.len());
    assert_eq!(crate::theme::doing_badge(Doing::Inference).0, SPECTRE);
}
