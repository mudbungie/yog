//! Full-window acceptance smoke (§11): render the whole shell — the top bar
//! (attention strip + tab bar), the conversation list, the selected
//! conversation center with its inline Login banner, the composer, the
//! activity accessory, and the per-agent inspector — over a populated fixture
//! workspace, cycling every inspector tab and config mode, and assert each
//! data surface reaches the paint layer. This drives the coverage-excluded
//! shell glue end to end, so it is the reachability proof that complements the
//! per-widget render tests.

mod alerts;
mod bands;
mod birth;
mod drafts;
mod drift;
mod echo;
mod elision;
mod fixture;
mod floor;
mod focus;
mod geometry;
mod hover;
mod inbox_composer;
mod legible;
mod masthead;
mod mint_seed;
mod modal;
mod name_column;
mod naming;
mod one_rendering;
mod overlap;
mod picker;
mod raise;
mod recall;
mod screen;
mod search_tab;
mod settings;
mod slash;
mod smoke;
mod start_draft;
mod started;
mod tabs;
mod unfold;
mod walk;
mod walls;
mod wound;

use super::render;
use crate::cli_outbound::Cli;
use fixture::World;

/// The full-window geometry this smoke test asserts against — deliberately
/// wider and shorter than the [`crate::paint_probe::screen`] default, because
/// §11's three columns only all lay out above a minimum width.
fn input() -> egui::RawInput {
    crate::paint_probe::screen_sized(1600.0, 2400.0)
}

/// Render the whole window and return every painted galley's text. Three
/// frames: egui panels are their default height on the frame they first appear
/// (the content height lands in panel state for the next frame), and the
/// inbox-composer's queue region adds one more one-frame settle of its own
/// (bl-929d — its measured content height feeds the panel's, so the chrome
/// below the region reaches its final seat on the frame after the panel
/// adopts it). The settled third frame is the one the operator actually sees.
fn painted(world: &mut World, lernie: &Cli, bl: &Cli) -> String {
    // A `bz` handle for the §8.3 Login surface; unspawned here — the Toolchain
    // pane only shells to bz on an explicit click, unreachable headless.
    let bz = Cli::new("bz");
    let ctx = egui::Context::default();
    let mut frame = || {
        ctx.run(input(), |ctx| {
            render(ctx, &mut world.model, &mut world.state, lernie, bl, &bz);
        })
    };
    let _ = frame();
    let _ = frame();
    let out = frame();
    crate::paint_probe::text_of(&out)
}

/// The window sizes the paint-layer properties are asserted at: yog's
/// documented `min_inner_size` (`src/main.rs`), the audit's default capture, an
/// ordinary half-screen, and a maximized 4K pane — the small end and the large
/// end of QUALITY §2's shot sheet plus the two in between. One list, because a
/// property asserted at a size its sibling skips is a hole neither can see.
pub(super) const SIZES: [(f32, f32); 4] = [
    (420.0, 320.0),
    (800.0, 500.0),
    (1150.0, 760.0),
    (2560.0, 1700.0),
];

/// The whole window at `w` x `h`, settled, over the populated fixture — the one
/// sized render the paint-layer *properties* are asserted against (`overlap`'s
/// disjointness and `legible`'s marked-as-cut). Both ask a question of the same
/// frame, so both must be looking at the same frame: a property that held on a
/// bespoke fixture and failed on the shipped one would be no property at all.
///
/// Enough frames to settle: panel heights, the tail idiom's pad and the
/// inbox-composer's fold line each land a frame after the content they measure,
/// and what an operator sees is the steady state, not any one of those frames.
pub(super) fn window(
    w: f32,
    h: f32,
    trail: bool,
    tab: crate::keymap::CenterTab,
    inspector: crate::keymap::InspectorTab,
) -> egui::FullOutput {
    let (lernie, bl, bz) = (Cli::new("lernie"), Cli::new("bl"), Cli::new("bz"));
    let mut world = fixture::world();
    let ws = world.ws.clone();
    world.model.focus_agent(&ws, "c-1");
    world.model.select_tab(inspector);
    world.state.activity_open = trail;
    let ctx = egui::Context::default();
    super::center::focus(&world.model, &mut world.state, tab);
    let mut frame = || {
        ctx.run(crate::paint_probe::screen_sized(w, h), |ctx| {
            render(ctx, &mut world.model, &mut world.state, &lernie, &bl, &bz);
        })
    };
    for _ in 0..4 {
        let _ = frame();
    }
    frame()
}
