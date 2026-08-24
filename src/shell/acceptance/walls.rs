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
//!
//! **The two directions are two files** (§12's budget, on the seam the doc
//! above already names): this one holds the **data** half — an unsaved draft
//! survives a workspace round trip, and dies with the sphere it was typed in —
//! and [`theater`] the **reach** half, where a live capability goes on acting
//! under the next wall. The two-sphere fixture both run on is here.

/// The reach half: a live sign-in stream and an open picker are the wall's too.
mod theater;

use super::fixture::{World, world};
use super::screen::{Screen, click, locate};
use crate::keymap::CenterTab;

/// The raw TOML an operator typed into workspace A's brazen editor and never
/// applied — §5.3's carve-out ("text typed in a box can live in RAM until
/// sent"), which a workspace switch is not.
const DRAFT_A: &str = "yog-5894-unsaved-draft";

/// The header the §9.1 raw editor folds behind — clicked open so the draft
/// itself reaches the paint layer, rather than asserted from RAM alone.
const RAW_HEADER: &str = "raw config.toml";

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
pub(super) fn two_spheres(screen: &Screen) -> (World, std::path::PathBuf, std::path::PathBuf) {
    let mut world = world();
    let a = world.ws.clone();
    let b = world.add_workspace("other", "c-2");
    world.converge();
    world.model.focus_workspace(&crate::naming::leaf(&a));
    screen.idle(&mut world);
    (world, a, b)
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
    world.model.focus_workspace(&crate::naming::leaf(&b));
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
    world.model.focus_workspace(&crate::naming::leaf(&a));
    let back = screen.text(&mut world);
    assert!(
        back.contains(DRAFT_A),
        "returning to A restores the draft it was left with:\n{back}"
    );
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

    world.model.focus_workspace(&crate::naming::leaf(&b));
    screen.idle(&mut world);
    world.model.focus_workspace(&crate::naming::leaf(&a));
    let reborn = screen.text(&mut world);
    assert!(
        !reborn.contains(DRAFT_A),
        "a dead sphere's draft is not handed to the path's next occupant:\n{reborn}"
    );
}
