//! The §11 inbox-composer drive (bl-929d): pending deposits render above the
//! draft with fold arrows, the fold line's position is the content height
//! (floor = the bare input row, cap = half the pane), the snap-down fires on
//! delivery — structurally, off the pending count dropping — and the in-flight
//! strip keeps its seat above the line. Driven on the real window with an
//! explicit clock, because the snap is time-eased render ephemera.

use super::super::render;
use super::fixture::{World, world};
use super::input;
use crate::cli_outbound::Cli;

/// **Where the fold line SITS**, one budget below this file (§12): its height is
/// the queue's content height, floored at the bare input row and capped at half
/// the pane, and the snap-down that returns it there on a delivery. Split from
/// its siblings on a real seam — these beats measure one panel's geometry over
/// a driven clock, the beats here read what the queue *contains*.
mod fold_line;

/// A frame driver with an explicit clock — the snap eases on `i.time`, so the
/// tests hand it the frame's moment instead of the wall.
pub(super) struct Frames {
    ctx: egui::Context,
    lernie: Cli,
    bl: Cli,
    bz: Cli,
}

impl Frames {
    pub(super) fn new() -> Self {
        Self {
            ctx: egui::Context::default(),
            lernie: Cli::new("yog-absent-lernie"),
            bl: Cli::new("yog-absent-bl"),
            bz: Cli::new("yog-absent-bz"),
        }
    }

    pub(super) fn run(&self, world: &mut World, t: f64) -> egui::FullOutput {
        self.ctx.run(
            egui::RawInput {
                time: Some(t),
                ..input()
            },
            |ctx| {
                render(
                    ctx,
                    &mut world.model,
                    &mut world.state,
                    &self.lernie,
                    &self.bl,
                    &self.bz,
                );
            },
        )
    }

    /// Settle the panels and the queue's one-frame measurements around `t` —
    /// and the **wire to a fixed point** first (REMOTE §9.7's harness ruling):
    /// since bl-48ae the composer's target line and the §11 strip are
    /// selections out of an answered forest, so a driver that only ran frames
    /// would compose against a window that had not been told what was selected.
    pub(super) fn settle(&self, world: &mut World, t: f64) {
        self.run(world, t);
        self.run(world, t);
        world.drain(&mut |world| {
            self.run(world, t);
        });
        // The queue's own measurement, which the drain cannot settle: the fold
        // line is last frame's painted content height eased over `i.time`
        // (bl-929d), and since the pending listing became `Query::Inbox`'
        // answer (bl-b4b5) that content lands on a frame the drain's fixed
        // point has already passed. Each of these settles the wire *and*
        // advances the clock, because the drain returns on the pass that
        // answers rather than on a frame that has read the answer.
        for i in 0..4 {
            world.settle();
            self.run(world, t + f64::from(i) * 0.01);
        }
    }

    /// The stored rect of a bottom panel, by its id.
    pub(super) fn panel(&self, id: &str) -> egui::Rect {
        egui::containers::panel::PanelState::load(&self.ctx, egui::Id::new(id))
            .expect("the panel stores its rect")
            .rect
    }
}

/// Land a deposit in c-1's inbox — what `lernie message` (or any other
/// instance's send) leaves there.
pub(super) fn deposit(world: &World, name: &str, at: &str, body: &str) {
    let dir = world.ws.join("inbox/c-1");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join(name),
        format!("---\nfrom: user\ndeposited_at: {at}\n---\n{body}"),
    )
    .unwrap();
}

/// Turn the coalescing window off (a legal cadence — `DEBOUNCE_BOUNDS`
/// floors at zero) so a marked workspace derives on the very next pass:
/// these tests drive the clock explicitly, and a real 100 ms debounce would
/// put wall-clock sleeps in the suite.
pub(super) fn quick(mut world: World) -> World {
    std::fs::write(
        world.model.state_root().join("cadence.yaml"),
        "cadence:\n  watcher:\n    debounce_ms: 0\n",
    )
    .unwrap();
    world.model.after_lernie_verb();
    world.converge();
    world
}

/// Converge the world on a disk change: mark the workspace root dirty — the
/// same root the live watcher would deliver (§7.1) — and run one derivation
/// pass plus the frame's take of it.
pub(super) fn converge_ws(world: &mut World) {
    let ws = world.ws.clone();
    world.model.mark_dirty([ws]);
    world.converge();
}

