//! **The §8.5 Search tab is offered, never permanent** (bl-1ca2) — and offered
//! on the *question*, never on the hits (bl-648a).
//!
//! Split from [`super::tabs`] at §12's per-file budget, on the seam the two
//! files divide on: that one asserts what holds of **every** center tab — a
//! reachable strip, a combo from inside the box, Escape home, no tab painting
//! over the conversation's accessories — while the Search tab alone comes and
//! goes, and the rule that decides *when* is a surface of its own. Both drive
//! the real window.

use super::fixture::world;
use crate::keymap::CenterTab;

/// The §8.5 results are **a view of the published answer**, so the tab is
/// offered exactly while there is one: a landed search focuses it without a
/// second gesture, and an empty query clears the answer, retires the tab and
/// drops the center home — the vanishing *is* the dismissal, and there is
/// still no search mode to enter or leave.
#[test]
fn the_search_tab_is_offered_with_an_answer_and_goes_when_the_answer_does() {
    let (litany, bl) = (
        crate::cli_outbound::Cli::new("litany"),
        crate::cli_outbound::Cli::new("bl"),
    );
    let mut world = world();
    let quiet = super::painted(&mut world, &litany, &bl);
    assert!(
        !quiet.lines().any(|line| line == "Search"),
        "no answer, no tab:\n{quiet}"
    );

    // The ask, through the composer's own line seat — the same door Ctrl+F
    // opens the sentence for.
    let key = crate::actions::DraftKey::composer(Some(world.ws.clone()), None);
    world
        .state
        .actions
        .drafts
        .set(key.clone(), "/search hello".to_owned());
    assert!(crate::shell::slash::run(
        &mut world.model,
        &mut world.state,
        &litany,
        &bl,
        &key,
        "/search hello"
    ));
    assert_eq!(
        world.state.center,
        CenterTab::Search,
        "asking focuses the answer's tab — the ask is the operator's one gesture"
    );
    world.searches();
    let answered = super::painted(&mut world, &litany, &bl);
    assert!(
        answered.lines().any(|line| line == "Search"),
        "the tab is offered while there is an answer:\n{answered}"
    );

    // The empty query: the answer clears, so the tab goes and the center falls
    // back home rather than showing an empty peer.
    assert!(crate::shell::slash::run(
        &mut world.model,
        &mut world.state,
        &litany,
        &bl,
        &key,
        "/search"
    ));
    world.searches();
    let cleared = super::painted(&mut world, &litany, &bl);
    assert!(
        !cleared.lines().any(|line| line == "Search"),
        "a cleared answer retires its tab:\n{cleared}"
    );
    assert_eq!(
        world.state.center,
        CenterTab::Conversation,
        "and the center is home again"
    );
}

/// **A search that matched nothing paints the answer it is** (bl-648a,
/// QUALITY H2 — "an empty region says what it is and names the paved path in
/// full", precedents bl-b491, bl-b2ed).
///
/// The defect this pins is worse than a blank pane: the tab was offered on
/// "are there hits?", so a zero-hit answer un-offered the §8.5 tab and
/// `center.rs` reseated the operator on Conversation — the frame after
/// searching was byte-identical to never having searched, and the surface
/// they were reading vanished under them.
///
/// Asserted on the **painted glyphs** (bl-bc06 — `Galley::text()` is the input
/// string, so it cannot witness what reached the glass): the needle the
/// operator typed is on screen, in the pane, with the way on beneath it.
#[test]
fn a_search_that_matches_nothing_says_so_and_keeps_its_tab() {
    let (litany, bl) = (
        crate::cli_outbound::Cli::new("litany"),
        crate::cli_outbound::Cli::new("bl"),
    );
    let mut world = world();
    let key = crate::actions::DraftKey::composer(Some(world.ws.clone()), None);
    assert!(crate::shell::slash::run(
        &mut world.model,
        &mut world.state,
        &litany,
        &bl,
        &key,
        "/search zzzznotathing"
    ));
    world.searches();
    let painted = super::painted(&mut world, &litany, &bl);
    assert_eq!(
        world.state.center,
        CenterTab::Search,
        "the operator stays on the answer they asked for:\n{painted}"
    );
    assert!(
        painted.lines().any(|line| line == "Search"),
        "and its tab is still offered:\n{painted}"
    );
    assert!(
        painted
            .lines()
            .any(|line| line == "no matches for `zzzznotathing`"),
        "the pane names the absence with the needle in it:\n{painted}"
    );
    assert!(
        painted
            .lines()
            .any(|line| line == crate::search::SEARCHED_EVERYTHING),
        "and names the paved path in full:\n{painted}"
    );
}
