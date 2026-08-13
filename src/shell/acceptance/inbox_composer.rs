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

/// A frame driver with an explicit clock — the snap eases on `i.time`, so the
/// tests hand it the frame's moment instead of the wall.
struct Frames {
    ctx: egui::Context,
    lernie: Cli,
    bl: Cli,
    bz: Cli,
}

impl Frames {
    fn new() -> Self {
        Self {
            ctx: egui::Context::default(),
            lernie: Cli::new("yog-absent-lernie"),
            bl: Cli::new("yog-absent-bl"),
            bz: Cli::new("yog-absent-bz"),
        }
    }

    fn run(&self, world: &mut World, t: f64) -> egui::FullOutput {
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

    /// Settle the panels and the queue's one-frame measurements around `t`.
    fn settle(&self, world: &mut World, t: f64) {
        for i in 0..4 {
            self.run(world, t + f64::from(i) * 0.01);
        }
    }

    /// The stored rect of a bottom panel, by its id.
    fn panel(&self, id: &str) -> egui::Rect {
        egui::containers::panel::PanelState::load(&self.ctx, egui::Id::new(id))
            .expect("the panel stores its rect")
            .rect
    }
}

/// Land a deposit in c-1's inbox — what `lernie message` (or any other
/// instance's send) leaves there.
fn deposit(world: &World, name: &str, at: &str, body: &str) {
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
fn drain(world: &World) {
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

/// The fold line's position IS the content height: the empty inbox is the
/// bare input row (the general path, zero items), each landing item pushes the
/// line up, and past half the pane the line stops — more items scroll instead
/// of climbing.
#[test]
fn the_fold_line_is_the_content_height_with_floor_and_cap() {
    let mut world = quick(world());
    let ws = world.ws.clone();
    world.model.focus_agent(&ws, "c-1");
    let frames = Frames::new();

    // Zero items: the bare input row.
    drain(&world);
    converge_ws(&mut world);
    frames.settle(&mut world, 0.0);
    let bare = frames.panel("composer").height();

    // One item raises the line; a second raises it again.
    deposit(&world, "user-001.md", "t0", "one");
    converge_ws(&mut world);
    frames.settle(&mut world, 1.0);
    let one = frames.panel("composer").height();
    deposit(&world, "user-002.md", "t1", "two");
    converge_ws(&mut world);
    frames.settle(&mut world, 2.0);
    let two = frames.panel("composer").height();
    assert!(one > bare + 4.0, "an item raises the line: {bare} → {one}");
    assert!(two > one + 4.0, "and the next again: {one} → {two}");

    // The cap: a flood of items stops the line at half the pane — the
    // sixty-deposit queue and the eighty-deposit queue sit at the same
    // boundary, and the extra rows scroll behind it.
    for i in 3..=60 {
        deposit(&world, &format!("user-{i:03}.md"), "t", "pile");
    }
    converge_ws(&mut world);
    frames.settle(&mut world, 3.0);
    let sixty = frames.panel("composer").height();
    for i in 61..=80 {
        deposit(&world, &format!("user-{i:03}.md"), "t", "pile");
    }
    converge_ws(&mut world);
    frames.settle(&mut world, 4.0);
    let eighty = frames.panel("composer").height();
    assert!(
        sixty > two,
        "the line kept climbing to the cap: {two} → {sixty}"
    );
    assert!(
        (eighty - sixty).abs() < 2.0,
        "past the cap the line holds still: {sixty} → {eighty}"
    );
    let window = input().screen_rect.expect("the probe sizes the screen");
    assert!(
        sixty < window.height() / 2.0 + 60.0,
        "the cap is half the pane: {sixty}"
    );
}

/// The snap-down is triggered structurally — the pending count dropping on
/// delivery — and eases the line from its pre-drain height to the bare row;
/// no gesture is consulted, so a drain by driver, scan or another instance
/// snaps identically.
#[test]
fn a_delivery_drain_snaps_the_line_down_to_its_floor() {
    let mut world = quick(world());
    let ws = world.ws.clone();
    world.model.focus_agent(&ws, "c-1");
    deposit(&world, "user-002.md", "t1", "two");
    deposit(&world, "user-003.md", "t2", "three");
    converge_ws(&mut world);
    let frames = Frames::new();
    frames.settle(&mut world, 0.0);
    let full = frames.panel("composer").height();

    // The delivery commit lands: the inbox empties with no yog gesture.
    drain(&world);
    converge_ws(&mut world);
    // The drop is observed at t=5.0; mid-ease the line sits between the two
    // heights, and past the ease it settles on the bare row.
    frames.run(&mut world, 5.0);
    frames.run(&mut world, 5.0 + crate::composer::SNAP_SECS * 0.3);
    let mid = frames.panel("composer").height();
    frames.settle(&mut world, 6.0);
    let bare = frames.panel("composer").height();
    assert!(bare < full - 8.0, "the queue emptied: {full} → {bare}");
    assert!(
        mid > bare + 2.0,
        "mid-ease the line is still descending: {mid} !> {bare}"
    );
    assert!(mid < full + 2.0, "and never above where it started: {mid}");
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
