//! **Capability theater is the other direction** (bl-5894, DESIGN §16.2 as
//! amended by the blast-radius ruling): a live capability that goes on painting,
//! and goes on being actionable, under the *next* workspace.
//!
//! Split from [`super`] at §12's budget on the seam that file's own doc names —
//! bl-c0e2's one move was two defects pulling opposite ways, and no amount of
//! clearing fixes both. There the cost is **data**: an unsaved draft thrown away
//! by a switch, or handed on to the sphere that merely reoccupies a dead one's
//! path. Here it is **reach**: a `bz --login` stream writing A's credential
//! printed under B, and a picker opened against A's providers left open where a
//! click in it would write B's config lineage from A's candidate set. Neither
//! half is evidence for the other, and only a wall that owns the box holds both.

use std::path::Path;

use super::super::fixture::World;
use super::super::screen::Screen;
use super::two_spheres;
use crate::cli_outbound::Cli;
use crate::keymap::CenterTab;

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

    world.model.focus_workspace(&crate::naming::leaf(&b));
    let under_b = screen.text(&mut world);
    assert!(
        under_b.contains("Login (bz browser sign-in)"),
        "B has a Login surface of its own:\n{under_b}"
    );
    assert!(
        !under_b.contains(line),
        "but A's authorize line is not on it — a device code is for one sphere:\n{under_b}"
    );

    world.model.focus_workspace(&crate::naming::leaf(&a));
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

    world.model.focus_workspace(&crate::naming::leaf(&b));
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

    world.model.focus_workspace(&crate::naming::leaf(&a));
    world.converge();
    let back = settle_for(&screen, &mut world, PICKER_SCOPE);
    assert!(
        world.state.wall.picker.open,
        "and A's picker is still where A left it"
    );
    assert!(back.contains(PICKER_SCOPE), "on screen again:\n{back}");
}
