//! **Focus discipline** (§11, `super::super::focus`), driven end to end through
//! the real window: after launch and after each basic operation, does the
//! keyboard actually sit in the message box?
//!
//! Asserted on egui's own answer — `Context::wants_keyboard_input()`, the same
//! predicate the §11 suppression rule reads — not on the request bit, so a
//! mechanism that set the bit and never spent it would fail here. Escape is the
//! release gesture, which makes it the tests' way of putting the keyboard back
//! down between operations. The driver is [`super::screen`].
//!
//! The **keyboard** half of the discipline — a selection landing the composer
//! however it was made, and the combo the walk then continues on — is
//! [`super::walk`], its own file at the rule's own seam.

use super::super::focus;
use super::fixture::{world, world_empty};
use super::screen::{Screen, command_shift, press};

/// The §3.4 invitation above the bootstrap's one box (`shell::bootstrap`) — a
/// run no other surface paints, so its presence identifies the frame.
const BOOTSTRAP_INVITATION: &str = "start a conversation:";

/// What the centre says instead when a workspace IS focused and no conversation
/// is selected (`shell::workspace::center`) — the frame this file's empty-world
/// test used to run against without noticing.
const SELECT_A_CONVERSATION: &str = "select a conversation";

/// Launch: the operator opens yog and types. No click, no key first — the
/// request stands from [`ShellState::new`] and the first painted composer
/// spends it.
#[test]
fn launch_lands_the_keyboard_in_the_composer() {
    let mut world = world();
    assert!(
        world.state.focus_composer,
        "a fresh ShellState carries the launch request"
    );
    let screen = Screen::new();
    assert!(
        screen.idle(&mut world),
        "the first frame hands the keyboard to the composer"
    );
    assert!(
        !world.state.focus_composer,
        "and spends the request rather than re-grabbing every frame"
    );
}

/// The empty world takes the same path — the bootstrap composer is not a
/// special case with a focus flag of its own (it used to carry an `egui::Id`
/// memory bit; one mechanism replaced it).
///
/// **The surface is asserted before the keyboard is** (bl-37bf). This test ran
/// for its whole life against `world_unfocused`, which withheld the startup
/// focus *argument* while leaving the workspace in the roster — so
/// `AppModel::startup_focus` derived a focus onto it, `shell::bootstrap` was
/// never called, and the box that took the keyboard was the start pane's. The
/// assertion passed on a widget that is not the one its own message names,
/// which is the vacuity shape bl-36c3 catalogued: a predicate satisfied by
/// something other than the behaviour under test.
///
/// So the frame proves it is the right frame first, by the two runs only the
/// bootstrap paints — the tagline under the wordmark, and the §3.4 invitation.
/// A world with a workspace in it paints neither, and paints
/// [`SELECT_A_CONVERSATION`] instead.
#[test]
fn the_empty_world_bootstrap_takes_the_launch_request_too() {
    let mut world = world_empty();
    let screen = Screen::new();
    let painted = screen.text(&mut world);
    for run in [crate::theme::TAGLINE, BOOTSTRAP_INVITATION] {
        assert!(
            painted.contains(run),
            "`{run}` is the bootstrap's own — this is not the bootstrap frame:\n{painted}"
        );
    }
    assert!(
        !painted.contains(SELECT_A_CONVERSATION),
        "and the conversation centre is not up beside it:\n{painted}"
    );
    assert!(
        screen.idle(&mut world),
        "the bootstrap box takes the keyboard"
    );
}

/// Opening a conversation with the pointer: the composer is re-aimed at what
/// was clicked, so the keyboard follows it there.
#[test]
fn opening_a_conversation_by_pointer_hands_over_the_keyboard() {
    let mut world = world();
    let ws = world.ws.clone();
    let screen = Screen::new();
    screen.idle(&mut world);
    screen.release(&mut world);
    focus::conversation(&mut world.model, &mut world.state, &ws, "c-1");
    assert!(
        screen.idle(&mut world),
        "the click that selected the conversation put the cursor in the box"
    );
}

/// Switching workspace, likewise — and `new conversation` is this same move
/// with the agent selection cleared.
#[test]
fn switching_workspace_by_pointer_hands_over_the_keyboard() {
    let mut world = world();
    let ws = world.ws.clone();
    let screen = Screen::new();
    screen.idle(&mut world);
    screen.release(&mut world);
    focus::workspace(
        &mut world.model,
        &mut world.state,
        &crate::naming::leaf(&ws),
    );
    assert!(screen.idle(&mut world), "and the tab click does too");
}

/// Escape with nothing pending is the release, not a round trip: re-grabbing on
/// the same press would make the bare-key plane unreachable.
#[test]
fn escape_with_nothing_pending_does_not_re_grab() {
    let mut world = world();
    let screen = Screen::new();
    screen.idle(&mut world);
    screen.release(&mut world);
    assert!(
        !screen.idle(&mut world),
        "the release holds — Escape then `i` is the §11 idiom, and `i` is a key"
    );
}

