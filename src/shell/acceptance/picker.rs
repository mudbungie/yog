//! **The §9.4 picker pane's contents reach the paint layer** (bl-a842).
//!
//! Until the fixture carried a config lineage, [`super::model_pick::pane`] took
//! its first early return on every acceptance world — the "cannot read
//! `roles:`" refusal — and everything below it was unreachable: the role
//! strip, the two brazen-sourced dropdowns (bl-bd89), the fault a dead
//! assignment earns (bl-53be) and the write half (bl-fb6b). The two beats that
//! did open the picker ([`super::settings`]) assert panel *height*, which the
//! refusal line alone was enough to grow, so they were true and they were not
//! about the pane's contents.
//!
//! §9.4's whole shell half is coverage-excluded on the stated grounds that
//! everything a click calls is covered (`tarpaulin.toml`). That holds for the
//! judgements; it never held for the wiring, and this is the wiring's
//! reachability proof — the same role that the seed's `roles:` declares, read
//! back off the paint layer.

use super::fixture::world;
use super::painted;
use crate::cli_outbound::Cli;

/// The pane below the row, driven open on the real window. One frame first:
/// the picker is the wall's RAM (bl-5894), so the flag goes on the focused
/// sphere's own picker rather than the launch bundle's, which `focus_wall`
/// swaps out from under a flag set before the first render.
fn open_pane(world: &mut super::fixture::World) -> String {
    let (lernie, bl) = (Cli::new("yog-absent-lernie"), Cli::new("yog-absent-bl"));
    let ws = world.ws.clone();
    world.model.focus_agent(&ws, "c-1");
    let _ = painted(world, &lernie, &bl);
    world.state.wall.picker.open = true;
    painted(world, &lernie, &bl)
}

/// The refusal is gone and the strip is there: every role the seeded
/// `providers.yaml` declares is painted with the model it runs on, which is the
/// §9.4 ruling that the strip is "whatever roles the file declares, not a
/// worker/compactor special case".
#[test]
fn the_open_pane_paints_the_role_strip_the_config_declares() {
    let mut world = world();
    let out = open_pane(&mut world);
    assert!(
        !out.contains("cannot read `roles:`"),
        "the fixture's workspace carries a config lineage now:\n{out}"
    );
    assert!(out.contains("role"), "the strip's own label:\n{out}");
    for role in ["worker · claude-sonnet-5", "compactor · claude-haiku-4-5"] {
        assert!(
            out.contains(role),
            "the strip names {role:?} — the assignment read off the branch, \
             beside the role it is scoped to:\n{out}"
        );
    }
}

/// The scope claim the pane makes about a write, on the paint layer: the branch
/// it moves, the workspace it moves it for, and the conversation it does **not**
/// move — §9.4's whole blast-radius sentence, unreachable before this fixture
/// had a branch to name.
#[test]
fn the_open_pane_states_the_blast_radius_of_a_pick() {
    let mut world = world();
    let out = open_pane(&mut world);
    assert!(
        out.contains("changes config/default for the whole ws workspace"),
        "the branch and the workspace a pick moves:\n{out}"
    );
    assert!(
        out.contains("this one stays frozen at"),
        "and the conversation it exempts, named by its governing commit:\n{out}"
    );
}

/// The §9.2 judgement surfaced at the point of choice (bl-53be): this world's
/// global `models.yaml` declares only `m`, so both seeded roles name a model
/// lernie would refuse to load — and the picker says so *before* the fire
/// instead of after it. The mark rides the strip; the reason is spelled out
/// under the selected role.
#[test]
fn a_role_whose_model_is_undeclared_wears_its_fault_in_the_pane() {
    let mut world = world();
    let out = open_pane(&mut world);
    assert!(
        out.contains("worker · claude-sonnet-5 ⚠"),
        "the dead assignment is marked in the strip:\n{out}"
    );
    assert!(
        out.contains(
            "⚠ claude-sonnet-5 is not declared in models.yaml — lernie refuses to load a \
             config whose role names an undeclared model"
        ),
        "and the selected role's reason is painted in full under it:\n{out}"
    );
}
