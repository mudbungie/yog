//! The §3.4 raise claim (bl-7407): a wall the derivation has not read is still
//! the wall the focus names, and the claim goes the moment the derivation
//! carries it.

use crate::app::tests::Harness;
use crate::nav::tabs::Kind;

/// The blocker this ball exists for. `lernie new` has returned, the enumeration
/// predates it, and the focus is a **name** — so without the claim the composer's
/// bare rung would resolve into the previous wall (the bl-9acf defect). With it,
/// every door answers on the frame the receipt landed.
#[test]
fn the_raised_wall_resolves_before_the_derivation_reads_it() {
    let mut h = Harness::new();
    let (_c, mut rig) = h.model();
    // The wall is founded AFTER the boot derivation, exactly as `lernie new`
    // founds it after the snapshot the receipt is folded onto.
    let raised = h.mint_named("ops", "c-9");
    assert!(
        !rig.model.workspaces().iter().any(|w| w.path == raised),
        "the derivation has not read it — that is the whole premise"
    );

    rig.model
        .adopt_workspace(&crate::naming::leaf(&raised), Some(&raised));

    assert_eq!(rig.model.focused_ws_name().as_deref(), Some("ops"));
    assert_eq!(
        rig.model.focused_workspace(),
        Some(raised.clone()),
        "the claim resolves the name the enumeration cannot"
    );
    assert_eq!(
        rig.model.start_bare_inputs().workspace,
        raised,
        "so the composer's bare Enter fires into the wall just raised (bl-9acf)"
    );
    assert!(
        rig.model
            .focused_tree()
            .is_some_and(|t| t.agents.is_empty()),
        "and the centre pane paints an empty workspace, not a blank frame"
    );
    let bar = rig.tab_bar();
    assert!(
        bar.tabs
            .iter()
            .any(|t| t.name == "ops" && t.selected && t.kind == Kind::Named),
        "the wall wears its own selected tab: {bar:?}"
    );
}

/// Retirement, and the one thing it protects: the painted snapshot may never
/// enumerate a workspace twice, because `by_leaf` refuses an ambiguous name and
/// the focus would stop resolving the instant the derivation caught up.
#[test]
fn the_derivation_showing_the_wall_retires_the_claim() {
    let mut h = Harness::new();
    let (_c, mut rig) = h.model();
    let raised = h.mint_named("ops", "c-9");
    rig.model
        .adopt_workspace(&crate::naming::leaf(&raised), Some(&raised));

    rig.tick();

    assert_eq!(
        rig.model
            .workspaces()
            .iter()
            .filter(|w| w.path == raised)
            .count(),
        1,
        "enumerated once — the claim went as the derivation arrived"
    );
    assert_eq!(
        rig.model.focused_workspace(),
        Some(raised),
        "and the focus resolves off the derivation now, unchanged"
    );
}

/// Every landed `Prepare` comes through the same door — ▶ Continue, the
/// bootstrap, a bare start into the wall already focused — so a start into an
/// enumerated workspace must be the general path with the claim retired at
/// once, not a branch that asks whether this one was a raise.
#[test]
fn a_start_into_an_enumerated_workspace_claims_nothing() {
    let h = Harness::new();
    let (_c, mut rig) = h.model();
    let before = rig.model.workspaces().len();

    rig.model
        .adopt_workspace(&crate::naming::leaf(&h.ws), Some(&h.ws));

    assert_eq!(rig.model.focused_ws_name().as_deref(), Some("ws"));
    assert_eq!(
        rig.model.workspaces().len(),
        before,
        "nothing was folded: the derivation already showed it"
    );
    assert_eq!(rig.model.focused_workspace(), Some(h.ws.clone()));
}

/// The claim's **row** projection (bl-296f), and the one thing it protects: the
/// §11 tab bar folds an answered listing now, and an answer that reaches the
/// seat *after* the derivation caught up — the asker's cadence is not the
/// frame's — must not put the wall in twice. `by_leaf` refuses an ambiguous
/// name, so a duplicated row would stop the focus resolving at the very moment
/// the world agreed with it.
#[test]
fn the_claim_never_doubles_a_wall_the_answer_already_carries() {
    let mut h = Harness::new();
    let (_c, mut rig) = h.model();
    let raised = h.mint_named("ops", "c-9");
    rig.model
        .adopt_workspace(&crate::naming::leaf(&raised), Some(&raised));

    // The listing as it stands with the derivation still behind: one row for
    // the claim, minted here because nothing else carries the wall.
    let claimed = crate::test_support::chrome::ws_rows(&rig.model);
    assert_eq!(
        claimed.iter().filter(|r| r.workspace == "ops").count(),
        1,
        "the claim leads the wall into the listing"
    );

    // The same claim against an answer that already enumerates the wall — the
    // ordinary shape of a landed answer newer than the last retirement pass.
    let answered = rig.model.raised_rows(claimed.clone());
    assert_eq!(
        answered.iter().filter(|r| r.workspace == "ops").count(),
        1,
        "and adds nothing to a listing that already has it"
    );
    assert_eq!(answered.len(), claimed.len(), "no row moved either");
}
