//! **The request itself** (§11, `super::super::focus`): raised at launch, spent
//! exactly once, by whichever composer paints.
//!
//! Split from [`super`] at §12's budget on the seam that file's own doc names —
//! *"after launch and after each basic operation"*. Those beats ask what an
//! **operation** does to the keyboard, over the populated world an operator
//! works in; these ask what the mechanism does on its own, which is why the
//! empty world is here: the bootstrap composer is not a special case with a
//! focus flag of its own, and proving that takes a frame with no workspace in
//! it at all.

use super::super::super::focus;
use super::super::fixture::{world, world_empty};
use super::super::screen::Screen;

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
