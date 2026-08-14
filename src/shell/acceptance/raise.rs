//! **A raise leaves one goal box, and a blank goal never fires** (bl-9acf),
//! driven end to end through the real window.
//!
//! The raise's contract is §3.4's: found the sphere, focus it, and let the
//! docked composer be its goal box. What it did instead was open the §8.1 start
//! draft over that composer — the ball rung's mechanism reused by a rung with no
//! prefill (§3.4's table: bare composes none) — so the operator who typed a name
//! got two goal boxes and one of them was empty. Its Send was live: `lernie
//! prompt` fired with an empty goal, an ops row and a
//! model call bought with no instruction behind them.
//!
//! Both halves read one predicate ([`crate::actions::goal_present`]): a blank
//! prefill opens no draft, and a blank draft fires nothing — from the Send
//! button or from the §11 Enter binding, which is the other hand on the same
//! trigger.

use super::fixture::{fake_lernie, seed_world, world};
use super::screen::{Screen, press};
use crate::start::Prepared;
use crate::test_support::spawn_guard;
use tempfile::tempdir;

#[test]
fn a_raise_focuses_its_sphere_and_opens_no_second_goal_box() {
    let bin = tempdir().unwrap();
    let mut world = world();
    // The nested world seeded, so the fake is left holding only the one verb
    // that has to author anything (§16.6 W3).
    seed_world(&world);
    // Held across the script's write and every exec of it (the ETXTBSY window
    // `test_support` documents); re-entrant, so the flow's own `git` forks pass.
    let _g = spawn_guard();
    let screen = Screen::with_lernie(fake_lernie(bin.path()));
    screen.idle(&mut world);
    // `w` is a bare key, so the composer lets go first — the §11 idiom.
    screen.release(&mut world);

    screen.frame(&mut world, vec![press(egui::Key::W, egui::Modifiers::NONE)]);
    world.state.new_ws.typed = "ops".to_owned();
    screen.frame(
        &mut world,
        vec![press(egui::Key::Enter, egui::Modifiers::NONE)],
    );

    assert!(!world.state.new_ws.open, "Enter submitted the name form");
    let focused = world
        .model
        .focused_workspace()
        .expect("the raise focused what it raised (§3.4)");
    assert!(
        focused.ends_with("ops") && focused.join("repo.git").is_dir(),
        "and the sphere is really on disk at {}",
        focused.display()
    );
    assert!(
        world.state.start.pending.is_none(),
        "the raise's contract is focus + the composer — no start draft"
    );

    let out = screen.text(&mut world);
    assert!(
        !out.contains("Start goal →"),
        "so no second box is painted:\n{out}"
    );
    assert!(
        !out.contains("Send (detached prompt)"),
        "and nothing offers to fire an empty goal:\n{out}"
    );
    assert_eq!(
        out.matches("will be named ").count(),
        1,
        "exactly one name prediction, because there is exactly one goal box:\n{out}"
    );
    assert!(
        out.contains("start a conversation"),
        "and that box is the docked composer, aimed at the new sphere:\n{out}"
    );
    assert!(
        screen.idle(&mut world),
        "which holds the keyboard: the operator types their goal, no click"
    );
}

/// The other half, at the trigger the pointer cannot reach: a pending draft the
/// operator has emptied. Enter must be inert — the draft stays standing, and
/// nothing spawns. `lernie` here is [`Screen::new`]'s deliberately absent one,
/// so a fire would leave an ops row with a spawn error; that no failure ever
/// appears is the proof nothing was attempted.
#[test]
fn enter_on_a_blank_start_draft_fires_nothing_and_keeps_the_draft() {
    let mut world = world();
    let screen = Screen::new();
    screen.idle(&mut world);
    screen.release(&mut world);
    let ws = world.ws.clone();
    world.state.start.pending = Some(Prepared {
        name: "ws".to_owned(),
        workspace: ws.clone(),
        binding: None,
        goal: "  \n\t ".to_owned(),
        origin: crate::opslog::Origin::Conversation,
    });

    screen.frame(
        &mut world,
        vec![press(egui::Key::Enter, egui::Modifiers::NONE)],
    );
    world.converge();

    assert!(
        world.state.start.pending.is_some(),
        "a blank goal does not spend the draft — the box stays for the typing"
    );
    assert!(
        world
            .model
            .last_failure(crate::opslog::Origin::Conversation)
            .is_none(),
        "and nothing was dispatched: no `lernie prompt` on a blank payload"
    );
}
