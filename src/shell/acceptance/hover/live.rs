//! The §11 discoverability invariant **read off behavior instead of names**
//! (bl-8e7a): drive the real window, walk its keyboard floor, and require every
//! response that senses a click to have said something.
//!
//! [`super`]'s source scan asks the question of a **list of constructor
//! spellings** (`CONTROLS`). That list is hand-maintained, so a widget built by
//! a call it does not know about is not judged mute — it is not judged at all,
//! and nothing anywhere goes red. That is bl-45c7's failure mode in its
//! quieter form: a check passing on the subset it happens to know.
//!
//! Here nothing is enumerated. The drive presses Tab — §11's own focus floor
//! (`keymap::spell::FLOOR`) — around each surface the window can show, and asks
//! egui two questions about every widget the cursor lands on, both of which
//! egui answers about *itself*: **does this response sense a click**
//! ([`egui::Response::sense`], the widget's own declaration, whatever
//! constructor made it) and **did a tooltip open for it**
//! (`egui::popup::was_tooltip_open_last_frame`, egui's own association of a
//! tooltip with its widget, since the tooltip area's id is derived from the
//! widget's). A control invented tomorrow, by a call nobody here has heard of,
//! is judged the day it is painted.
//!
//! **Both halves stand, and neither subsumes the other.** This one is
//! name-free but reaches only what the fixture paints; the source scan reaches
//! every seat in the tree — a provider Login row, a workflow-file button — but
//! only through the spellings it lists. So a new
//! constructor in a reachable seat now fails here whatever it is called, and
//! the list's blind spot is what remains of the hole rather than the whole of
//! it.

use super::super::fixture::{World, world};
use super::super::screen::{Screen, press};
use crate::keymap::{CenterTab, InspectorTab};
use std::collections::HashSet;

/// Tab presses per surface — a bound, not a count: the walk stops the moment
/// the floor's cursor lands where it has already been, so this only has to
/// exceed the widest surface the fixture paints.
const LAP: usize = 200;

/// One click-sensing widget the walk landed on: whether it said anything, and
/// what it reads as on screen — an `egui::Id` is a hash, so a failure naming
/// only the id would name nothing an author could go and fix.
struct Control {
    seat: egui::Rect,
    label: String,
    said_something: bool,
}

/// Is this response the **layer's own handle** rather than a control on it?
///
/// egui gives every `Area` a click-sensing response of its own — a combo box's
/// open popup, a menu — so that a click can bring it to the front (`egui::Area`:
/// *"allow clicks to bring to front"*). That is a surface, not a control: the
/// operator presses the rows inside it, and those are walked in their own
/// right. Told apart by egui's own construction rather than guessed from shape
/// or size — the handle's id is `layer.id.with("move")`, so the question is put
/// to the response instead of to a heuristic.
fn is_a_surface(response: &egui::Response) -> bool {
    response.id == response.layer_id.id.with("move")
}

/// Walk the focus floor once around the surface currently painted, collecting
/// every click-sensing widget it lands on.
fn controls(screen: &Screen, world: &mut World) -> Vec<(egui::Id, Control)> {
    let mut walked = Vec::new();
    let mut visited = HashSet::new();
    for _ in 0..LAP {
        screen.frame(world, vec![press(egui::Key::Tab, egui::Modifiers::NONE)]);
        // egui's floor passes through "nothing focused" on its way round the
        // lap; the press after that lands on the first widget again, which is
        // where the walk stops.
        let Some(id) = screen.focused() else {
            continue;
        };
        if !visited.insert(id) {
            break;
        }
        let response = screen
            .response(id)
            .expect("the widget the cursor is on is a widget of this frame");
        if response.sense.click && !is_a_surface(&response) {
            walked.push((id, response.rect, screen.tooltipped(id)));
        }
    }
    let mut painted = Vec::new();
    for clipped in screen.shapes(world, Vec::new()) {
        crate::paint_probe::collect(&clipped.shape, &mut painted);
    }
    walked
        .into_iter()
        .map(|(id, seat, said_something)| {
            let label = painted
                .iter()
                .filter(|(_, at)| seat.contains_rect(*at))
                .map(|(text, _)| text.as_str())
                .collect::<Vec<_>>()
                .join(" ");
            (
                id,
                Control {
                    seat,
                    label,
                    said_something,
                },
            )
        })
        .collect()
}

/// Walk the surface now painted and fold it into the run: every click-sensing
/// widget the drive has not already judged, and the mute ones among them.
fn sweep(
    screen: &Screen,
    world: &mut World,
    judged: &mut HashSet<egui::Id>,
    mute: &mut Vec<String>,
) {
    screen.idle(world);
    for (id, control) in controls(screen, world) {
        if judged.insert(id) && !control.said_something {
            mute.push(format!("{:?} at {:?}", control.label, control.seat));
        }
    }
}

/// **The invariant, derived from behavior.** Every control the window paints
/// says what pressing it does — asked of the widgets themselves, over every
/// surface the §11 tab strips can show.
#[test]
fn every_click_sensing_control_the_window_paints_says_what_it_does() {
    let mut world = world();
    let ws = world.ws.clone();
    world.model.focus_agent(&ws, "c-1");
    // The bottom accessory is a surface of its own, and closed it paints none
    // of its controls.
    world.state.activity_open = true;
    let screen = Screen::new();
    screen.idle(&mut world);
    // The operator's hover is one pointer position and this walk is in
    // twenty-odd places; with every tooltip painted unconditionally the
    // question becomes *which widget opened one* — a fact about the widget
    // rather than about where a test put the mouse.
    screen.reveal();

    let (mut judged, mut mute) = (HashSet::new(), Vec::new());
    // The tours are the enums themselves — `CenterTab::all` / `InspectorTab::all`,
    // the same lists the §11 strips and the digit map read — so a tab added to
    // the window is walked without anyone remembering to add it here.
    for tab in CenterTab::all() {
        world.state.center = tab;
        sweep(&screen, &mut world, &mut judged, &mut mute);
    }
    world.state.center = CenterTab::Conversation;
    for tab in InspectorTab::all() {
        world.model.select_tab(tab);
        sweep(&screen, &mut world, &mut judged, &mut mute);
    }

    assert!(
        judged.len() > 50,
        "the drive reached {} controls — a walk that stopped walking proves \
         nothing, the same two-direction discipline the source scan keeps",
        judged.len()
    );
    assert!(
        mute.is_empty(),
        "these controls sense a click and say nothing on hover — §11 requires \
         every one of them to state what pressing it does:\n{}",
        mute.join("\n")
    );
}
