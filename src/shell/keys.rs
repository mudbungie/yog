//! The §11 keyboard surface: the `egui::Key` lift, the dispatch through the
//! pure [`keymap`](crate::keymap), and **the one home of every key-reachable
//! effect** — each widget's `.clicked()` branch calls the same function, so a
//! gesture has one implementation whichever hand fires it.
//!
//! Coverage-excluded glue like the rest of `src/shell/*`: the table it
//! dispatches through (`src/keymap`) and everything the effects call
//! (`AppModel`, `crate::start`, `crate::actions`) are tested.
//!
//! Three §11 rules decide what is here: a key fires a verb on the target the
//! selection already names (so no effect below grows its own cursor), choosing
//! *among* several targets stays a pointer gesture (so Move's destination, the
//! overflow entries, and which descent-tree member to open are not bound), and
//! a combo may only repaint or create — never fire a verb at the selection,
//! since a combo is the one plane that survives a text box holding the
//! keyboard.
//!
//! **"Which row to expand" left rule 2 with bl-fa82.** It was this doc's own
//! example of a pointer-only pick, written when nothing but a pointer could
//! name a list row. `←`/`→` unfold the row the *selection* already names, which
//! is rule 1 exactly; what rule 2 fences is a pick among many, not a verb on
//! the one target already selected.

use crate::AppModel;
use crate::cli_outbound::Cli;
use crate::keymap::{self, Held, Key, KeyAction, Mods};
use std::path::Path;

use super::ShellState;

/// egui `Key` → digit value for the tab-select keys, plus `0` — which names no
/// tab and carries the zoom reset on the combo plane (§11).
const DIGIT_KEYS: [(egui::Key, u8); 10] = [
    (egui::Key::Num0, 0),
    (egui::Key::Num1, 1),
    (egui::Key::Num2, 2),
    (egui::Key::Num3, 3),
    (egui::Key::Num4, 4),
    (egui::Key::Num5, 5),
    (egui::Key::Num6, 6),
    (egui::Key::Num7, 7),
    (egui::Key::Num8, 8),
    (egui::Key::Num9, 9),
];

/// egui `Key` → the §11 letters, bare and combo alike (`j` carries only
/// Ctrl+J). The table in `keymap` stays the authority on what each one means on
/// each modifier plane, including which are unbound.
const LETTER_KEYS: [(egui::Key, char); 13] = [
    (egui::Key::I, 'i'),
    (egui::Key::N, 'n'),
    (egui::Key::W, 'w'),
    (egui::Key::S, 's'),
    (egui::Key::X, 'x'),
    (egui::Key::F, 'f'),
    (egui::Key::C, 'c'),
    (egui::Key::R, 'r'),
    (egui::Key::B, 'b'),
    (egui::Key::M, 'm'),
    (egui::Key::G, 'g'),
    (egui::Key::A, 'a'),
    (egui::Key::J, 'j'),
];

/// egui `Key` → the zoom signs (§11). `=` lifts to [`Key::Plus`] alongside `+`
/// because zooming in must not demand Shift, exactly as browsers allow.
const SIGN_KEYS: [(egui::Key, Key); 3] = [
    (egui::Key::Plus, Key::Plus),
    (egui::Key::Equals, Key::Plus),
    (egui::Key::Minus, Key::Minus),
];

/// Lift the frame's key presses and run each one's effect (§11). What holds the
/// keyboard is passed through to the pure table rather than gating here: a bare
/// key is suppressed while a text field wants it (which is why Escape reaches
/// [`KeyAction::Cancel`] only after egui has spent one surrendering that focus),
/// a bare key under a modal reaches nothing but that modal's own Escape, and a
/// combo is never suppressed at all. `Modifiers::command` is ⌘ on macOS and Ctrl
/// elsewhere, so the lift spells each combo once for both §10 targets.
pub(super) fn handle(
    ctx: &egui::Context,
    model: &mut AppModel,
    state: &mut ShellState,
    lernie: &Cli,
    bl: &Cli,
) {
    // Text size is derived, never held: `ui.json` is the authority (§4.1) and
    // the context is its projection, re-asserted every frame. That is what
    // makes the size survive a relaunch — and what keeps a second instance's
    // adopted change from fighting this one's (egui's own zoom keys are off,
    // `theme::apply`). The call is a no-op when the factor already matches.
    ctx.set_zoom_factor(model.zoom());
    // What holds the keyboard (§11 suppression). A modal outranks text focus
    // whether or not its own field has it: the plane belongs to the surface
    // that owns the frame, not to the box inside it (bl-d921).
    let held = if super::modal::open(state) {
        Held::Modal
    } else if ctx.wants_keyboard_input() {
        Held::TextBox
    } else {
        Held::Nothing
    };
    let pressed: Vec<(Key, Mods)> = ctx.input(|i| {
        let mods = match (i.modifiers.command, i.modifiers.shift) {
            (true, false) => Mods::Command,
            (true, true) => Mods::CommandShift,
            (false, _) => Mods::Bare,
        };
        let mut keys = Vec::new();
        for (egui_key, key) in [
            (egui::Key::ArrowUp, Key::Up),
            (egui::Key::ArrowDown, Key::Down),
            (egui::Key::ArrowLeft, Key::Left),
            (egui::Key::ArrowRight, Key::Right),
            (egui::Key::Enter, Key::Enter),
            (egui::Key::Escape, Key::Escape),
        ] {
            if i.key_pressed(egui_key) {
                keys.push((key, mods));
            }
        }
        for (egui_key, n) in DIGIT_KEYS {
            if i.key_pressed(egui_key) {
                keys.push((Key::Digit(n), mods));
            }
        }
        for (egui_key, c) in LETTER_KEYS {
            if i.key_pressed(egui_key) {
                keys.push((Key::Char(c), mods));
            }
        }
        for (egui_key, key) in SIGN_KEYS {
            if i.key_pressed(egui_key) {
                keys.push((key, mods));
            }
        }
        keys
    });
    for action in pressed
        .into_iter()
        .filter_map(|(key, mods)| keymap::keymap(key, mods, held))
    {
        effect(action, model, state, lernie, bl);
    }
}