/// The §11 jump-to-the-composer binding, on both planes: bare `i` from the
/// keyboard plane, Ctrl+I from anywhere at all (it survives text focus).
#[test]
fn i_and_ctrl_i_jump_the_keyboard_to_the_composer() {
    let mut world = world();
    let screen = Screen::new();
    screen.idle(&mut world);
    screen.release(&mut world);
    screen.frame(&mut world, vec![press(egui::Key::I, egui::Modifiers::NONE)]);
    assert!(screen.idle(&mut world), "bare `i` claims the box");

    screen.release(&mut world);
    screen.frame(
        &mut world,
        vec![press(egui::Key::I, egui::Modifiers::COMMAND)],
    );
    assert!(screen.idle(&mut world), "and so does Ctrl+I");
}

/// The one place Ctrl+I could not reach before: the `new workspace` form used
/// to `request_focus` unconditionally every frame, out-shouting the request
/// forever. It claims the keyboard on open and then lets go on demand.
#[test]
fn the_new_workspace_form_claims_the_keyboard_but_ctrl_i_can_take_it_back() {
    let mut world = world();
    let screen = Screen::new();
    screen.idle(&mut world);
    // Ctrl+Shift+N opens the form — the combo plane, so it lands even with the
    // composer already holding the keyboard. The form then takes it.
    screen.frame(&mut world, vec![press(egui::Key::N, command_shift())]);
    assert!(
        world.state.new_ws.open,
        "Ctrl+Shift+N opened the §11 name form (the precondition, not the claim)"
    );
    assert!(screen.idle(&mut world), "the form holds the keyboard");

    screen.frame(
        &mut world,
        vec![press(egui::Key::I, egui::Modifiers::COMMAND)],
    );
    assert!(
        screen.idle(&mut world),
        "and Ctrl+I still reaches the composer from inside it"
    );
}

/// Dismissing a modal hands the keyboard back. The §3.6 delete dialog is the
/// one whose dismissal a headless frame can actually walk — its subject
/// vanishing is a real door, where the ✕ is a click and unreachable — and the
/// hand-back is read as one edge for all three doors, so this covers them.
#[test]
fn a_dismissed_modal_hands_the_keyboard_back() {
    let mut world = world();
    let ws = world.ws.clone();
    let screen = Screen::new();
    screen.idle(&mut world);
    screen.release(&mut world);
    assert!(
        crate::boundary::answer::confirmation_of(&world.model.snap, &ws).is_none(),
        "the fixture workspace is foreign — §3.6 offers no confirmation on it, \
         which is what makes its dialog close on the frame it opens"
    );
    super::super::delete::open(&world.model, &mut world.state, &crate::naming::leaf(&ws));
    assert!(
        !screen.idle(&mut world),
        "the modals paint last, so the frame that dismisses one has already \
         drawn the composer: the request is for the frame after"
    );
    assert!(world.state.delete.target.is_none(), "and it did dismiss");
    assert!(
        screen.idle(&mut world),
        "the next frame puts the cursor back in the box"
    );
}

/// Sending a message keeps the keyboard: Enter is spent as `lost_focus` and a
/// click never had it, so without the hand-back the next message starts with a
/// hunt for the box.
#[test]
fn sending_a_message_keeps_the_keyboard() {
    let mut world = world();
    let ws = world.ws.clone();
    world.model.focus_agent(&ws, "c-1");
    world.state.actions.drafts.set(
        crate::actions::DraftKey::Message("c-1".to_owned()),
        "ping".to_owned(),
    );
    let screen = Screen::new();
    // Launch already put the cursor in the box, which is the precondition Enter
    // needs: the composer's send is the focused box's own plain Enter
    // (bl-4515 — the multiline widget's return key is Shift+Enter, so the
    // plain press stays whole for the send).
    assert!(screen.idle(&mut world), "the cursor starts in the box");
    screen.frame(
        &mut world,
        vec![press(egui::Key::Enter, egui::Modifiers::NONE)],
    );
    assert!(
        screen.idle(&mut world),
        "the send left the cursor where the next message gets typed"
    );
}

/// The mechanism itself: a request is spent exactly once, by whichever composer
/// paints — never held, never re-grabbing a box the operator has left.
#[test]
fn a_request_is_spent_once() {
    let mut world = world();
    assert!(world.state.focus_composer, "launch stands the request up");
    let screen = Screen::new();
    screen.idle(&mut world);
    assert!(!world.state.focus_composer, "the composer spent it");
    focus::request(&mut world.state);
    focus::request(&mut world.state);
    assert!(world.state.focus_composer, "asking twice is asking once");
    screen.idle(&mut world);
    assert!(!world.state.focus_composer, "and one frame clears it");
}
