//! S19-T1 group-paint — plus S19-T2's click half (every affordance answers its
//! intent) and S19-T3's picks half (two compare unasked; picks walk): the fan
//! group card puts the cohort on screen as one group — the shared base said
//! once, each candidate's mark, figures, churn and terminal response side by
//! side — and a workspace with no fan paints no card at all. Clicks are aimed
//! by paint (`painted_settled`), never by input text.

use crate::science::compose::Intent;
use crate::science::render::{Seat, group};
use crate::science::tests::candidate;
use crate::science::{Attempt, Outcome};

fn painted(rows: &[Attempt], seat: &mut Seat) -> String {
    crate::paint_probe::paint(|ui| {
        group(ui, rows, seat);
    })
}

/// The burden check, structural: a workspace holding only the ordinary claim
/// attempt (or nothing) grows no group card.
#[test]
fn no_candidates_paints_nothing_at_all() {
    let mut seat = Seat::default();
    assert_eq!(painted(&[], &mut seat), "");
    let mut claim = candidate("at-1", None);
    claim.diff.handle = None;
    assert_eq!(painted(&[claim], &mut seat), "");
}

/// The card: the group header states the cohort once (count, ball, shared
/// base at the inspector's short width), each candidate column carries its
/// handle, mark, figures, churn and clipped response, and the two-member
/// cohort's responses are compared without a pick.
#[test]
fn the_card_states_the_cohort_and_compares_two_by_itself() {
    let rows = vec![
        candidate("at-1", Some("shared\nalpha")),
        candidate("at-2", Some("shared\nbeta")),
    ];
    let mut seat = Seat::default();
    let text = painted(&rows, &mut seat);
    assert!(text.contains("fan · 2 candidates on bl-1"), "{text}");
    assert!(text.contains("base basebas"), "{text}");
    assert!(text.contains("Judge"), "{text}");
    assert!(text.contains("Synthesize"), "{text}");
    assert!(text.contains("at-1"), "{text}");
    assert!(text.contains("pending"), "{text}");
    assert!(text.contains("3 steps · 1m01s wall · 77 tokens"), "{text}");
    assert!(text.contains("+4 −1 across 1 files"), "{text}");
    assert!(text.contains("shared\nalpha"), "{text}");
    assert!(text.contains("Deliver"), "{text}");
    assert!(text.contains("Retire"), "{text}");
    // V3.3's response diff, unasked: the only pair there is.
    assert!(text.contains("response diff · − at-1 · + at-2"), "{text}");
    assert!(text.contains("  shared"), "{text}");
    assert!(text.contains("− alpha"), "{text}");
    assert!(text.contains("+ beta"), "{text}");
}

/// Every derived mark in words, and the honest absences: a candidate with no
/// bound conversation and one that has said nothing say so.
#[test]
fn every_outcome_and_absence_is_said_in_words() {
    let mut delivered = candidate("at-1", Some("won"));
    delivered.outcome = Outcome::Accepted {
        commit: "cafecafe1234beef".to_owned(),
    };
    let mut stale = candidate("at-2", Some("lost"));
    stale.outcome = Outcome::Rejected {
        by: Some("at-1".to_owned()),
    };
    stale.wall_secs = 3905;
    let mut discarded = candidate("at-3", None);
    discarded.outcome = Outcome::Rejected { by: None };
    discarded.conversation = None;
    let mut reworked = candidate("at-4", None);
    reworked.outcome = Outcome::Reworked;
    reworked.wall_secs = 5;
    let mut seat = Seat::default();
    let text = painted(&[delivered, stale, discarded, reworked], &mut seat);
    assert!(text.contains("delivered cafecaf"), "{text}");
    assert!(text.contains("stale — at-1 delivered"), "{text}");
    assert!(text.contains("discarded"), "{text}");
    assert!(text.contains("reworked"), "{text}");
    assert!(text.contains("no conversation bound yet"), "{text}");
    assert!(text.contains("nothing said yet"), "{text}");
    // The wall spelling at every magnitude an operator reads.
    assert!(text.contains("1h05m wall"), "{text}");
    assert!(text.contains("5s wall"), "{text}");
    // Four members and no picks: nothing compares until two are picked.
    assert!(!text.contains("response diff"), "{text}");
    // Intact records say nothing about compaction — the general path.
    assert!(!text.contains("record compacted"), "{text}");
}

/// A compacted record is stated on the card (bl-fde5): the figures the operator
/// judges by were read over a rewritten conversation, and that bound must be on
/// the same surface as the figures.
#[test]
fn a_compacted_record_is_stated_beside_its_figures() {
    let mut squashed = candidate("at-1", Some("done"));
    squashed.compacted = 12;
    let mut seat = Seat::default();
    let text = painted(&[squashed], &mut seat);
    assert!(
        text.contains("record compacted — 12 entries gone"),
        "{text}"
    );
}

