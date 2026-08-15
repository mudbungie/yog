//! The composer's draft is per **target**, not one global buffer that re-labels
//! its verb (§11, §5.3, bl-a69a): a goal typed for a new conversation must not
//! follow the selection into `→ message <name>` and be sent to an unrelated
//! agent. Driven through the real keyboard, on the real window, because the bug
//! was invisible to every per-widget test — the buffer was correct, its *key*
//! was missing.

use super::fixture::{MINTED_FIRST, world};
use super::screen::{Screen, press};
use crate::actions::DraftKey;

/// Type into one target, switch to another: the second box is its own, and
/// switching back restores the first. The industry-normal rule (every chat app
/// keeps a draft per conversation), asserted at the paint layer.
#[test]
fn a_draft_belongs_to_the_target_it_was_typed_for() {
    let mut world = world();
    let ws = world.ws.clone();
    let screen = Screen::new();
    // Launch puts the cursor in the composer (§11), and nothing is selected —
    // so this is the new-conversation box, whose Enter would fire a start.
    assert!(screen.idle(&mut world), "the cursor starts in the composer");
    screen.frame(&mut world, vec![egui::Event::Text("ship the goal".into())]);
    let goal = screen.text(&mut world);
    assert!(
        goal.contains("ship the goal"),
        "the start goal is in the box:\n{goal}"
    );

    // Select an existing conversation. The composer retargets — and the goal
    // must NOT come with it: Enter here would deposit a fresh start's text as a
    // message to an agent that never asked for it.
    world.model.focus_agent(&ws, "c-1");
    let message = screen.text(&mut world);
    assert!(
        message.contains("→ message hello"),
        "the composer retargets to the selection:\n{message}"
    );
    assert!(
        !message.contains("ship the goal"),
        "the start goal must not follow the selection:\n{message}"
    );

    // This target's own draft. A one-word needle is safe again (bl-cba6): the
    // new-conversation seat below paints a §3.3 name preview, and the fixture's
    // mint seed is pinned, so that word is the known `MINTED_FIRST` rather than
    // a fresh draw from entropy that a needle could collide with by chance.
    screen.frame(&mut world, vec![egui::Event::Text("steer".into())]);
    let typed = screen.text(&mut world);
    assert!(
        typed.contains("steer"),
        "the message draft is its own:\n{typed}"
    );

    // Back to the new conversation: the goal is still there, and the message
    // stayed behind with the agent it was written to.
    world.model.focus_workspace(&crate::naming::leaf(&ws));
    let back = screen.text(&mut world);
    assert!(
        back.contains("ship the goal"),
        "switching back restores the target's own draft:\n{back}"
    );
    assert!(
        back.contains(&format!("will be named {MINTED_FIRST}")),
        "the pinned §3.3 preview is the word beside this needle:\n{back}"
    );
    assert!(
        !back.contains("steer"),
        "the message draft stayed with its agent:\n{back}"
    );
}

/// The composer's key contract (§11, bl-4515): **Shift+Enter inserts a newline
/// at the cursor without sending; Enter alone sends.** Driven through the real
/// keyboard because the seam is egui's own — the multiline box's return key is
/// Shift+Enter, and the plain press is read back out as the send.
#[test]
fn shift_enter_newlines_the_draft_and_enter_alone_sends() {
    let mut world = world();
    let ws = world.ws.clone();
    let screen = Screen::new();
    assert!(screen.idle(&mut world), "the cursor starts in the composer");
    world.model.focus_agent(&ws, "c-1");
    let key = DraftKey::Message("c-1".to_owned());

    // A two-line draft: text, Shift+Enter, text. Nothing may fire on the combo
    // — the Screen's lernie is absent, so a send would banner in ichor.
    screen.frame(&mut world, vec![egui::Event::Text("first line".into())]);
    screen.frame(
        &mut world,
        vec![press(egui::Key::Enter, egui::Modifiers::SHIFT)],
    );
    screen.frame(&mut world, vec![egui::Event::Text("second line".into())]);
    assert_eq!(
        world.state.actions.drafts.text(&key),
        "first line\nsecond line",
        "Shift+Enter put a newline at the cursor, and only that"
    );
    world.converge();
    assert!(
        world
            .model
            .last_failure(crate::opslog::Origin::Conversation)
            .is_none(),
        "and it dispatched nothing"
    );
    let quiet = screen.text(&mut world);
    assert!(
        quiet.contains("first line") && quiet.contains("second line"),
        "the multi-line draft reaches the paint layer on both lines:\n{quiet}"
    );

    // Enter alone is the send. The absent lernie refuses the spawn, which is
    // the proof the verb fired — and a refused send keeps the draft (§5.3)
    // exactly as typed: no third line, because the plain press never reaches
    // the box as a newline.
    screen.frame(
        &mut world,
        vec![press(egui::Key::Enter, egui::Modifiers::NONE)],
    );
    world.converge();
    assert_eq!(
        world.state.actions.drafts.text(&key),
        "first line\nsecond line",
        "Enter was spent as the send, never as a newline"
    );
    assert!(
        world
            .model
            .last_failure(crate::opslog::Origin::Conversation)
            .is_some(),
        "the send really dispatched (the absent binary leaves its ops row)"
    );
}
