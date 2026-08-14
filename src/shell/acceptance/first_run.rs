//! **A stranger can sign the sphere in before spending a turn** (bl-3b62, ruled
//! at bl-9b52 Q3), driven through the real window from the empty world.
//!
//! Sign-in used to surface only as derived agent state *after* an auth-failed
//! step (§13.3), so discovering it cost a conversation and a dead first turn.
//! The §11 Login tab was always reachable from the empty world — the strip and
//! the navigator entry both name it — but it answered with its own
//! empty-roster sentence, because the §16.2 wall lens rode
//! `focused_workspace()` and a stranger has no focus. It rides §3.4's
//! `start_workspace` now, so the roster the empty world paints is the wall of
//! the sphere the first message founds.
//!
//! What is asserted here is both halves of that, because either alone would
//! pass on the defect: the roster **reaches the glass with rows on it**, and
//! the wall those rows and any sign-in are bound to is the one the founded
//! workspace will read.

use super::fixture::{world, world_empty};
use super::screen::{Screen, command_shift, press};
use crate::keymap::CenterTab;
use crate::names::DEFAULT_NAME;

/// Ctrl+Shift+3 from the empty world — **with the keyboard resting in the
/// bootstrap box**, which is the only state that world is ever in (§11 focus
/// discipline opens it focused) — lands on a populated §8.3 roster.
///
/// QUALITY F1: the reach is a §11 binding, not a pointer path. QUALITY H2 in
/// the other direction: the pane's "brazen listed no provider rows" line is the
/// honest answer to an empty roster and the wrong one here, so its absence is
/// asserted too.
#[test]
fn the_empty_world_reaches_a_populated_roster_from_inside_the_box() {
    let mut world = world_empty();
    let screen = Screen::new();
    assert!(
        screen.idle(&mut world),
        "the empty world opens with the keyboard in the bootstrap box"
    );
    assert!(
        world.model.focused_workspace().is_none(),
        "and with nothing focused, or this is not the state under test"
    );

    screen.frame(&mut world, vec![press(egui::Key::Num3, command_shift())]);
    assert_eq!(
        world.state.center,
        CenterTab::Login,
        "Ctrl+Shift+3 focuses the Login tab without the box letting go first"
    );

    let painted = screen.text(&mut world);
    let rows = world.state.wall.login.rows.clone();
    assert!(
        !rows.is_empty(),
        "the roster is the newborn wall's, and brazen's shipped table is not empty"
    );
    for row in &rows {
        assert!(
            painted.contains(&row.name) && painted.contains(&row.fact),
            "every row reaches the glass with its credential fact:\n{painted}"
        );
    }
    assert!(
        !painted.contains("brazen listed no provider rows"),
        "the empty-roster sentence is the answer to a different question:\n{painted}"
    );
    assert!(
        painted.contains(DEFAULT_NAME),
        "and the pane names the sphere a sign-in here lands in:\n{painted}"
    );
}

/// The sphere is **derived**, not the empty world's constant: the same line
/// under a focused workspace names that workspace. Without this the assertion
/// above passes on a pane that has `home` written into it.
#[test]
fn the_roster_names_whichever_sphere_the_window_is_pointed_at() {
    let mut world = world();
    let screen = Screen::new();
    screen.idle(&mut world);
    crate::shell::center::focus(&world.model, &mut world.state, CenterTab::Login);
    let painted = screen.text(&mut world);
    let leaf = world
        .ws
        .file_name()
        .expect("the fixture workspace has a leaf")
        .to_string_lossy()
        .into_owned();
    assert_ne!(leaf, DEFAULT_NAME, "the fixture must not be named `home`");
    assert!(
        painted.contains(&format!("belong to the workspace {leaf}")),
        "the focused sphere is the one named:\n{painted}"
    );
}

/// The load-bearing half: the wall the roster was read against — and that any
/// `bz --login` fired here is spawned with — is the wall of the workspace the
/// first message founds. A roster painted against some other sphere would look
/// identical and sign the operator into nothing.
#[test]
fn the_newborn_rosters_wall_is_the_one_the_first_message_will_use() {
    let mut world = world_empty();
    let screen = Screen::new();
    screen.idle(&mut world);
    let holder = &world.state.wall.login;
    assert_eq!(
        holder.workspace.as_deref(),
        Some(world.model.start_workspace().as_path()),
        "the roster belongs to the sphere §3.4 says the next Enter lands in"
    );
    let wall = holder
        .wall
        .iter()
        .find(|(key, _)| key == crate::world::wall::YOG_WALL)
        .map(|(_, value)| std::path::PathBuf::from(value))
        .expect("the spawn layer names the wall");
    assert_eq!(
        wall.file_name().map(std::ffi::OsStr::to_string_lossy),
        Some(DEFAULT_NAME.into()),
        "and it is `<world>/walls/home` — the name IS the wall (§16.2), so the \
         credential written here is the one the founded `home` reads"
    );
}