/// One intent's effect — the same call the matching widget makes.
fn effect(action: KeyAction, model: &mut AppModel, state: &mut ShellState, lernie: &Cli, bl: &Cli) {
    match action {
        KeyAction::ListPrev => super::focus::list_step(model, state, -1),
        KeyAction::ListNext => super::focus::list_step(model, state, 1),
        KeyAction::ExpandRow => super::focus::expand_row(model, state),
        KeyAction::CollapseRow => super::focus::collapse_row(model, state),
        KeyAction::Tab(tab) => model.select_tab(tab),
        KeyAction::Center(tab) => super::center::focus(model, state, tab),
        KeyAction::FocusComposer => super::focus::request(state),
        KeyAction::NewConversation => new_conversation(model, state),
        KeyAction::NewWorkspace => super::new_ws::open(state),
        KeyAction::StartHead => start_head(model, state, lernie, bl),
        KeyAction::Stop => super::dispatch::stop_selected(model, state, lernie, bl),
        KeyAction::Scan => super::dispatch::scan_focused(model, lernie, bl),
        KeyAction::Search => search_line(model, state),
        KeyAction::CloseBall => super::ball_bar::close_focused(model, lernie, bl),
        KeyAction::ReleaseBall => super::ball_bar::release_focused(model, lernie, bl),
        KeyAction::ToggleBalls => toggle_balls(model),
        KeyAction::ToggleModelPicker => state.wall.picker.toggle(),
        KeyAction::ToggleGrouping => state.group_by_ball = !state.group_by_ball,
        KeyAction::ToggleActivity => state.activity_open = !state.activity_open,
        KeyAction::Zoom(step) => model.zoom_nudge(step),
        KeyAction::Fire => fire_pending(model, state, lernie, bl),
        KeyAction::Cancel => cancel_pending(model, state),
        KeyAction::DismissModal => super::modal::dismiss(state),
    }
}

/// Enter — fire the pending start goal (§8.1), then hand the keyboard back to
/// the message composer if it actually launched (§11 focus discipline).
fn fire_pending(model: &mut AppModel, state: &mut ShellState, lernie: &Cli, bl: &Cli) {
    if super::start_pane::send_pending(model, &mut state.start, lernie, bl) {
        super::focus::request(state);
    }
}

/// Esc — put down whatever is up, nearest first: the pending start goal, else
/// a focused center tab, which returns to the conversation (QUALITY F3's
/// "Escape dismisses", kept now that the overlays are tabs — bl-1ca2; nothing
/// typed is lost either way, since a config draft is the editor's and a
/// composer draft is its target's).
///
/// The hand-back is gated on there having *been* something: with nothing up,
/// Escape is the composer's own release gesture (§11), and re-grabbing the box
/// on the same press would make that release unreachable — the operator could
/// never get back to the bare-key plane.
fn cancel_pending(model: &AppModel, state: &mut ShellState) {
    if state.start.pending.take().is_some() {
        super::focus::request(state);
    } else if !super::center::conversation_open(state) {
        super::center::focus(model, state, crate::keymap::CenterTab::Conversation);
    }
}

/// `new conversation` (§11): clear the agent selection — the composer then targets
/// a new root — and hand it the keyboard. A no-op with no workspace focused,
/// where there is no composer to target.
pub(super) fn new_conversation(model: &mut AppModel, state: &mut ShellState) {
    let Some(ws) = model.focused_workspace().map(Path::to_path_buf) else {
        return;
    };
    super::focus::workspace(model, state, &ws);
}

/// Ctrl+F — the find reflex (§8.5): put the composer on a `/search ` line and
/// hand it the keyboard. No new control and no search mode; the line seat is
/// already the typed way in, so the key only starts the sentence.
fn search_line(model: &AppModel, state: &mut ShellState) {
    let key = crate::actions::drafts::DraftKey::composer(
        model.focused_workspace().map(Path::to_path_buf),
        model.focused_agent_id(),
    );
    state.actions.drafts.set(key, "/search ".to_owned());
    super::focus::request(state);
}

/// ▶ Start the balls section's **first ready row** — the row it paints at its
/// top, which is the one the keyboard can name (§11 rule 2 leaves the pick
/// among several to the pointer). Nothing ready is a no-op.
fn start_head(model: &mut AppModel, state: &mut ShellState, lernie: &Cli, bl: &Cli) {
    let Some(inputs) = model.startable().into_iter().next() else {
        return;
    };
    super::start_pane::run_prepare(model, state, lernie, bl, inputs);
}

/// Fold / unfold the balls section — the persisted §4.1 collapse override, the
/// same write the section header's click makes.
fn toggle_balls(model: &mut AppModel) {
    let collapsed = model.is_collapsed("balls");
    model.set_collapsed("balls", !collapsed);
}
