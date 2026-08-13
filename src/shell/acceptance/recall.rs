//! The §11 prompt-recall drive (bl-f908): ↑ at the composer box's top row
//! brings back what the operator already said in this conversation — pending
//! deposits ahead of the delivered transcript — and ↓ walks forward, past the
//! newest, to the draft that was displaced. Driven through the real keyboard
//! on the real window, because the whole gesture is a seam with egui: the key
//! has to be taken from the widget before it moves the caret, and the caret's
//! own row is what decides whether it may be taken at all.

use super::fixture::{World, world};
use super::screen::{Screen, press};
use crate::actions::DraftKey;

fn arrow(key: egui::Key) -> Vec<egui::Event> {
    vec![press(key, egui::Modifiers::NONE)]
}

/// The fixture's conversation has said two things: `please ping` (delivered,
/// `001-user.md`) and `follow-up message` (still pending in the inbox). Newest
/// first, that is the pending one, then the delivered one — and forward past
/// the newest is the half-typed draft, verbatim.
#[test]
fn up_pages_back_through_your_own_turns_and_down_returns_the_draft() {
    let mut world = world();
    let ws = world.ws.clone();
    world.converge();
    let screen = Screen::new();
    assert!(screen.idle(&mut world), "the cursor starts in the composer");
    world.model.focus_agent(&ws, "c-1");
    let key = DraftKey::Message("c-1".to_owned());
    screen.frame(&mut world, vec![egui::Event::Text("half-typed".into())]);
    let draft = |world: &World| world.state.actions.drafts.text(&key);
    assert_eq!(draft(&world), "half-typed");

    screen.frame(&mut world, arrow(egui::Key::ArrowUp));
    assert_eq!(
        draft(&world),
        "follow-up message",
        "↑ recalls the newest thing said here — the pending deposit"
    );
    screen.frame(&mut world, arrow(egui::Key::ArrowUp));
    assert_eq!(
        draft(&world),
        "please ping",
        "↑ again reaches the delivered transcript"
    );
    screen.frame(&mut world, arrow(egui::Key::ArrowUp));
    assert_eq!(
        draft(&world),
        "please ping",
        "the oldest turn is the end of the walk"
    );

    screen.frame(&mut world, arrow(egui::Key::ArrowDown));
    assert_eq!(draft(&world), "follow-up message", "↓ comes forward again");
    screen.frame(&mut world, arrow(egui::Key::ArrowDown));
    assert_eq!(
        draft(&world),
        "half-typed",
        "and past the newest the draft comes back verbatim"
    );
}

/// The gate, end to end: with the caret off the top row the arrow is the
/// caret's, exactly as before — a two-line draft takes one ↑ to reach its top
/// line and only then recalls.
#[test]
fn a_caret_below_the_top_row_still_owns_the_arrow() {
    let mut world = world();
    let ws = world.ws.clone();
    world.converge();
    let screen = Screen::new();
    assert!(screen.idle(&mut world), "the cursor starts in the composer");
    world.model.focus_agent(&ws, "c-1");
    let key = DraftKey::Message("c-1".to_owned());
    screen.frame(&mut world, vec![egui::Event::Text("first".into())]);
    screen.frame(
        &mut world,
        vec![press(egui::Key::Enter, egui::Modifiers::SHIFT)],
    );
    screen.frame(&mut world, vec![egui::Event::Text("second".into())]);

    screen.frame(&mut world, arrow(egui::Key::ArrowUp));
    assert_eq!(
        world.state.actions.drafts.text(&key),
        "first\nsecond",
        "the caret was on the second row: ↑ moved it, and recalled nothing"
    );
    screen.frame(&mut world, arrow(egui::Key::ArrowUp));
    assert_eq!(
        world.state.actions.drafts.text(&key),
        "follow-up message",
        "from the top row the same key recalls"
    );
}
