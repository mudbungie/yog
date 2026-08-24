//! The keyboard driver the §11 focus tests steer by: one persistent
//! `egui::Context` across frames, real key events in, and egui's own
//! `wants_keyboard_input()` out. Split from [`super::focus`] for §12's budget —
//! the driver is how a frame is run, the tests are what a frame must show.

use super::super::render;
use super::fixture::World;
use crate::cli_outbound::Cli;

/// **How a beat names a seat on the glass** — the paint-layer locator, split
/// from the driver at §12's budget on the seam the two already had: this file
/// is how a frame is *run*, `aim` is how a coordinate is found in one.
mod aim;
/// **How a beat spells an input** — the key, the click, the release and the
/// modifier plane they arrive on, split from the driver at the same budget and
/// on the same seam, one door over.
mod gesture;
pub(super) use aim::{locate, rect_of, rects_of};
use gesture::modifiers_of;
pub(super) use gesture::{click, command_shift, press};

/// The window under test: one persistent `egui::Context`, so focus carries
/// frame to frame exactly as it does for the operator.
pub(super) struct Screen {
    ctx: egui::Context,
    lernie: Cli,
    bl: Cli,
    bz: Cli,
    /// The window this driver paints into, when a beat needs a **particular**
    /// one (bl-86a5). `None` is [`super::input`]'s 1600x2400, which is taller
    /// than any list the suite can build — so a §11 rule 5 budget defect in a
    /// column that has to divide itself is invisible at that size, and a
    /// pointer beat about one has to say which window it means.
    size: Option<(f32, f32)>,
}

impl Screen {
    /// Binaries that deliberately do not exist: the send test below is the one
    /// acceptance case that actually *dispatches*, and what it asserts is where
    /// the keyboard ends up, not what `lernie` did. A name nothing resolves
    /// fails the spawn at once, so the frame never forks the operator's real
    /// substrate to prove a focus rule.
    pub(super) fn new() -> Self {
        Self {
            ctx: egui::Context::default(),
            lernie: Cli::new("/yog-absent-lernie"),
            bl: Cli::new("/yog-absent-bl"),
            bz: Cli::new("/yog-absent-bz"),
            size: None,
        }
    }

    /// The same driver on a window of a **named** size ([`Screen::size`]).
    pub(super) fn sized(w: f32, h: f32) -> Self {
        Self {
            size: Some((w, h)),
            ..Self::new()
        }
    }

    /// The same driver with a `lernie` that really answers — for the one
    /// acceptance case whose gesture has to *succeed* end to end (the §3.4
    /// raise, whose surface afterwards is what bl-9acf is about). `bl` and `bz`
    /// stay absent: a bare rung mutates no ball and paints no login.
    pub(super) fn with_lernie(lernie: Cli) -> Self {
        Self {
            lernie,
            ..Self::new()
        }
    }

    /// Run one frame on `events` and report whether a text box holds the
    /// keyboard afterwards.
    pub(super) fn frame(&self, world: &mut World, events: Vec<egui::Event>) -> bool {
        let _ = self.run(world, events);
        self.ctx.wants_keyboard_input()
    }

    /// Run one idle frame and return every painted galley's text — the same
    /// read [`super::painted`] makes, but on **this** screen's persistent
    /// context, so what a box shows can be asserted after the keyboard has
    /// typed into it.
    pub(super) fn text(&self, world: &mut World) -> String {
        let out = self.run(world, Vec::new());
        crate::paint_probe::text_of(&out)
    }

    /// Run one frame and hand back everything it painted. A **pointer** test has
    /// to find its coordinate before it can click one: a widget's rect is not
    /// addressable from outside the frame that built it, but the galley it
    /// painted is, and a key names the same seat far more stably than a number.
    pub(super) fn shapes(
        &self,
        world: &mut World,
        events: Vec<egui::Event>,
    ) -> Vec<egui::epaint::ClippedShape> {
        self.output(world, events).shapes
    }

    /// One frame, whole — for a test that must read two things off the **same**
    /// frame (what a run says and what hue it was painted in), which two calls
    /// could not honestly give it.
    pub(super) fn output(&self, world: &mut World, events: Vec<egui::Event>) -> egui::FullOutput {
        self.run(world, events)
    }

