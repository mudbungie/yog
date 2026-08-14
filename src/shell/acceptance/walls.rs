//! **Nothing crosses a wall, and nothing is lost at one** (bl-5894, DESIGN
//! §16.2 as amended by the blast-radius ruling; §5.3's "RAM, but per
//! target, not per box").
//!
//! bl-c0e2 put brazen's config, credentials and model cache inside a workspace
//! but left the RAM *over* those files as one box per window, re-lensed when
//! focus moved. That is two defects in one move, and they pull opposite ways:
//! re-loading the box threw away an unsaved draft (data loss), while everything
//! the re-lens did not touch — a live `bz --login` stream, an open picker with
//! the roster it fetched — stayed on screen and actionable under the *next*
//! workspace (capability theater). No amount of clearing fixes both; the wall
//! has to own the box.
//!
//! These drives hold both directions on the real window, switching workspace
//! through the frame's own [`focus_wall`](crate::shell::ShellState::focus_wall):
//! away and back restores what was typed, and away shows none of it.

use std::path::Path;

use super::fixture::{World, world};
use super::screen::{Screen, click, locate};
use crate::cli_outbound::Cli;
use crate::keymap::CenterTab;

/// The raw TOML an operator typed into workspace A's brazen editor and never
/// applied — §5.3's carve-out ("text typed in a box can live in RAM until
/// sent"), which a workspace switch is not.
const DRAFT_A: &str = "yog-5894-unsaved-draft";

/// The header the §9.1 raw editor folds behind — clicked open so the draft
/// itself reaches the paint layer, rather than asserted from RAM alone.
const RAW_HEADER: &str = "raw config.toml";

/// A fragment of the birth scope sentence the §9.4 pane paints under its
/// heading — painted by the open picker and by nothing else, so its presence is
/// the picker's own.
const PICKER_SCOPE: &str = "workspace default too";

/// A `bz` that prints one line to stderr — where brazen writes its authorize
/// URL (§8.3) — and exits 0, so a login stream really is running.
fn fake_bz(dir: &Path, line: &str) -> Cli {
    use std::os::unix::fs::PermissionsExt;
    let path = dir.join("bz-login");
    std::fs::create_dir_all(dir).unwrap();
    std::fs::write(&path, format!("#!/bin/sh\necho '{line}' >&2\nexit 0\n")).unwrap();
    let mut perms = std::fs::metadata(&path).unwrap().permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(&path, perms).unwrap();
    Cli::new(path)
}

/// Fold the §9.1 raw editor open by clicking its header, so the draft behind it
/// reaches the paint layer — without this a `contains(DRAFT_A)` assertion is
/// vacuous in both directions, since the fold starts closed.
fn open_raw_editor(screen: &Screen, world: &mut World) {
    let shapes = screen.shapes(world, Vec::new());
    let header = locate(&shapes, RAW_HEADER).expect("the raw editor's fold header");
    click(screen, world, header);
}

/// A world with two spheres, focus on the first and one frame painted — the
/// frame is what seats the wall, so every drive below starts from a settled A.
fn two_spheres(screen: &Screen) -> (World, std::path::PathBuf, std::path::PathBuf) {
    let mut world = world();
    let a = world.ws.clone();
    let b = world.add_workspace("other", "c-2");
    world.converge();
    world.model.focus_workspace(&a);
    screen.idle(&mut world);
    (world, a, b)
}