/// Drain the inbox — what lernie's delivery commit does to `inbox/<id>/`.
pub(super) fn drain(world: &World) {
    let dir = world.ws.join("inbox/c-1");
    for entry in std::fs::read_dir(&dir).unwrap().flatten() {
        std::fs::remove_file(entry.path()).unwrap();
    }
}

/// The vertical center of the first galley containing `needle`.
fn center_y(painted: &[crate::paint_probe::Painted], needle: &str) -> f32 {
    painted
        .iter()
        .find(|(text, _)| text.contains(needle))
        .unwrap_or_else(|| panic!("{needle:?} not painted"))
        .1
        .center()
        .y
}

/// Pending items render oldest-first above the draft, one line each with the
/// jsonview fold arrow; the input is the queue's last item and has no arrow;
/// a fold override — RAM keyed by the deposit's inbox path (§5.3) — opens
/// exactly its row.
#[test]
fn pending_items_stack_above_the_draft_with_fold_arrows_and_ram_folds() {
    let mut world = quick(world());
    let ws = world.ws.clone();
    world.model.focus_agent(&ws, "c-1");
    deposit(&world, "user-002.md", "t1", "second note\nmore detail");
    converge_ws(&mut world);
    let frames = Frames::new();
    frames.settle(&mut world, 0.0);
    let painted = crate::paint_probe::painted_of(&frames.run(&mut world, 0.1));

    // Oldest first, the draft last: t0 above t1 above the box's hint.
    let first = center_y(&painted, "✉ user · t0");
    let second = center_y(&painted, "✉ user · t1");
    let box_hint = center_y(&painted, "message hello");
    assert!(first < second, "oldest first: {first} !< {second}");
    assert!(
        second < box_hint,
        "the draft is last: {second} !< {box_hint}"
    );

    // Each pending row carries the fold arrow on its own line; the input row
    // carries none (§11: "anything but the user input gets a fold arrow").
    let arrow_on = |y: f32| {
        painted
            .iter()
            .any(|(text, rect)| text == "▶" && (rect.center().y - y).abs() < 3.0)
    };
    assert!(arrow_on(first), "the first pending row folds");
    assert!(arrow_on(second), "the second pending row folds");
    assert!(!arrow_on(box_hint), "the input has no arrow");

    // Folded: the first line rides inline, the rest stays shut.
    let text = crate::paint_probe::text_of(&frames.run(&mut world, 0.2));
    assert!(text.contains("second note"), "folded preview:\n{text}");
    assert!(!text.contains("more detail"), "folded body stays shut");

    // The override is RAM keyed by the deposit's inbox path (§5.3).
    world
        .state
        .composer
        .folds
        .insert("inbox/c-1/user-002.md".to_owned());
    let open = crate::paint_probe::text_of(&frames.run(&mut world, 0.3));
    assert!(open.contains("more detail"), "the fold opens:\n{open}");
}

/// The in-flight strip's seat is preserved (bl-905f, re-ruled by bl-929d):
/// above the fold line, with the transcript's present, never inside the
/// pending region. Driven with a real held executor lock and response writer,
/// the two observations the §3.5 classifier reads.
#[test]
fn the_in_flight_strip_sits_above_the_fold_line() {
    let mut world = quick(world());
    let ws = world.ws.clone();
    world.model.focus_agent(&ws, "c-1");
    // The driver: the inbox-dir lock fd and the response.json writer fd, held
    // by this very process — what the /proc probes actually scan for.
    let _lock = std::fs::File::open(ws.join("inbox/c-1")).unwrap();
    let _writer = std::fs::OpenOptions::new()
        .append(true)
        .open(ws.join("steps/c-1/001/response.json"))
        .unwrap();
    converge_ws(&mut world);
    let frames = Frames::new();
    frames.settle(&mut world, 0.0);
    let text = crate::paint_probe::text_of(&frames.run(&mut world, 0.1));
    assert!(
        text.contains("a model call is streaming"),
        "the strip is up:\n{text}"
    );
    let strip = frames.panel("flight-strip");
    let composer = frames.panel("composer");
    assert!(
        strip.bottom() <= composer.top() + 1.0,
        "the strip sits above the fold line: strip {} !≤ composer {}",
        strip.bottom(),
        composer.top()
    );
}
