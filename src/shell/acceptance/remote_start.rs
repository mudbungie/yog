//! **A start fired at a workspace a §8.2 entry hosts founds nothing here**
//! (REMOTE §8.2, bl-e349), driven through the real window.
//!
//! The defect this file is the guard for was silent and it was destructive.
//! `Snapshot::ws_path` answers `Err` for a workspace held on another box —
//! REMOTE §8.2 rules that deliberately: *"a remote name still has no local
//! PATH, on purpose"* — and the start pane flattened that `Err` with
//! `unwrap_or_default()` and posted the act anyway, aimed at `PathBuf::new()`.
//! One rung up, `start_workspace` read the focused *path* rather than the
//! focused *name*, so a remote focus was indistinguishable from no focus at all
//! and took the §3.1 bootstrap default. The two together fired the operator's
//! goal at a LOCAL workspace called `home`, founded it, and focused it — a tab
//! flip and silence, with the conversation running in a wall nobody had asked
//! for.
//!
//! Both beats read the posted act **off the outbox without answering it**. The
//! fixture holds one engine and answers every act through it regardless of
//! routing, so answering here would found the entry's workspace locally and
//! make "nothing was founded" unreadable. Which channel the act then goes down
//! is `wire::channels`' and `wire::dial`'s own question, proven there; what is
//! proven here is the only thing a window decides — the name the act carries,
//! and that nothing on this box was made to carry it.

use super::fixture::world;
use super::screen::{Screen, press};
use crate::boundary::Action;
use crate::start::Prepared;

/// The entry's leaf — a name this box holds no directory for, and one that
/// shares no substring with any other needle this world paints.
const ENTRY: &str = "cobalt";

/// What the operator types. Its own constant so the assertion that it reached
/// the box names the same string the Enter then fires.
const GOAL: &str = "do the thing";

/// The world with one §8.2 entry attached and its roster row landed, focused on
/// the workspace that entry hosts. This is the operator's posture in the report:
/// a remote workspace selected, the start pane open over it.
fn remote_focus(screen: &Screen) -> super::fixture::World {
    let mut world = world();
    world.attach_entry(ENTRY);
    // Two frames: the first declares the union roster, the second paints with
    // the entry's answer landed — the asker's own two-pass shape.
    screen.text(&mut world);
    screen.text(&mut world);
    world.model.focus_workspace(ENTRY);
    assert_eq!(
        world.model.hosting_entry(ENTRY).as_deref(),
        Some(ENTRY),
        "the premise: the union says another engine holds it"
    );
    assert_eq!(
        world.model.focused_workspace(),
        None,
        "and this box has no path for it (REMOTE §8.2)"
    );
    world
}

/// **The prepare.** The composer's Enter posts `Action::Prepare` naming the
/// workspace the operator is looking at — never `home`, which is a *local* name
/// and would be founded here — and its receipt chains the `Prompt` at the same
/// name, so the whole §8.1 pair lands at the host.
#[test]
fn the_composers_enter_at_a_remote_focus_runs_the_whole_pair_at_the_host() {
    let screen = Screen::new();
    let mut world = remote_focus(&screen);
    let names_root = crate::binding::names_root(&world.yog_data);
    let before = crate::binding::workspaces(&world.yog_data, &world.lernie_workspaces).len();

    // The keyboard into the goal box, then the §11 Enter — one box, one Enter.
    crate::shell::focus::request(&mut world.state);
    screen.frame(&mut world, Vec::new());
    screen.frame(&mut world, vec![egui::Event::Text(GOAL.to_owned())]);
    assert!(
        screen.text(&mut world).contains(GOAL),
        "the goal is in the box before the Enter that fires it"
    );
    screen.frame(
        &mut world,
        vec![press(egui::Key::Enter, egui::Modifiers::NONE)],
    );

    let named: Vec<Option<String>> = world.acted.iter().map(Action::workspace).collect();
    assert_eq!(
        named,
        vec![Some(ENTRY.to_owned()), Some(ENTRY.to_owned())],
        "the §8.1 pair, both halves addressed at the focused workspace — which \
         is what routes them to its host: {:?}",
        world.acted
    );
    assert!(
        matches!(world.acted.first(), Some(Action::Prepare { .. }))
            && matches!(world.acted.get(1), Some(Action::Prompt { .. })),
        "prepare then prompt: {:?}",
        world.acted
    );
    founded_nothing(&world, &names_root, before);
}

/// **The fire.** With a goal already composed, Enter posts `Action::Prompt`
/// carrying the prepared workspace's own name — the site that flattened an
/// unresolvable path to `PathBuf::new()` and fired the act anyway.
#[test]
fn a_send_at_a_remote_focus_founds_no_local_workspace() {
    let screen = Screen::new();
    let mut world = remote_focus(&screen);
    let names_root = crate::binding::names_root(&world.yog_data);
    let before = crate::binding::workspaces(&world.yog_data, &world.lernie_workspaces).len();

    // Exactly what a landed `Prepare` leaves: the goal box, open on the
    // workspace it prepared.
    world.state.start.pending = Some(Prepared {
        workspace: ENTRY.to_owned(),
        binding: None,
        lineage: None,
        goal: "ship it".to_owned(),
        origin: crate::opslog::Origin::Conversation,
    });
    let pane = screen.text(&mut world);
    assert!(
        pane.contains(&format!("Start goal → {ENTRY}")),
        "the composer is up on the remote workspace:\n{pane}"
    );

    screen.frame(
        &mut world,
        vec![press(egui::Key::Enter, egui::Modifiers::NONE)],
    );

    let Some(Action::Prompt { prepared, .. }) = world.acted.first() else {
        panic!(
            "the pending start's Enter is a §8.1 prompt: {:?}",
            world.acted
        );
    };
    assert_eq!(
        prepared.workspace, ENTRY,
        "fired at the workspace it was prepared in, by name"
    );
    founded_nothing(&world, &names_root, before);
    assert!(
        !screen.text(&mut world).contains("Start goal →"),
        "and the landed fire consumed the pane, exactly as a local start's does"
    );
}

/// The invariant both beats close on: this box made nothing to carry a name it
/// does not host, and the operator was not moved off what they aimed at.
fn founded_nothing(world: &super::fixture::World, names_root: &std::path::Path, before: usize) {
    assert!(
        !names_root.join("home").exists(),
        "the §3.1 bootstrap name was never even spelled here — that phantom, \
         founded and focused, is the whole of bl-e349"
    );
    assert!(
        !names_root.join(ENTRY).exists(),
        "and neither was a local stand-in wearing the entry's own name"
    );
    assert_eq!(
        crate::binding::workspaces(&world.yog_data, &world.lernie_workspaces).len(),
        before,
        "the §3.1 enumeration is exactly what it was before the Enter"
    );
    assert_eq!(
        world.model.focused_ws_name().as_deref(),
        Some(ENTRY),
        "and the operator is still looking at what they aimed at — no tab flip"
    );
}
