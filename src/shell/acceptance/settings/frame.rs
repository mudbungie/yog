//! **The settled frame the settings seat is measured on** — one persistent
//! `egui::Context` whose panel rects can be read back by id, and the galley
//! locators over what it painted. Split from [`super`] at §12's budget on the
//! seam the two halves already had: this is how a frame is *settled and read*,
//! the beats are what the seat must then show.

use super::super::super::render;
use super::super::fixture::World;
use super::super::input;
use crate::cli_outbound::Cli;
use crate::paint_probe::Painted;

/// A settled full-window frame: its galleys with positions, on a context whose
/// panel rects can then be read back by id.
pub(super) struct Window {
    ctx: egui::Context,
    litany: Cli,
    bl: Cli,
    bz: Cli,
}

impl Window {
    pub(super) fn new() -> Self {
        Self {
            ctx: egui::Context::default(),
            litany: Cli::new("/yog-absent-litany"),
            bl: Cli::new("/yog-absent-bl"),
            bz: Cli::new("/yog-absent-bz"),
        }
    }

    /// Four frames — panels adopt their content height a frame late, and the
    /// composer's queue region settles one after that — then the galleys of the
    /// frame an operator would actually be looking at.
    pub(super) fn settled(&self, world: &mut World) -> Vec<Painted> {
        self.settled_on(world, input())
    }

    /// The same settle on a window of the caller's own size.
    pub(super) fn settled_on(&self, world: &mut World, raw: egui::RawInput) -> Vec<Painted> {
        let frame = |world: &mut World| {
            self.ctx.run(raw.clone(), |ctx| {
                render(
                    ctx,
                    &mut world.model,
                    &mut world.state,
                    &self.litany,
                    &self.bl,
                    &self.bz,
                );
            })
        };
        // Two frames, then the **wire settled to a fixed point** (REMOTE §9.7's
        // harness ruling): this seat's rows are a selection out of an answered
        // forest and a standing `Query::Agent` since bl-48ae, so a driver that
        // only ran frames would measure a panel holding nothing.
        let _ = frame(world);
        let _ = frame(world);
        world.drain(&mut |world| {
            let _ = frame(world);
        });
        let mut out = None;
        for _ in 0..4 {
            out = Some(frame(world));
        }
        crate::paint_probe::painted_of(&out.expect("four frames ran"))
    }

    /// The stored rect of a panel, by its id.
    pub(super) fn panel(&self, id: &str) -> egui::Rect {
        egui::containers::panel::PanelState::load(&self.ctx, egui::Id::new(id))
            .expect("the panel stores its rect")
            .rect
    }
}

/// Every painted galley whose text contains `needle`, with its rect.
pub(super) fn all(painted: &[Painted], needle: &str) -> Vec<egui::Rect> {
    painted
        .iter()
        .filter(|(text, _)| text.contains(needle))
        .map(|(_, rect)| *rect)
        .collect()
}

/// The one galley containing `needle`, or a panic naming it.
pub(super) fn one(painted: &[Painted], needle: &str) -> egui::Rect {
    *all(painted, needle)
        .first()
        .unwrap_or_else(|| panic!("{needle:?} not painted"))
}
