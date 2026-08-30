//! The §11 **birth-config block** reaches the paint layer (bl-824e): with a
//! workspace focused and no conversation selected, the center is not an empty
//! seat — it holds the parameters the next conversation will be born with, at
//! the TOP, in the header's own place.

use super::fixture::world;
use super::painted;
use crate::cli_outbound::Cli;

/// Focus the workspace, select nothing: the block paints the ONE §9.4 model row
/// — the two dropdowns over the config-branch head a `litany prompt` would fork
/// (bl-cd2a). The head's own oid rides their hover; the line is the pair.
#[test]
fn an_unselected_workspace_paints_the_birth_config_block() {
    let (litany, bl) = (Cli::new("litany"), Cli::new("bl"));
    let mut world = world();
    let ws = world.ws.clone();
    // `focus_workspace` selects no agent — the very state the block is for.
    world.model.focus_workspace(&crate::naming::leaf(&ws));
    let out = painted(&mut world, &litany, &bl);
    assert!(out.contains("new conversation"), "block heading:\n{out}");
    // What the branch head assigns, not a placeholder for it: since bl-a842 the
    // fixture's `config/default` carries litany's own template, so the pair a
    // conversation started here would be born on is a real one. The `(none)`
    // this used to read was the *absent* assignment — a state a `litany new`
    // workspace is never in, and one whose row is pinned where it belongs, in
    // `model_pick::tests::header`.
    assert!(
        out.contains("anthropic") && out.contains("claude-sonnet-5"),
        "the two dropdowns show what the branch head assigns the worker role:\n{out}"
    );
    assert!(
        !out.contains("change…"),
        "the row IS the selection: no affordance stands in front of it:\n{out}"
    );
}

/// bl-7927: an editable text box with the default pre-chosen, at the top in
/// the config block rather than at the bottom beside the message. One carrier, at the top, pre-filled — and
/// the composer's old `dir (optional)` box is **gone**, not duplicated.
#[test]
fn the_work_directory_box_rides_the_block_pre_filled_and_leaves_the_composer() {
    let (litany, bl) = (Cli::new("litany"), Cli::new("bl"));
    let mut world = world();
    let ws = world.ws.clone();
    world.model.focus_workspace(&crate::naming::leaf(&ws));
    // What the fixture's env resolves `~` to — the bare rung's driver cwd
    // (§3.4), which is exactly what the box is seeded with.
    let home = world.state.actions.path_dir.clone();
    assert!(!home.is_empty(), "the box is born holding the default");
    let out = painted(&mut world, &litany, &bl);
    assert!(out.contains("work directory:"), "the block's row:\n{out}");
    assert!(
        out.contains(&home),
        "pre-filled with the bare rung's own resolution:\n{out}"
    );
    assert!(
        !out.contains("dir (optional)"),
        "the composer no longer carries the same fact:\n{out}"
    );
}

/// bl-6191: a work directory that is not there is refused **at the field**, in
/// §3.1's idiom — a sentence beside the box, no spawn, no ops wound. Before
/// this, Enter fired and the fork's ENOENT was reported against the *program*:
/// "failed to spawn `<yog binary>`: No such file or directory", telling an
/// operator who typed a bad directory that their binary was missing.
#[test]
fn a_work_directory_that_is_not_there_flags_red_at_the_field() {
    let (litany, bl) = (Cli::new("litany"), Cli::new("bl"));
    let mut world = world();
    let ws = world.ws.clone();
    world.model.focus_workspace(&crate::naming::leaf(&ws));
    // The pre-filled default is a real directory, so the field says nothing.
    let clean = painted(&mut world, &litany, &bl);
    assert!(
        !clean.contains("work directory does not exist"),
        "the seeded default is lawful and silent:\n{clean}"
    );
    let missing = world.ws.join("nonexistent-uat-dir");
    world.state.actions.path_dir = missing.display().to_string();
    let flagged = painted(&mut world, &litany, &bl);
    assert!(
        flagged.contains(&format!(
            "work directory does not exist: {}",
            missing.display()
        )),
        "the refusal names the directory the operator typed:\n{flagged}"
    );
    assert!(
        !flagged.contains("failed to spawn"),
        "and nothing spawned to blame a binary for it:\n{flagged}"
    );
}

/// Expanding it is the §9.4 picker itself — reused, never re-implemented — and
/// the scope sentence it paints is the birth one: the pick moves the workspace
/// default, because litany takes no per-conversation config.
#[test]
fn expanding_the_block_opens_the_one_picker_scoped_to_the_workspace_default() {
    let (litany, bl) = (Cli::new("litany"), Cli::new("bl"));
    let mut world = world();
    let ws = world.ws.clone();
    world.model.focus_workspace(&crate::naming::leaf(&ws));
    // The picker is the **wall's** RAM (bl-5894), and the frame is what seats a
    // wall — so the flag is set on the sphere's own picker, after one paint, not
    // on the no-wall bundle a launch starts holding.
    let _ = painted(&mut world, &litany, &bl);
    world.state.wall.picker.open = true;
    let out = painted(&mut world, &litany, &bl);
    assert!(
        out.contains("workspace default too"),
        "the birth scope sentence, not the frozen-conversation one:\n{out}"
    );
    assert!(
        out.contains("about to start"),
        "and it names the conversation the pick governs:\n{out}"
    );
    assert!(
        !out.contains("this one stays frozen"),
        "the frozen clause names a conversation the block does not have:\n{out}"
    );
}
