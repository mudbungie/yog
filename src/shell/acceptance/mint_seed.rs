//! **A seed lives exactly as long as the prediction it backs** (bl-28ba), driven
//! through the real window.
//!
//! The §3.3 mint takes ONE draw off `SplitMix64::from_seed(mint_seed)`, and the
//! seed was rolled once at shell construction and never again. So every fire of
//! a session drew the same start index into the pool; the second landed on an
//! occupied slot, and the collision walk — a forward step through a pool that is
//! first-word-major — handed back the *next* slot, which shares its first word.
//! Three starts, three siblings: `recite-a`, `recite-b`, `recite-c`. Unique, and
//! unreadable as a fleet.
//!
//! Both directions are asserted here, because the fix is a lifetime and a
//! lifetime has two ends: a landed fire retires the seed it spent (the
//! prediction became the fired `--name`), and nothing else does — a launch that failed
//! minted no name, so its prediction still stands and its seed with it.

use super::fixture::{MINT_SEED, MINTED, MINTED_FIRST, fake_lernie, seed_world, world};
use super::screen::{Screen, press};
use crate::cli_outbound::Cli;
use crate::start::Prepared;
use crate::test_support::spawn_guard;
use tempfile::tempdir;

/// The greyed §3.3 prediction on screen — the operator's whole view of the mint,
/// and the only thing the seed is *for*. Exactly one is ever painted (§11: one
/// box, one Enter), so the line is the fact.
fn predicted(screen: &str) -> String {
    screen
        .lines()
        .find(|line| line.starts_with("will be named "))
        .expect("the composer previews the name its Enter would mint")
        .to_owned()
}

/// Type a goal into the docked composer and press Enter — the start the operator
/// repeats for each new agent. The §11 birth block's work-directory box holds
/// its pre-filled default (bl-7927), so this is the ordinary path rung.
fn fire(screen: &Screen, world: &mut super::fixture::World, goal: &str) {
    screen.frame(world, vec![egui::Event::Text(goal.to_owned())]);
    screen.frame(world, vec![press(egui::Key::Enter, egui::Modifiers::NONE)]);
    world.converge();
}

#[test]
fn consecutive_fires_each_predict_and_spend_a_seed_of_their_own() {
    let bin = tempdir().unwrap();
    let mut world = world();
    seed_world(&world);
    let _g = spawn_guard();
    let screen = Screen::with_lernie(fake_lernie(bin.path()));
    assert!(screen.idle(&mut world), "the cursor starts in the composer");

    let first = predicted(&screen.text(&mut world));
    let seed = world.state.start.mint_seed;
    // The fixture pins its seed (bl-cba6), so the opening prediction is not
    // merely *stable* — it is a known word, and this is where that pin is
    // stated. A wordlist edit that moves it fails here, not as a needle
    // collision in some unrelated test's `Screen::text`.
    assert_eq!(seed, MINT_SEED, "the world starts from the pinned seed");
    assert_eq!(
        first,
        format!("will be named {MINTED_FIRST}"),
        "and the pinned seed's one draw is the word the suite may name"
    );
    fire(&screen, &mut world, "open the gate");

    // The seed is the assertable half — the draw itself is not. Its successor is
    // taken only on `Ok`, so a changed seed is also the proof the fire landed.
    let respun = world.state.start.mint_seed;
    assert_ne!(
        respun, seed,
        "a landed fire retires the seed it spent (bl-28ba)"
    );
    // Named, not merely *different* (bl-dd3d): the successor comes off the spent
    // seed's own stream, so with the opening seed pinned every later prediction
    // is a known word too. `assert_ne!` here used to be a coin flip over
    // lernie's 541-word pool — it flaked twice in one day, once taking an
    // unrelated close gate with it. A repeat now fails every run, and names the
    // word it repeated.
    assert_eq!(
        predicted(&screen.text(&mut world)),
        format!("will be named {}", MINTED[1]),
        "so the next agent is predicted a fresh name, not the walk's next slot"
    );

    // Three, because three is what the operator complained about: the defect was
    // not one repeat but a run of siblings off one start index.
    fire(&screen, &mut world, "shut the gate");
    assert_ne!(
        world.state.start.mint_seed, respun,
        "and every fire after it retires its own"
    );
    assert_eq!(
        predicted(&screen.text(&mut world)),
        format!("will be named {}", MINTED[2]),
        "three fires, three named-in-advance names"
    );
    // The rule the three names carry, stated over the pins themselves so it
    // holds however the corpus moves: re-pinning a changed sequence cannot
    // quietly re-admit a repeat.
    assert_eq!(
        MINTED
            .iter()
            .collect::<std::collections::HashSet<_>>()
            .len(),
        MINTED.len(),
        "three fires, three unrelated names"
    );
}

#[test]
fn a_launch_that_never_left_the_ground_keeps_its_prediction() {
    let mut world = world();
    // `Screen::new`'s deliberately absent `lernie`: the spawn fails, so the fire
    // minted nothing and stamped nothing.
    let screen = Screen::new();
    let absent = Cli::new("yog-absent-lernie");
    let ws = world.ws.clone();
    world.state.start.pending = Some(Prepared {
        name: "ws".to_owned(),
        workspace: ws.clone(),
        binding: None,
        goal: "fix the gate".to_owned(),
        origin: crate::opslog::Origin::Balls,
    });

    let before = predicted(&screen.text(&mut world));
    let seed = world.state.start.mint_seed;
    assert!(
        !super::super::start_pane::send_pending(
            &mut world.model,
            &mut world.state.start,
            &absent,
            &absent
        ),
        "the absent binary fails the launch"
    );

    assert_eq!(
        world.state.start.mint_seed, seed,
        "a failed launch spends no seed — nothing took the name it predicted"
    );
    assert!(
        world.state.start.pending.is_some(),
        "the goal stands, so the prediction over it must too"
    );
    assert_eq!(
        predicted(&screen.text(&mut world)),
        before,
        "and the retry is offered the same name the first attempt promised"
    );
}

#[test]
fn the_ball_rungs_send_retires_the_seed_the_same_way() {
    let bin = tempdir().unwrap();
    let mut world = world();
    seed_world(&world);
    let _g = spawn_guard();
    let lernie = fake_lernie(bin.path());
    let ws = world.ws.clone();
    world.state.start.pending = Some(Prepared {
        name: "ws".to_owned(),
        workspace: ws.clone(),
        binding: None,
        goal: "fix the gate".to_owned(),
        origin: crate::opslog::Origin::Balls,
    });
    let seed = world.state.start.mint_seed;

    // The §8.1 pane's Send — the other hand on the same trigger (the §11 Enter
    // binding calls this too). One rule, both hands: the ball rung's fire spends
    // its prediction exactly as the composer's does.
    assert!(super::super::start_pane::send_pending(
        &mut world.model,
        &mut world.state.start,
        &lernie,
        &lernie
    ));
    assert_ne!(
        world.state.start.mint_seed, seed,
        "a landed ball-rung fire retires its seed too"
    );
    assert!(
        world.state.start.pending.is_none(),
        "and the draft it fired is spent with it"
    );
}
