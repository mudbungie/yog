//! **The start-window drive's own fixtures** (§7.2, §3.4) — the world, the
//! gestures and the paint reads every beat in [`super`] and its siblings shares,
//! split off at §12's per-file budget on the seam that already existed: what a
//! drive *does to* the window lives here, what the window must then *show* lives
//! in the beats.

pub(super) use super::super::fixture::{MINTED_FIRST, world};

use super::super::fixture::World;
use super::super::screen::{Screen, press};

/// The operator's words. Deliberately unlike anything the fixture, the theme or
/// the §3.3 wordlist paints, so `Screen::text` containing it can only be this
/// send (bl-f16e's rule: assert on what identifies *this* run).
///
/// **Short, because the row it lands in is a column** (bl-0219). The subtitle
/// shares one `left_to_right` line with the §3.3 title inside the conversation
/// panel, whose wrap mode is `Truncate` (§11 rule 1), so the title's width is
/// the subtitle's budget — and lernie's mint went from one lowercase word to a
/// PascalCase pair, which is most of that budget. `unbar the postern` painted
/// as `unbar the po…` and the beat read the glyphs, correctly, as not carrying
/// what was typed. Elision there is §11 working (`super::elision`); the beat
/// under test is the *immediacy* of the echo, so the phrase it echoes is one
/// that fits beside a two-word name. The verbatim claim is still made, on
/// `ConvRow::subtitle` below, where nothing elides.
pub(super) const SAID: &str = "unbar it";

/// Type `SAID` into the docked composer and press Enter — the whole gesture,
/// and the last thing that happens before the frame under test.
///
/// The trailing [`AppModel::refresh`](crate::AppModel::refresh) is the **frame's
/// own model duty** (§7.2), which `main.rs` runs once per update and the
/// acceptance driver leaves to its caller. It takes a published snapshot if
/// there is one and folds the pending echo; it starts no derivation and touches
/// no disk — which is exactly the claim each beat then pins by asserting the
/// derivation has not moved.
pub(super) fn say(screen: &Screen, world: &mut World) {
    typed(screen, world, SAID);
}

/// [`say`] with the words as a parameter — what a beat about a *second* send
/// needs, since two sends saying the same thing could not be told apart on the
/// glass by the words alone.
pub(super) fn typed(screen: &Screen, world: &mut World, text: &str) {
    screen.frame(world, vec![egui::Event::Text(text.to_owned())]);
    screen.frame(world, vec![press(egui::Key::Enter, egui::Modifiers::NONE)]);
    world.model.refresh();
}

/// The §11 conversation rows `name` would paint right now — the list as the
/// frame just built it, filtered to the one conversation under test.
pub(super) fn rows_named(world: &World, name: &str) -> Vec<crate::nav::convs::ConvRow> {
    crate::test_support::convs::conversations(&world.model, 0)
        .into_iter()
        .filter(|r| r.display_name() == name)
        .collect()
}

/// Turn the §7.2 coalescing window off (a legal cadence — `DEBOUNCE_BOUNDS`
/// floors at zero) so a marked workspace derives on the very next pass: these
/// beats drive the worker by hand, and a real 100 ms debounce would put a
/// wall-clock sleep in the suite.
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

/// What the driver's write looks like to yog: the workspace root marked dirty —
/// the same root the live watcher would announce (§7.1) — then one derivation
/// pass and the frame's take of it.
pub(super) fn converge_ws(world: &mut World) {
    let ws = world.ws.clone();
    world.model.mark_dirty([ws]);
    world.converge();
}

/// How many agent worktrees the driver has written. The real-substrate proof
/// that a beat's frame is showing the echo and not a derivation: a `lernie
/// prompt` that had landed would have made one here.
pub(super) fn branches(world: &World) -> usize {
    std::fs::read_dir(world.ws.join("agents")).map_or(0, |d| d.flatten().count())
}

/// One **settled** frame. egui panels reach their content height a frame after
/// the content they measure, and the queue region adds a settle of its own
/// (bl-929d) — what the operator sees is the steady state, so that is the frame
/// a beat reads.
pub(super) fn shot(screen: &Screen, world: &mut World) -> egui::FullOutput {
    for _ in 0..3 {
        screen.output(world, Vec::new());
    }
    screen.output(world, Vec::new())
}

/// Every fill colour this frame put on the glass — the role stripe among them
/// (`theme::role_stripe` is a `rect_filled`), which is how the §11 fading is
/// visible to a test at all: a faded row's stripe is its own hue at reduced
/// solidity, so the two states are two different colours on one frame.
pub(super) fn fills(out: &egui::FullOutput) -> Vec<egui::Color32> {
    let mut all = Vec::new();
    for clipped in &out.shapes {
        crate::paint_probe::collect_fills(&clipped.shape, &mut all);
    }
    all
}

/// The operator's own role stripe at the §11 pending solidity — what a §7.2
/// echo paints and nothing else does.
pub(super) fn faded_user() -> egui::Color32 {
    let (hue, _) = crate::theme::role_badge(crate::theme::Role::User);
    hue.gamma_multiply(crate::theme::tone_solidity(crate::transcript::Tone::Weak))
}
