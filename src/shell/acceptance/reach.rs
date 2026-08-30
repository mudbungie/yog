//! **What the operator must be able to press stays on the glass** (§11 rules 5
//! and 6 as extended by bl-86a5), driven through the real window at every size
//! the audit renders.
//!
//! Two surfaces, one defect, and it is a defect of *length*: a container whose
//! content has no bound lays the rest of itself past its own edge, where egui
//! paints it into a clip nothing shows and no pointer reaches. It cost the
//! window the navigator's ⚙ Config entry — the only visible door to the §3.6
//! workspace delete, so a wall with a long enough conversation list could not
//! be deleted through the window at all — and it cost both §3.6 dialogs their
//! own arming field and fire button, which is a destructive confirmation that
//! cannot be read before it is fired.
//!
//! **Length is the only axis these beats vary.** They run over
//! [`world_crowded`], which is the shipped fixture with more rows in it and no
//! other difference, so a beat that reddens here and passes elsewhere has found
//! a budget defect rather than a fixture. Nothing here asserts against a
//! galley's input string (`Galley::text()` is the text that went IN): every
//! read is [`crate::paint_probe::seen_of`], which narrows a run to what its
//! clip rect let through and drops what was clipped away entirely — which is
//! exactly the question.
//!
//! The keyboard floor is untouched and must stay so (`acceptance::floor`): egui
//! focuses a clipped widget as happily as a visible one, so the keyboard never
//! saw this defect and no keyboard beat could have caught it. The fix is
//! spatial and so is its guard.

use super::fixture::{crowd, world_crowded};
use super::screen::{Screen, click, locate, rects_of};
use crate::delete::agent::Census;
use crate::keymap::CenterTab;

/// What a beat types into an arming field: a word §3.6 refuses (it is not the
/// subject's name, so the fire button stays disarmed) and that nothing else in
/// the window paints, so the run found on the glass is the field's own and
/// never some other surface answering for it.
const ARMING: &str = "Armingrow";

/// How many descendants the agent dialog's census carries — taller than the
/// tallest window in [`SIZES`](super::SIZES) at any row height, because a
/// census that fits proves nothing about one that does not.
const DESCENDANTS: usize = 120;

/// Every galley whose glyphs **begin with** `head` that a finished frame really
/// put on the glass, in screen points.
///
/// A head rather than the whole label because elision is a different rule with
/// its own guard (`acceptance::legible`): a fire button egui cut to `Delete
/// work…` inside a narrow dialog is still ON the glass, which is the only
/// question here, and demanding the whole string would redden this file for the
/// other rule's defect.
///
/// The screen is the last clip. `seen_of` already drops a run its container
/// threw away, but an `egui::Window` is sized by its own content and laid over
/// everything, so its tail goes below the bottom edge with the screen's own
/// rect still around it — a run that intersects no pixel of the screen is a run
/// nobody can read or press, whichever clip let it through.
fn on_glass(out: &egui::FullOutput, size: (f32, f32), head: &str) -> Vec<egui::Rect> {
    let (w, h) = size;
    let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(w, h));
    crate::paint_probe::seen_of(out)
        .into_iter()
        .filter(|seen| seen.text.starts_with(head))
        .map(|seen| seen.shown.intersect(screen))
        .filter(|rect| rect.width() > 0.5 && rect.height() > 0.5)
        .collect()
}

/// The frame a beat about a **just-opened dialog** reads. A modal is an
/// `egui::Area` and an area's rect is not known until it has been laid out
/// once, so the frame it first appears on carries runs whose clip rect is not
/// yet a rectangle — `seen_of` drops every one of them, and a beat that read
/// that frame would report an empty screen whatever the layout did. One frame
/// to seat it, one laid out against the seated one, which is the same two-step
/// [`super::painted`] takes for a fresh context.
fn seated(screen: &Screen, world: &mut super::fixture::World) -> egui::FullOutput {
    screen.idle(world);
    screen.output(world, Vec::new())
}

/// **The navigator's doors survive a list taller than the column.** Before the
/// band stack the conversation list was first in a plain top-down flow, and a
/// `ScrollArea` sizes itself from what is available: the list took the whole
/// column and the balls section, the clients section and both entries laid out
/// past the panel's bottom edge, invisible and un-clickable.
///
/// Read in the column only. `Login` is on this window three times — the entry,
/// the §11 tab strip and the §8.3 row's verb — so a claim about the entry has
/// to say where it landed, which the panel's own stored right edge answers.
#[test]
fn the_navigator_keeps_its_doors_on_the_glass_under_a_tall_list() {
    let mut report = Vec::new();
    for size in super::SIZES {
        let (w, h) = size;
        let mut world = world_crowded();
        let screen = Screen::sized(w, h);
        let out = seated(&screen, &mut world);
        let column = screen.column();
        // **The fixture has to bite at this size**, or the rest of the beat is
        // a claim about a column that never had to divide itself. Counted here
        // rather than assumed from the crowd's own size: a taller row makes
        // the list longer, and this is the reading that would notice.
        let names = crowd::names();
        let rows = names
            .iter()
            .filter(|name| !on_glass(&out, size, name).is_empty())
            .count();
        if rows == 0 || rows == names.len() {
            report.push(format!(
                "at {w:.0}x{h:.0} the column showed {rows} of {} crowd rows — it \
                 must show some and not all, or nothing here is being tested",
                names.len()
            ));
        }
        for door in ["⚙ Config", "Login"] {
            let seated = on_glass(&out, size, door)
                .into_iter()
                .any(|rect| rect.right() <= column + 1.0);
            if !seated {
                report.push(format!(
                    "at {w:.0}x{h:.0} the navigator's `{door}` reached no pixel of the column"
                ));
            }
        }
    }
    // Every size at once: a budget defect is a shape across sizes, and stopping
    // at the first hides which of them a fix actually moved.
    assert!(report.is_empty(), "{}", report.join("\n"));
}