/// Paint frames until `needle` shows, or give up — a streamed verb's lines
/// arrive on the drain thread, so the frame that plants the run is not
/// necessarily the frame that paints its first line.
fn settle_for(screen: &Screen, world: &mut World, needle: &str) -> String {
    let mut out = String::new();
    for _ in 0..200 {
        out = screen.text(world);
        if out.contains(needle) {
            return out;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    out
}

/// A brazen draft belongs to the **wall** it was typed in (§5.3 per target, not
/// per box). Switching workspace is not a dismissal: it hides the draft, it does
/// not destroy it, and the sphere the operator returns to is where they left it.
#[test]
fn an_unsaved_brazen_draft_survives_a_workspace_round_trip() {
    let screen = Screen::new();
    let (mut world, a, b) = two_spheres(&screen);

    // Focus the Config tab through the one gesture that carries §9's re-read,
    // then type into brazen's raw draft and fold the editor open so it paints.
    crate::shell::center::focus(&world.model, &mut world.state, CenterTab::Config);
    world
        .state
        .wall
        .brazen
        .editor
        .as_mut()
        .expect("workspace A is a wall, so it has a config.toml to edit")
        .draft_mut()
        .push_str(DRAFT_A);
    open_raw_editor(&screen, &mut world);
    let under_a = screen.text(&mut world);
    assert!(
        under_a.contains(DRAFT_A),
        "the draft is in workspace A's box:\n{under_a}"
    );

    // Workspace B. Its own brazen pane paints — a different file, its own empty
    // draft — and A's unapplied text is nowhere on screen.
    world.model.focus_workspace(&b);
    let under_b = screen.text(&mut world);
    assert!(
        under_b.contains("brazen config.toml"),
        "B has a brazen pane of its own:\n{under_b}"
    );
    assert!(
        !under_b.contains(DRAFT_A),
        "and A's draft is not in it — nothing crosses a wall:\n{under_b}"
    );

    // Back to A: verbatim, because it never went anywhere.
    world.model.focus_workspace(&a);
    let back = screen.text(&mut world);
    assert!(
        back.contains(DRAFT_A),
        "returning to A restores the draft it was left with:\n{back}"
    );
}

/// A live `bz --login` stream is workspace A's (§8.3, §5.3's streamed-piped
/// row): it writes A's credential, so it may not paint under B — and switching
/// away must not kill it either, which is why the run is *parked* with its wall
/// rather than dropped. Dropping it would SIGTERM a sign-in the operator is in
/// the middle of.
#[test]
fn a_sign_in_stream_paints_only_under_the_wall_that_started_it() {
    let screen = Screen::new();
    let (mut world, a, b) = two_spheres(&screen);
    crate::shell::center::focus(&world.model, &mut world.state, CenterTab::Login);

    let line = "yog-5894-authorize-url";
    let bz = fake_bz(&world.yog_data, line);
    let run = crate::login::start(
        &bz,
        "openai-chatgpt",
        world.model.state_root(),
        "t0",
        Some(&a),
    )
    .expect("the fake bz spawns");
    world.state.wall.login.run = Some(run);

    let under_a = settle_for(&screen, &mut world, line);
    assert!(
        under_a.contains(line),
        "A's sign-in prints where it was started:\n{under_a}"
    );

    world.model.focus_workspace(&b);
    let under_b = screen.text(&mut world);
    assert!(
        under_b.contains("Login (bz browser sign-in)"),
        "B has a Login surface of its own:\n{under_b}"
    );
    assert!(
        !under_b.contains(line),
        "but A's authorize line is not on it — a device code is for one sphere:\n{under_b}"
    );

    world.model.focus_workspace(&a);
    let back = screen.text(&mut world);
    assert!(
        back.contains(line),
        "and the run was parked with A, not killed by leaving it:\n{back}"
    );
}

/// The §9.4 picker is the wall's too. Its open flag, selected role, half-made
/// pick and the roster it fetched all ride one box, so the flag is the whole
/// question: a picker opened against A's providers must not be *on screen*
/// under B, because a click in it would write B's config lineage from A's
/// candidate set. B's picker is B's own, and it starts closed.
#[test]
fn a_picker_opened_in_one_wall_is_not_open_in_the_next() {
    let screen = Screen::new();
    let (mut world, a, b) = two_spheres(&screen);

    // The birth-config seat: nothing selected, so the settings stack asks what a
    // conversation started *now* would run on (§11, bl-824e) — one of the two
    // seats the one picker pane has.
    world.state.wall.picker.open = true;
    let under_a = settle_for(&screen, &mut world, PICKER_SCOPE);
    assert!(
        under_a.contains(PICKER_SCOPE),
        "the picker pane is open on A:\n{under_a}"
    );

    world.model.focus_workspace(&b);
    world.converge();
    let under_b = settle_for(&screen, &mut world, "work directory:");
    assert!(
        !world.state.wall.picker.open,
        "B's picker is B's own, and it was never opened"
    );
    assert!(
        !under_b.contains(PICKER_SCOPE),
        "so no write surface from A paints over B:\n{under_b}"
    );
    assert!(
        !under_b.contains(crate::model_pick::WRITE_NOTE),
        "and none of what a pick would write:\n{under_b}"
    );

    world.model.focus_workspace(&a);
    world.converge();
    let back = settle_for(&screen, &mut world, PICKER_SCOPE);
    assert!(
        world.state.wall.picker.open,
        "and A's picker is still where A left it"
    );
    assert!(back.contains(PICKER_SCOPE), "on screen again:\n{back}");
}

/// A wall's RAM lives exactly as long as its wall (§3.6, §16.2). Unmaking a
/// workspace removes its wall *directory* so a sphere created later under the
/// same §3.1 name cannot inherit its credentials — and the box over that
/// directory has to go on the same terms, because the key here is the workspace
/// path and a same-named rebirth reoccupies it exactly.
#[test]
fn unmaking_a_workspace_unmakes_its_ram_too() {
    let screen = Screen::new();
    let (mut world, a, b) = two_spheres(&screen);
    crate::shell::center::focus(&world.model, &mut world.state, CenterTab::Config);
    world
        .state
        .wall
        .brazen
        .editor
        .as_mut()
        .expect("workspace A is a wall")
        .draft_mut()
        .push_str(DRAFT_A);
    open_raw_editor(&screen, &mut world);
    let typed = screen.text(&mut world);
    assert!(
        typed.contains(DRAFT_A),
        "the draft is on screen before the sphere is unmade:\n{typed}"
    );

    // What §3.6's fire does once the unmaking landed.
    world.state.forget_wall(&a);

    world.model.focus_workspace(&b);
    screen.idle(&mut world);
    world.model.focus_workspace(&a);
    let reborn = screen.text(&mut world);
    assert!(
        !reborn.contains(DRAFT_A),
        "a dead sphere's draft is not handed to the path's next occupant:\n{reborn}"
    );
}
