//! The conversation pane's **bottom stack, in its ruled order**
//! (§11 bottom accessories, bl-58e4) — read off the composed frame, at every
//! window size the audit captures.
//!
//! The ruling: the work directory, the budget, the context and the model
//! selection must not sit between the input bar and the chat — those elements
//! belong below the input box, not above it. Those four elements are the settings band, and the claim has two
//! signs, which is why neither half alone is the test: the **settings band is
//! below** the goal box, *and* the **in-flight strip is still above** it —
//! bl-905f's seat is untouched, so a fix that swept the whole stack below the
//! input would satisfy the ruling's letter and break an older one.
//!
//! Stated as one reading order ([`ORDER`]) rather than a pair of inequalities:
//! the defect this file exists for was a band on the wrong side of another, and
//! a list of bands top-to-bottom is that fact with nothing else in it.
//!
//! Sized, because this surface is repeatedly right at one size and wrong at
//! another (bl-7414 narrow-tall, bl-5410 at 420x320 and 800x500) — and
//! especially here, since the bands are *budgeted* (§11 rule 5), so which of
//! them the pane can seat at all changes with the window.

use super::super::render;
use super::fixture::world;
use crate::cli_outbound::Cli;

/// The bands of the conversation pane's bottom stack, **top to bottom on
/// screen**. This list is the ruling.
const ORDER: [&str; 3] = ["flight-strip", "composer", "conversation-settings"];

/// Which bands each window size actually seats, and in what order — pinned
/// outright rather than asserted as "at least the composer", because a band the
/// budget silently stopped paying for is exactly the regression the ordering
/// check would then pass over in silence.
///
/// **The 420x320 row is unchanged by bl-58e4** and was measured against the
/// stack as it stood before it: the documented minimum has never had the room
/// for all three bands, and the strip is the one §11 rule 5 declines to seat
/// there. Reordering the stack did not move that line, which is the other half
/// of "bl-905f is untouched" — the strip did not lose a seat it had.
/// **The 480x1400 row is the narrow-tall size class** (bl-7414): a tiled left
/// third of a portrait monitor, and the one size in the list where height is
/// abundant and width is not. It seats all three bands — the strip's seat is a
/// question of vertical budget (§11 rule 5), and 1400 pt has room to spare —
/// which is the point of pinning it: the size was added to catch *width*
/// defects, and this row states outright that it cost the stack nothing.
const CENSUS: [&str; 5] = [
    "420x320: [\"composer\", \"conversation-settings\"]",
    "480x1400: [\"flight-strip\", \"composer\", \"conversation-settings\"]",
    "800x500: [\"flight-strip\", \"composer\", \"conversation-settings\"]",
    "1150x760: [\"flight-strip\", \"composer\", \"conversation-settings\"]",
    "2560x1700: [\"flight-strip\", \"composer\", \"conversation-settings\"]",
];

/// Every band [`ORDER`] names that a frame actually seated, in that order, with
/// the rect egui stored for it.
fn seats(ctx: &egui::Context) -> Vec<(&'static str, egui::Rect)> {
    ORDER
        .iter()
        .filter_map(|id| {
            egui::containers::panel::PanelState::load(ctx, egui::Id::new(*id))
                .map(|state| (*id, state.rect))
        })
        .collect()
}

/// Pairs of bands out of the ruled order, enumerated — the defect, said as
/// arithmetic. The tolerance is a point: docked panels abut by design, and two
/// bands sharing a seam are in order, not on top of each other.
fn out_of_order(seated: &[(&'static str, egui::Rect)]) -> Vec<String> {
    seated
        .windows(2)
        .filter_map(|pair| match pair {
            [(above, upper), (below, lower)] if upper.bottom() > lower.top() + 1.0 => {
                Some(format!(
                    "{above} (bottom {}) is not above {below} (top {})",
                    upper.bottom(),
                    lower.top()
                ))
            }
            _ => None,
        })
        .collect()
}

/// The whole window at `w` x `h` with something really in flight in the open
/// conversation — the state that puts all three bands on screen at once.
///
/// The band ids are wiped from the context before the last frame, so what is
/// read back is what **this** frame seated: a panel egui was shown on an early
/// frame keeps its stored rect forever, and at the small sizes the settling
/// frames are exactly where a band the budget cannot ultimately pay for gets
/// seated once and then dropped.
fn stack(w: f32, h: f32) -> Vec<(&'static str, egui::Rect)> {
    let (lernie, bl, bz) = (
        Cli::new("yog-absent-lernie"),
        Cli::new("yog-absent-bl"),
        Cli::new("yog-absent-bz"),
    );
    let mut world = super::inbox_composer::quick(world());
    let ws = world.ws.clone();
    world.model.focus_agent(&ws, "c-1");
    // The driver as the §5.1 #28 probes actually see it: the inbox-dir lock fd
    // and the response.json writer fd, held by this very process.
    let _lock = std::fs::File::open(ws.join("inbox/c-1")).unwrap();
    let _writer = std::fs::OpenOptions::new()
        .append(true)
        .open(ws.join("steps/c-1/001/response.json"))
        .unwrap();
    super::inbox_composer::converge_ws(&mut world);
    let ctx = egui::Context::default();
    let mut frame = || {
        let _ = ctx.run(crate::paint_probe::screen_sized(w, h), |ctx| {
            render(ctx, &mut world.model, &mut world.state, &lernie, &bl, &bz);
        });
    };
    for _ in 0..5 {
        frame();
    }
    ctx.data_mut(|data| {
        for id in ORDER {
            data.remove::<egui::containers::panel::PanelState>(egui::Id::new(id));
        }
    });
    frame();
    seats(&ctx)
}

/// The ruling, at every size: the settings band below the input box, the
/// in-flight strip above it, and the same bands seated at each size as the day
/// this was written.
#[test]
fn the_settings_band_reads_below_the_input_box_at_every_window_size() {
    let mut census = Vec::new();
    let mut report = Vec::new();
    for (w, h) in super::SIZES {
        let seated = stack(w, h);
        let names: Vec<&str> = seated.iter().map(|(id, _)| *id).collect();
        census.push(format!("{w:.0}x{h:.0}: {names:?}"));
        for pair in out_of_order(&seated) {
            report.push(format!("at {w:.0}x{h:.0}: {pair}"));
        }
    }
    // Every size at once: a resize defect is a shape across sizes, and stopping
    // at the first hides which of them a fix actually moved.
    assert!(report.is_empty(), "{}", report.join("\n"));
    assert_eq!(census, CENSUS, "which bands each size seats has changed");
}

/// The reader bites — the other direction of the same discipline `make
/// rules-audit` runs on its fixtures. The check above is the whole evidence for
/// the claim above it, so it is shown a frame laid out **the way the stack
/// stood before this ball**: the composer at the pane's bottom edge, the
/// settings rows above it, the strip innermost. That frame must be reported.
#[test]
fn the_reader_reports_the_order_the_ruling_retired() {
    let ctx = egui::Context::default();
    let _ = ctx.run(super::input(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            for id in ["composer", "conversation-settings", "flight-strip"] {
                egui::TopBottomPanel::bottom(id).show_inside(ui, |ui| {
                    ui.label(id);
                });
            }
        });
    });
    let found = out_of_order(&seats(&ctx));
    assert!(
        found
            .iter()
            .any(|pair| pair.contains("composer") && pair.contains("conversation-settings")),
        "the retired order must be reported: {found:?}"
    );
}