/// A long response is clipped in code (the paint layer must never rely on an
/// egui elision it cannot see), a mixed-base cohort states no shared base and
/// each candidate's churn absences say what is missing.
#[test]
fn clipping_mixed_bases_and_churn_absences_are_honest() {
    let mut long = candidate("at-1", Some(&"x".repeat(400)));
    long.base = None;
    // A binary change counts as a file and adds no lines.
    if let crate::workdiff::Change::Diff { files, .. } = &mut long.diff.change {
        files.push(crate::workdiff::FileChurn {
            path: "logo.png".to_owned(),
            churn: crate::workdiff::Churn::Binary,
        });
    }
    let mut absent = candidate("at-2", None);
    absent.diff.change = crate::workdiff::Change::Absent {
        target: "work/bl-1".to_owned(),
        source: "attempt/at-2".to_owned(),
        missing: vec!["attempt/at-2".to_owned()],
    };
    let mut unreadable = candidate("at-3", None);
    unreadable.diff.change = crate::workdiff::Change::Unreadable;
    let mut seat = Seat::default();
    let text = painted(&[long, absent, unreadable], &mut seat);
    assert!(!text.contains("base basebas"), "{text}");
    assert!(text.contains("+4 −1 across 2 files"), "{text}");
    assert!(text.contains(&format!("{}…", "x".repeat(280))), "{text}");
    assert!(text.contains("no attempt/at-2 yet"), "{text}");
    assert!(text.contains("project unreadable"), "{text}");
}

/// Picks drive the diff for cohorts past two: two picked handles compare, and
/// a third pick replaces the elder so the pair walks. The truncation note
/// rides the diff when a compared response was cut.
#[test]
fn picks_compare_and_a_third_pick_replaces_the_elder() {
    let rows = vec![
        candidate("at-1", Some(&"l\n".repeat(500))),
        candidate("at-2", Some("r")),
        candidate("at-3", Some("s")),
    ];
    let mut seat = Seat {
        compare: vec!["at-1".to_owned(), "at-2".to_owned()],
        intent: None,
    };
    let text = painted(&rows, &mut seat);
    assert!(text.contains("response diff · − at-1 · + at-2"), "{text}");
    // The truncation note sits under 400 diff rows — past the ordinary probe's
    // 4096-point clip — so this one assertion reads a taller frame.
    let tall = crate::paint_probe::painted_settled(1600.0, 20000.0, |ui| {
        group(
            ui,
            rows.as_slice(),
            &mut Seat {
                compare: vec!["at-1".to_owned(), "at-2".to_owned()],
                intent: None,
            },
        );
    });
    assert!(
        tall.iter()
            .any(|(text, _)| text.contains("responses longer than this comparison reads")),
        "no truncation note in the tall frame"
    );
    // The walk: pick at-3 with two already picked — the elder (at-1) leaves.
    click(&rows, &mut seat, "compare", 2);
    assert_eq!(seat.compare, vec!["at-2".to_owned(), "at-3".to_owned()]);
    // Unpick: the same label on a picked candidate releases it.
    click(&rows, &mut seat, "compare", 1);
    assert_eq!(seat.compare, vec!["at-3".to_owned()]);
    let text = painted(&rows, &mut seat);
    assert!(!text.contains("response diff"), "{text}");
}

/// The affordances answer as intents: Judge and Synthesize on the group,
/// Deliver and Retire on their own candidate.
#[test]
fn every_affordance_answers_its_intent() {
    let rows = vec![candidate("at-1", None), candidate("at-2", None)];
    let mut seat = Seat::default();
    click(&rows, &mut seat, "Judge", 0);
    assert_eq!(seat.intent, Some(Intent::Judge));
    click(&rows, &mut seat, "Synthesize", 0);
    assert_eq!(seat.intent, Some(Intent::Synthesize));
    click(&rows, &mut seat, "Deliver", 1);
    assert_eq!(
        seat.intent,
        Some(Intent::Deliver {
            handle: "at-2".to_owned()
        })
    );
    click(&rows, &mut seat, "Retire", 0);
    assert_eq!(
        seat.intent,
        Some(Intent::Retire {
            handle: "at-1".to_owned()
        })
    );
}

/// Click the `nth` galley whose text is exactly `label`, aimed by the settled
/// paint — never by a coordinate or the input string.
fn click(rows: &[Attempt], seat: &mut Seat, label: &str, nth: usize) {
    let painted = crate::paint_probe::painted_settled(1600.0, 4096.0, |ui| {
        group(
            ui,
            rows,
            &mut Seat {
                compare: seat.compare.clone(),
                intent: None,
            },
        );
    });
    let hit = painted.iter().filter(|(text, _)| text == label).nth(nth);
    assert!(hit.is_some(), "{label} #{nth} is on screen");
    let pos = hit.map(|(_, rect)| rect.center()).unwrap_or_default();
    let ctx = egui::Context::default();
    let button = |pressed| egui::Event::PointerButton {
        pos,
        button: egui::PointerButton::Primary,
        pressed,
        modifiers: egui::Modifiers::default(),
    };
    for events in [vec![], vec![button(true)], vec![button(false)], vec![]] {
        let input = egui::RawInput {
            events,
            ..crate::paint_probe::screen_sized(1600.0, 4096.0)
        };
        let _ = ctx.run(input, |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                group(ui, rows, seat);
            });
        });
    }
}