/// **And the entry is still the entry**: the pointer half, at the seat the
/// operator reported — one click on `⚙ Config` focuses the Config tab, which is
/// where the §3.6 danger row lives.
///
/// The first two assertions are what makes the third evidence rather than a
/// restatement: the column must be showing *some* of the crowd and not *all* of
/// it, or the fixture is not overflowing and the beat would pass on the very
/// layout it exists to forbid. 800x500 rather than the suite's default window,
/// which at 2400 pt tall is taller than any list this fixture can build.
#[test]
fn the_config_entry_still_clicks_when_the_list_outgrows_the_column() {
    let mut world = world_crowded();
    let screen = Screen::sized(800.0, 500.0);
    screen.idle(&mut world);
    screen.release(&mut world);
    let shapes = screen.shapes(&mut world, Vec::new());
    let names = crowd::names();
    let shown = names
        .iter()
        .filter(|name| !rects_of(&shapes, name).is_empty())
        .count();
    assert!(shown > 0, "the crowded roster paints rows at all");
    assert!(
        shown < names.len(),
        "the fixture must really outgrow the column, or this beat asserts \
         nothing: {shown} of {} rows are on the glass",
        names.len()
    );
    let seat = locate(&shapes, "⚙ Config")
        .expect("the Config entry reaches the paint layer under a tall list");
    click(&screen, &mut world, seat);
    assert_eq!(
        world.state.center,
        CenterTab::Config,
        "the coordinate really is the Config entry"
    );
}

/// **The workspace dialog's arming row outlives its own census.** §3.6 mandates
/// the concrete enumeration and nothing bounds how long one is; the window is
/// `resizable(false)` and sized by its content, so a wall with enough
/// conversations in it laid the typed-name field and the fire button below the
/// bottom of the screen.
#[test]
fn the_workspace_delete_dialog_keeps_its_arming_row_on_the_glass() {
    let mut report = Vec::new();
    for size in super::SIZES {
        let (w, h) = size;
        let mut world = world_crowded();
        let screen = Screen::sized(w, h);
        screen.idle(&mut world);
        super::super::delete::open(&world.model, &mut world.state, crowd::WALL);
        world.state.delete.typed = ARMING.to_owned();
        let out = seated(&screen, &mut world);
        assert!(
            world.state.delete.target.is_some(),
            "the dialog stands on a named wall at {w:.0}x{h:.0} — a foreign one \
             closes on the frame it opens and would prove nothing"
        );
        for seat in [ARMING, "Delete workspace"] {
            if on_glass(&out, size, seat).is_empty() {
                report.push(format!(
                    "at {w:.0}x{h:.0} the workspace dialog's `{seat}` reached no pixel"
                ));
            }
        }
    }
    assert!(report.is_empty(), "{}", report.join("\n"));
}

/// **The conversation dialog's is the same claim one door over.** Its census is
/// the substrate's own answer (`litany delete --children --dry-run`), so the
/// beat hands it one rather than building 120 agents to be told about them: the
/// dialog paints what the census says, and how long that list is is the only
/// thing under test.
#[test]
fn the_conversation_delete_dialog_keeps_its_fire_button_on_the_glass() {
    let mut report = Vec::new();
    for size in super::SIZES {
        let (w, h) = size;
        let mut world = world_crowded();
        let ws = world.ws.clone();
        let screen = Screen::sized(w, h);
        screen.idle(&mut world);
        world.state.delete_agent = super::super::delete_agent::DeleteAgentState {
            target: Some((ws, "c-1".to_owned())),
            census: Some(Census {
                descendants: (0..DESCENDANTS).map(|n| format!("c-1-{n:04}-x")).collect(),
                pending_deposits: 7,
            }),
            typed: ARMING.to_owned(),
            ..super::super::delete_agent::DeleteAgentState::default()
        };
        let out = seated(&screen, &mut world);
        assert!(
            world.state.delete_agent.target.is_some(),
            "the dialog stands at {w:.0}x{h:.0}"
        );
        for seat in [ARMING, "Delete conversation"] {
            if on_glass(&out, size, seat).is_empty() {
                report.push(format!(
                    "at {w:.0}x{h:.0} the conversation dialog's `{seat}` reached no pixel"
                ));
            }
        }
    }
    assert!(report.is_empty(), "{}", report.join("\n"));
}
