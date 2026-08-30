//! **The prefix is one cell, and it is the only thing left of the title**
//! (§11, bl-8257) — the completion of the rule [`super`] pins for the attention
//! flag, split from it at §12's budget on the seam the two already had: that
//! beat asks whether one conditional mark moves the column, over a **derived**
//! world through the whole window; this one asks it of every conditional mark
//! there is, over rows built by hand and painted through `conv_row` alone.

use super::super::super::conv_row;
use super::super::fixture::world_titled;
use crate::cli_outbound::Cli;
use crate::monitor::{Check, Verdict};
use crate::nav::convs::{ConvRow, Flight};

/// Four rows differing in **exactly one** conditional mark apiece — plain, §10
/// uncertain, in flight, and carrying an alignment verdict — named so each
/// title is its own needle. Built by hand rather than derived: an uncertain
/// state comes from a [`Probe::Unknown`](crate::git_tree) answer on an injected
/// liveness probe, which a whole-window fixture cannot produce, and the claim
/// under test is about one row's layout rather than about the window.
fn one_conditional_each() -> [ConvRow; 4] {
    let base = |name: &str| ConvRow {
        root_id: format!("c-{name}"),
        state: crate::git_tree::AgentState::Quiescent,
        uncertain: false,
        preview: String::new(),
        age_secs: 0,
        flight: None,
        attention: 0,
        members: 1,
        depth: 0,
        direct: 0,
        stoppable: false,
        stop_children: false,
        ball: None,
        name: Some(name.to_owned()),
        name_display_only: false,
        verdict: None,
        tone: crate::transcript::Tone::Plain,
    };
    [
        base("plain"),
        ConvRow {
            uncertain: true,
            ..base("uncertain")
        },
        ConvRow {
            flight: Some(Flight::Tools),
            ..base("flying")
        },
        ConvRow {
            verdict: Some(Check {
                workspace: "w".into(),
                agent: "a".into(),
                verdict: Verdict::Drifting,
                sha: "deadbeef".into(),
                reason: "wandered".into(),
                model: "m".into(),
                input_tokens: None,
                output_tokens: None,
            }),
            ..base("judged")
        },
    ]
}

/// **The prefix is one cell, and it is the only thing left of the title**
/// (§11, bl-8257) — the completion of the rule the beat above pins for the
/// attention flag.
///
/// Three conditional elements outlived bl-b9e3 in the prefix: the live-activity
/// chip, the §10 uncertainty `?` and the alignment verdict badge. Each moved the
/// name column on exactly the rows that had it, and the two that are
/// *independent* marks (chip, verdict) now ride the trailing group while the
/// `?` — a suffix on the badge it qualifies, not a mark of its own — keeps its
/// seat in a fixed-width slot that is allocated whether or not it paints.
///
/// Asserted over four rows differing in **exactly one** conditional apiece,
/// painted through the real `conv_row::conversation_row`. Read with
/// [`seen_of`](crate::paint_probe::seen_of) rather than the laid rects (bl-36c3):
/// a mark clipped away by its seat is not on the glass, so a beat that measured
/// where it was *laid* would pass on a row the operator cannot see it in.
///
/// Both directions, because equal edges alone are satisfied by a tree that
/// simply stopped painting all three: each mark is asserted **present** on its
/// own row, **absent** from the plain one, and **right of every title**.
#[test]
fn no_conditional_mark_moves_the_name_column_and_all_of_them_ride_right() {
    let built = one_conditional_each();
    let rows: Vec<&ConvRow> = built.iter().collect();

    let litany = Cli::new("litany");
    let mut world = world_titled("hello");
    world.converge();
    let ctx = egui::Context::default();
    // Narrow enough that the row is genuinely width-bound (bl-9669), so
    // "pinned right" is a claim about a real edge rather than about slack.
    let mut frame = || {
        ctx.run(crate::paint_probe::screen_sized(360.0, 400.0), |c| {
            egui::CentralPanel::default().show(c, |ui| {
                let row_ctx = conv_row::RowCtx::of(&mut world.model, world.ws.clone());
                for row in &rows {
                    conv_row::conversation_row(
                        ui,
                        &mut world.model,
                        &mut world.state,
                        &litany,
                        row,
                        &row_ctx,
                    );
                }
            });
        })
    };
    let _ = frame();
    let seen = crate::paint_probe::seen_of(&frame());

    let shown = |needle: &str| {
        seen.iter()
            .filter(|s| s.text == needle)
            .map(|s| s.shown)
            .reduce(|a, b| if a.left() <= b.left() { a } else { b })
    };
    let title = |name: &str| {
        shown(name).unwrap_or_else(|| {
            panic!(
                "row {name:?} must paint its title:\n{:?}",
                seen.iter().map(|s| &s.text).collect::<Vec<_>>()
            )
        })
    };

    // One column: four rows, four conditional states, one left edge.
    let edges: Vec<(&str, f32)> = ["plain", "uncertain", "flying", "judged"]
        .into_iter()
        .map(|n| (n, title(n).left()))
        .collect();
    let first = edges[0].1;
    for (name, left) in &edges {
        assert!(
            (left - first).abs() < 0.5,
            "the title's left edge is the prefix's, not any conditional mark's: \
             {edges:?} — {name} sits at {left}, not {first}"
        );
    }

    // And every conditional mark really is painted, on its own row and right of
    // every title — without this the equality above holds on a tree that simply
    // deleted all three.
    let (chip, _, _) = crate::theme::flight_badge(Flight::Tools);
    let (badge, _, _) = crate::theme::verdict_badge(Verdict::Drifting);
    for (mark, owner) in [(chip, "flying"), (badge, "judged"), ("?", "uncertain")] {
        let seat = shown(mark)
            .unwrap_or_else(|| panic!("{owner} must paint {mark:?} somewhere on the glass"));
        let row = title(owner);
        assert!(
            (seat.center().y - row.center().y).abs() < row.height(),
            "{mark:?} belongs to {owner}'s row: {seat:?} vs {row:?}"
        );
        assert!(
            seat.left() > first,
            "{mark:?} rides the trailing group, right of every title: {} <= {first}",
            seat.left()
        );
    }
    // And the plain row wears none of them — without this, "right of every
    // title" is satisfied by a frame that paints all three on one row.
    let plain_row = title("plain");
    for mark in [chip, badge, "?"] {
        assert!(
            !seen.iter().any(|s| s.text == mark
                && (s.shown.center().y - plain_row.center().y).abs() < plain_row.height()),
            "the plain row carries no {mark:?}"
        );
    }
}
