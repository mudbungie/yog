//! **The empty world's masthead** (§3.4, STORIES S0) — the first surface an
//! operator ever meets, and until bl-37bf one no test could reach.
//!
//! `shell::bootstrap` paints only when `focused_workspace()` is `None`, and no
//! fixture produced that: `world_unfocused` withheld the startup-focus
//! *argument* while leaving the workspace in the roster, so
//! `AppModel::startup_focus` derived a focus onto it and the centre painted the
//! conversation view. The surface is coverage-excluded shell glue, so nothing
//! reported the gap, and the one test named for it passed on the start pane's
//! box. `fixture::world_empty` is the fixture that was missing.
//!
//! Split from [`super::legible`] at §12's per-file budget, on the seam the two
//! files really divide on: that one asserts a **property** over whatever the
//! shipped frame happens to paint, this one asserts a **surface** — three named
//! runs, in one order, over a fixture built for it.

use super::fixture::world_empty;
use crate::cli_outbound::Cli;
use crate::paint_probe::{Seen, seen_of};

/// How far two runs' centres may differ and still be called one axis: less
/// than a glyph, so nothing that is centred by a different rule than its
/// neighbour can hide inside it. The defect this pins was ~435 pt wide.
const ONE_AXIS: f32 = 2.0;

/// The settled empty-world frame at `w` x `h`, as painted galleys.
fn masthead(w: f32, h: f32) -> Vec<Seen> {
    let (litany, bl, bz) = (Cli::new("litany"), Cli::new("bl"), Cli::new("bz"));
    let mut world = world_empty();
    let ctx = egui::Context::default();
    let mut frame = || {
        ctx.run(crate::paint_probe::screen_sized(w, h), |ctx| {
            super::super::render(ctx, &mut world.model, &mut world.state, &litany, &bl, &bz);
        })
    };
    for _ in 0..3 {
        let _ = frame();
    }
    seen_of(&frame())
}

/// The masthead's three runs, in the order they are meant to read.
///
/// The wordmark is matched **whole**, not by prefix: the side panel's balls
/// hint paints `yog exec bl prime`, which a prefix match takes for the mark and
/// then reports as a masthead out of order — a needle loose enough to hit the
/// wrong surface is the same class of defect this file exists for. The §3.3
/// prediction is matched on its stem instead of the minted word, so this stays
/// a masthead assertion rather than a second copy of the wordlist pin
/// (`super::mint_seed` owns that).
fn stack(painted: &[Seen], w: f32, h: f32) -> [Seen; 3] {
    let line = |what: &str, hit: &dyn Fn(&str) -> bool| {
        painted
            .iter()
            .find(|seen| hit(&seen.text))
            .unwrap_or_else(|| {
                panic!(
                    "{what} is not on the empty world's masthead at {w}x{h}: {:?}",
                    painted.iter().map(|s| &s.text).collect::<Vec<_>>()
                )
            })
            .clone()
    };
    [
        line("the wordmark", &|text: &str| text == "yog"),
        line("the tagline", &|text: &str| text == crate::theme::TAGLINE),
        line("the name prediction", &|text: &str| {
            text.starts_with("will be named ")
        }),
    ]
}

/// **The empty world's masthead is three stacked runs, each whole** (§3.4,
/// STORIES S0) at every window size — the first surface an operator ever meets,
/// and until bl-37bf one no test could reach: `world_unfocused` left the
/// workspace in the roster, so `startup_focus` derived a focus onto it and the
/// centre painted the conversation view instead.
///
/// Asserted on **order and position**, not only on presence. The three lines are
/// a wordmark, what yog is, and the name the Enter would mint, and they mean
/// that in that order; a string assertion cannot tell the masthead from the same
/// three strings shuffled, or from two of them painted on one line. That is
/// bl-36c3's vacuity shape 3 — *pinning what is painted while the defect is
/// where it is painted* — and this surface is the one it was catalogued from.
#[test]
fn the_empty_worlds_masthead_stacks_three_whole_runs_in_order() {
    for (w, h) in super::SIZES {
        let painted = masthead(w, h);
        let stack = stack(&painted, w, h);
        for seen in &stack {
            assert!(
                seen.shown.width() >= seen.laid.width() - 1.0,
                "the masthead's `{}` is cut at {w}x{h}: {} pt of {} pt shown",
                seen.text,
                seen.shown.width(),
                seen.laid.width()
            );
        }
        for pair in stack.windows(2) {
            assert!(
                pair[1].laid.top() >= pair[0].laid.bottom() - 1.0,
                "the masthead is out of order at {w}x{h}: `{}` at y {} is not below \
                 `{}` ending at y {}",
                pair[1].text,
                pair[1].laid.top(),
                pair[0].text,
                pair[0].laid.bottom()
            );
        }
    }
}

/// **One masthead, one alignment axis** (bl-fb1c, QUALITY G3): the wordmark
/// lockup shares a centre with the tagline and the name prediction below it.
///
/// The defect this pins is `theme::wordmark` having been a bare
/// `ui.horizontal`, which claims the full available width — so
/// `vertical_centered` had nothing to centre and laid the mark and its word
/// against the panel's left edge (x≈270) while the two lines beneath them
/// centred (x≈705), one masthead on two axes.
///
/// The claim is made about the **lockup**, not the word: only the word reaches
/// the paint layer as text — the mark is circles, which carry no galley — so
/// the row's true left edge is the word's minus [`crate::theme::WORDMARK_LEAD`]
/// (the mark's width plus its gap). Centring the *word* on the panel would push
/// the mark off-axis to the left and is not what a centred masthead means.
#[test]
fn the_masthead_is_centred_on_one_axis_and_the_prediction_is_set_apart() {
    for (w, h) in super::SIZES {
        let painted = masthead(w, h);
        let [word, tagline, prediction] = stack(&painted, w, h);
        // The lockup is the word with the mark restored to its left.
        let lockup = word.laid.center().x - crate::theme::WORDMARK_LEAD / 2.0;
        for seen in [&tagline, &prediction] {
            assert!(
                (lockup - seen.laid.center().x).abs() <= ONE_AXIS,
                "the masthead splits across two axes at {w}x{h}: the wordmark lockup \
                 centres at x {lockup} while `{}` centres at x {}",
                seen.text,
                seen.laid.center().x
            );
        }
        // And the prediction is its own line, not the tail of the tagline: two
        // `ui.weak` runs of one size and colour separated by the stack's bare
        // ~3 pt read as one wrapped sentence — "the key and the gate will be
        // named growing".
        let apart = prediction.laid.top() - tagline.laid.bottom();
        assert!(
            apart >= super::super::bootstrap::SAID_APART,
            "`{}` runs straight on from the tagline at {w}x{h}: {apart} pt between them",
            prediction.text
        );
    }
}