    /// One frame of the real window, **with the wire it spoke on settled** —
    /// the acts it fired *and* the reads it declared.
    ///
    /// Since bl-1747 a gesture is posted rather than run (REMOTE §9.8), so its
    /// frame-side aftermath — a draft clearing, a workspace adopted, a seed
    /// spent, a dialog closing — lands on a *later* frame reading the receipt.
    /// And the §8.1 start family is two acts: the second is posted only when
    /// the first one's receipt arrives. A window pays that in ask periods; a
    /// drive would have to pay it in counted frames, at every call site, which
    /// is a fact about the transport leaking into every test that has nothing
    /// to do with it. So it is paid here, once.
    ///
    /// **bl-44e9 extended it to the read half, which is the same ruling one
    /// door over.** A migrated surface paints an answer that landed a round
    /// trip later, so a drive that only settled acts saw every such surface
    /// blank — and a beat asserting a row is *there* would have been failing
    /// for the transport's reason rather than the window's. The loop settles
    /// both to a fixed point: `World::settle` answers what is outstanding and
    /// says whether anything **moved**, which for reads is the standing set
    /// changing (every call answers the whole set, so "I answered something"
    /// would never go false). It terminates for the acts' own reason — only a
    /// receipt can post an act, and nothing posts one unprompted — and for the
    /// reads' equivalent: a surface's questions are a function of state, and a
    /// frame that changed no state declares the set it declared before.
    fn run(&self, world: &mut World, events: Vec<egui::Event>) -> egui::FullOutput {
        // The engine's spawns are the ones a posted act runs (REMOTE §9.8), so
        // this drive's fakes have to be the world's — a seat carries the
        // gesture and never a binary.
        world.substrate(&self.lernie, &self.bl);
        let mut out = self.paint(world, events);
        // The fixed point itself is `World::drain` (bl-13f9), one definition
        // for both drivers; what is this driver's own is the context the
        // repaints run on, so the loop takes the painting as a parameter.
        world.drain(&mut |world| out = self.paint(world, Vec::new()));
        out
    }

    /// One frame with the wire **left outstanding** — the gap between an act's
    /// post and its receipt, which every other read here settles away.
    ///
    /// That gap is where a whole class of defect lives and the only place it
    /// can be driven from (bl-56c6): the composer is not disabled across it, so
    /// what the operator types there is a draft like any other, and the fold
    /// that runs when the receipt lands has to leave it alone. A drive that
    /// settles to a fixed point per frame is never inside the gap and so can
    /// assert nothing about it.
    pub(super) fn unsettled(&self, world: &mut World, events: Vec<egui::Event>) -> bool {
        world.substrate(&self.lernie, &self.bl);
        let _ = self.paint(world, events);
        self.ctx.wants_keyboard_input()
    }

    /// One frame, and nothing else.
    fn paint(&self, world: &mut World, events: Vec<egui::Event>) -> egui::FullOutput {
        let input = egui::RawInput {
            // The held modifiers ride `RawInput` beside the events, not only on
            // them — `keys::handle` reads `i.modifiers`, so a combo spelled only
            // on the event would arrive as its bare twin and the plane under
            // test would be the wrong one.
            modifiers: modifiers_of(&events),
            events,
            ..self.window()
        };
        self.ctx.run(input, |ctx| {
            render(
                ctx,
                &mut world.model,
                &mut world.state,
                &self.lernie,
                &self.bl,
                &self.bz,
            );
        })
    }

    /// The raw input this driver's frames are laid into — its own size, or the
    /// suite's default window.
    fn window(&self) -> egui::RawInput {
        match self.size {
            Some((w, h)) => crate::paint_probe::screen_sized(w, h),
            None => super::input(),
        }
    }

    /// The right edge of the §11 conversation panel — the boundary between the
    /// list column and the centre, read off the panel's own stored rect (the
    /// same read `super::geometry` makes of it).
    ///
    /// A drive that asserts a row is **absent** needs it. The altitude-1 descent
    /// tree paints the same member names in the centre whenever a conversation
    /// is selected, and every gesture under test here selects one, so "nothing
    /// paints this name" is a claim about the column and never about the window
    /// — asserted window-wide it would fail on a correct tree, and its inverse
    /// would pass on one that never unfolded anything.
    pub(super) fn column(&self) -> f32 {
        egui::containers::panel::PanelState::load(&self.ctx, egui::Id::new("conversations"))
            .expect("the conversation panel stores its rect")
            .rect
            .right()
    }

    /// Which widget holds the frame's own focus — the §11 floor's cursor, the
    /// one Tab moves and Space presses (bl-478d).
    pub(super) fn focused(&self) -> Option<egui::Id> {
        self.ctx.memory(egui::Memory::focused)
    }

    /// What the widget `id` **was** — egui's own record of the response it
    /// handed the render site, sense and enablement included. The frame is over
    /// by the time a test asks, which is exactly what `read_response` answers.
    pub(super) fn response(&self, id: egui::Id) -> Option<egui::Response> {
        self.ctx.read_response(id)
    }

    /// Whether widget `id` had a tooltip open on the frame just run — egui's
    /// own association of a tooltip with the widget that owns it (the tooltip
    /// area's id is derived from the widget's), so a test needs no list of
    /// which controls are supposed to have one.
    pub(super) fn tooltipped(&self, id: egui::Id) -> bool {
        egui::popup::was_tooltip_open_last_frame(&self.ctx, id)
    }
}
