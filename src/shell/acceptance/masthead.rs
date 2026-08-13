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
use crate::paint_probe::seen_of;

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
///
/// It deliberately does **not** pin their horizontal alignment. bl-fb1c is open
/// on exactly that (the wordmark is left-aligned while the two lines below it
/// are centred, because `theme::wordmark` is a `ui.horizontal` and
/// `vertical_centered` centres a child by the width it requests), and an
/// assertion written to pass today would have to encode the defect. This is the
/// fixture that ball needs; the alignment claim is its to add.
#[test]
fn the_empty_worlds_masthead_stacks_three_whole_runs_in_order() {
    for (w, h) in super::SIZES {
        let (lernie, bl, bz) = (Cli::new("lernie"), Cli::new("bl"), Cli::new("bz"));
        let mut world = world_empty();
        let ctx = egui::Context::default();
        let mut frame = || {
            ctx.run(crate::paint_probe::screen_sized(w, h), |ctx| {
                super::super::render(ctx, &mut world.model, &mut world.state, &lernie, &bl, &bz);
            })
        };
        for _ in 0..3 {
            let _ = frame();
        }
        let painted = seen_of(&frame());
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
        };
        // The wordmark is matched **whole**, not by prefix: the side panel's
        // balls hint paints `yog exec bl prime`, which a prefix match takes for
        // the mark and then reports as a masthead out of order — a needle loose
        // enough to hit the wrong surface is the same class of defect this file
        // exists for. The §3.3 prediction is matched on its stem instead of the
        // minted word, so this stays a masthead assertion rather than a second
        // copy of the wordlist pin (`super::mint_seed` owns that).
        let stack = [
            line("the wordmark", &|text: &str| text == "yog"),
            line("the tagline", &|text: &str| text == crate::theme::TAGLINE),
            line("the name prediction", &|text: &str| {
                text.starts_with("will be named ")
            }),
        ];
        for seen in stack {
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
