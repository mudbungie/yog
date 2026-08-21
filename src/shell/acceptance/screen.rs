//! The keyboard driver the §11 focus tests steer by: one persistent
//! `egui::Context` across frames, real key events in, and egui's own
//! `wants_keyboard_input()` out. Split from [`super::focus`] for §12's budget —
//! the driver is how a frame is run, the tests are what a frame must show.

use super::super::render;
use super::fixture::World;
use crate::cli_outbound::Cli;

/// The window under test: one persistent `egui::Context`, so focus carries
/// frame to frame exactly as it does for the operator.
pub(super) struct Screen {
    ctx: egui::Context,
    lernie: Cli,
    bl: Cli,
    bz: Cli,
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

    /// One frame, and nothing else.
    fn paint(&self, world: &mut World, events: Vec<egui::Event>) -> egui::FullOutput {
        let input = egui::RawInput {
            // The held modifiers ride `RawInput` beside the events, not only on
            // them — `keys::handle` reads `i.modifiers`, so a combo spelled only
            // on the event would arrive as its bare twin and the plane under
            // test would be the wrong one.
            modifiers: modifiers_of(&events),
            events,
            ..super::input()
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

    /// Force every tooltip to paint, hover or no (`super::hover::live`'s drive
    /// and the paint-layer half both need it): the operator's hover is a
    /// pointer position no walk of the keyboard floor can be in two places for.
    pub(super) fn reveal(&self) {
        self.ctx.memory_mut(|m| m.set_everything_is_visible(true));
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

    /// A frame with no input.
    pub(super) fn idle(&self, world: &mut World) -> bool {
        self.frame(world, Vec::new())
    }

    /// Escape — egui spends it surrendering text focus (§11), which is how a
    /// test puts the keyboard back down before asking whether an operation
    /// picks it up again.
    pub(super) fn release(&self, world: &mut World) {
        assert!(
            !self.frame(world, vec![press(egui::Key::Escape, egui::Modifiers::NONE)]),
            "Escape is the release gesture: the box must let go"
        );
    }
}

/// The modifier plane a frame's presses arrive on.
fn modifiers_of(events: &[egui::Event]) -> egui::Modifiers {
    events
        .iter()
        .find_map(|e| match e {
            egui::Event::Key { modifiers, .. } => Some(*modifiers),
            _ => None,
        })
        .unwrap_or(egui::Modifiers::NONE)
}

/// The §11 Ctrl+Shift plane (⌘⇧ on macOS) — the `new workspace` combo.
pub(super) fn command_shift() -> egui::Modifiers {
    egui::Modifiers {
        shift: true,
        ..egui::Modifiers::COMMAND
    }
}

/// One full click at `pos`: move, press, release — three frames, because egui
/// hit-tests against the *previous* frame's widget rects, so a press in the
/// frame that first sees the pointer would test against nothing.
pub(super) fn click(screen: &Screen, world: &mut World, pos: egui::Pos2) {
    screen.frame(world, vec![egui::Event::PointerMoved(pos)]);
    for pressed in [true, false] {
        screen.frame(
            world,
            vec![egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: egui::Modifiers::NONE,
            }],
        );
    }
}

/// The rect of the galley reading exactly `text` — how a pointer test names a
/// seat without an `egui::Id` it has no way to know, and how a geometry test
/// asks which panel a row landed in. The **first**, in paint order; a word one
/// window paints twice needs [`rects_of`].
pub(super) fn rect_of(shapes: &[egui::epaint::ClippedShape], text: &str) -> Option<egui::Rect> {
    rects_of(shapes, text).into_iter().next()
}

/// **Every** rect a galley reading exactly `text` landed on. A label is not a
/// seat: `Login` is on this window three times — the navigator entry, the §11
/// tab strip's entry and the §8.3 row's verb — so a beat about the verb has to
/// tell them apart by where they landed, which it cannot do from the first
/// match alone.
pub(super) fn rects_of(shapes: &[egui::epaint::ClippedShape], text: &str) -> Vec<egui::Rect> {
    shapes
        .iter()
        .flat_map(|clipped| find(&clipped.shape, text))
        .collect()
}

/// The centre of that rect — the coordinate a click is aimed at.
pub(super) fn locate(shapes: &[egui::epaint::ClippedShape], text: &str) -> Option<egui::Pos2> {
    rect_of(shapes, text).map(|r| r.center())
}

/// Over [`paint_probe::collect`] — the ONE walk — and not a private copy of it.
///
/// This *was* a copy, and it was the copy bl-bc06 fixed and bl-36c3 swept for:
/// it matched on `Galley::text()`, which is the string that went IN. A row egui
/// truncated to `Login (bz browser…` still reports the whole label, so this
/// found it, handed back its rect, and the pointer test clicked confidently at
/// a seat whose painted text was not what it named — the one defect the paint
/// layer is the only witness for, aiming a click instead of reading a dump.
/// Both earlier balls fixed the homes they knew about; this copy was private to
/// the acceptance harness and survived both, which is why the check that
/// forbids the shape now lives in `rules/no-hand-rolled-paint-walk.yml` rather
/// than in anyone's memory (bl-70b8).
fn find(shape: &egui::Shape, text: &str) -> Vec<egui::Rect> {
    let mut painted = Vec::new();
    crate::paint_probe::collect(shape, &mut painted);
    painted
        .into_iter()
        .filter(|(seen, _)| seen == text)
        .map(|(_, rect)| rect)
        .collect()
}

pub(super) fn press(key: egui::Key, modifiers: egui::Modifiers) -> egui::Event {
    egui::Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers,
    }
}
