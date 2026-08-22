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
mod first_run;
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
mod orphan;
mod overlap;
mod picker;
mod raise;
mod reach;
mod recall;
mod refusal;
mod remedies;
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
mod width;
mod wire;
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
    let frame = |world: &mut World| {
        ctx.run(input(), |ctx| {
            render(ctx, &mut world.model, &mut world.state, lernie, bl, &bz);
        })
    };
    // Two frames before anything is settled, because a **fresh** `egui::Context`
    // measures before it paints: egui sizes a content-sized panel and culls a
    // scroll area against the *previous* frame's rect, so a surface read off
    // the first frame on a new context is read off an unmeasured layout.
    // [`Screen`](screen::Screen) needs no such pre-roll — its context is
    // persistent across the whole drive, which is the point of it.
    let _ = frame(world);
    let _ = frame(world);
    // Then the wire settled to a **fixed point** (`World::drain`, REMOTE §9.8's
    // harness ruling as bl-44e9 extended it to reads): every migrated surface
    // paints an answer that landed a round trip later, and the §11 step
    // drill-in two — its sequence name is picked out of the step list that
    // landed, which is why counting passes stopped being enough (bl-13f9).
    world.drain(&mut |world| {
        let _ = frame(world);
    });
    // **Then the queue region's own measurement, which the wire cannot settle**
    // (bl-b4b5, `Frames::settle`'s trailing frames for the same reason). The
    // composer's fold line is last frame's *painted* content height eased over
    // `i.time` (bl-929d), and since the pending listing became `Query::Inbox`'
    // answer that content changes on the frame the answer lands — so the fixed
    // point the drain reaches is a frame where the region is still easing.
    //
    // These are **clock** frames, not wire passes, and deliberately settle
    // nothing: an animation settles by time passing, and a settle between them
    // would close the §7.3 wound gate's window, which reads `false` for any
    // frame the steps answer has not reached (`app::grace`).
    for _ in 0..4 {
        let _ = frame(world);
    }
    // And one last frame, laid out against the settled one: that is the frame
    // the test reads, exactly as the operator's eye reads the repaint after the
    // answer rather than the one it arrived on. **Nothing is settled between
    // the two** — a `Link` settled twice without a frame between it declares
    // nothing and drops every answer, which is the same rule that makes a
    // collapsed pane free.
    crate::paint_probe::text_of(&frame(world))
}

/// The window sizes the paint-layer properties are asserted at: yog's
/// documented `min_inner_size` (`src/main.rs`), the audit's default capture, an
/// ordinary half-screen, and a maximized 4K pane — the small end and the large
/// end of QUALITY §2's shot sheet plus the two in between. One list, because a
/// property asserted at a size its sibling skips is a hole neither can see.
///
/// **480x1400 is a size *class*, not a fifth measurement** (bl-7414): a tiled
/// left third of a portrait monitor. Every other entry is landscape, so until
/// this one the suite had never rendered a window where height is abundant and
/// width is scarce — and the four rules that only bite there had never been
/// asked. It reddened `legible` on sight at three independent seats (the
/// lernie-global pane's workflow and declare rows, the config-branch pane's
/// lineage and file rows, and the marks pane's verb pair, each claiming egui's
/// fixed 280 pt `text_edit_width` in a 224 pt centre) and a fourth in the Steps
/// table, whose `Commit` heading was laid six points past the pane's clip so
/// that the `…` marking its own truncation fell outside it. A row that
/// overflows does not merely overflow: it ratchets the seat's `max_rect`, so
/// every row beneath it elides to a width that is not there.
pub(super) const SIZES: [(f32, f32); 5] = [
    (420.0, 320.0),
    (480.0, 1400.0),
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
    let frame = |world: &mut World| {
        ctx.run(crate::paint_probe::screen_sized(w, h), |ctx| {
            render(ctx, &mut world.model, &mut world.state, &lernie, &bl, &bz);
        })
    };
    // The wire settled between frames, for `painted`'s reason: a migrated
    // surface has nothing to lay out until its answer has landed, and a geometry
    // property asserted over a blank column is a property asserted over nothing.
    for _ in 0..4 {
        let _ = frame(&mut world);
        world.settle();
    }
    frame(&mut world)
}
