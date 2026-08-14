//! **One fact, one rendering** (QUALITY H1), driven end to end through the real
//! window: a fact yog derives once must reach the glass in exactly one seat, and
//! every other surface that wants it references that seat instead of repainting
//! it.
//!
//! The criterion has been violated by *pairs* of surfaces sharing a derivation —
//! two seats at one `row_views`, two seats at one live-activity class — so the
//! shape of the proof is always the same and lives in one file: paint the
//! surface that does **not** own the fact and assert the fact's own words are
//! absent from it, then paint the owner and assert they are there. Asserting
//! only the second half would pass on the defect.

use super::fixture::world;
use super::screen::{Screen, click};
use crate::keymap::CenterTab;

/// The ten brazen provider rows had two seats (bl-20cb): the §8.3 Login tab
/// painted name + credential fact + blocked reason with the sign-in verb, and
/// the §9.1 config pane painted the identical `row_views` sentences with no verb
/// at all. The roster now has one seat — the one that can act on a row — and the
/// config pane states the tally and hands over the gesture.
#[test]
fn the_provider_roster_has_one_seat_and_the_config_pane_references_it() {
    let mut world = world();
    let ws = world.ws.clone();
    world.model.focus_agent(&ws, "c-1");
    let screen = Screen::new();
    screen.idle(&mut world);

    crate::shell::center::focus(&world.model, &mut world.state, CenterTab::Config);
    let rows = world.state.wall.brazen.providers.len();
    assert!(
        rows > 0,
        "the linked brazen answers the pane with its built-in table, or this \
         proves nothing"
    );
    let config = screen.text(&mut world);
    assert!(
        config.contains(&format!(
            "{rows} provider rows are effective in this workspace"
        )),
        "the config pane states the effect the file has, counted from brazen's \
         own answer:\n{config}"
    );
    // The row sentences themselves — every credential model brazen's table can
    // carry, so the assertion cannot pass by the fixture happening to hold none
    // of one kind.
    for said in [
        "no credential needed",
        "not signed in",
        "signed in",
        "api-key provider — set the key in Config",
    ] {
        assert!(
            !config.contains(said),
            "the roster's own words must not paint in the config pane: {said:?}\n{config}"
        );
    }

    crate::shell::center::focus(&world.model, &mut world.state, CenterTab::Login);
    let login = screen.text(&mut world);
    assert!(
        login.contains("no credential needed") || login.contains("signed in"),
        "and the Login tab is where a row states its credential:\n{login}"
    );
}

/// The reference is a **control**, not a sentence to go looking for (QUALITY
/// H3a): the config pane's route reaches the roster in one press, and it is the
/// ordinary tab focus — no second way to open Login.
#[test]
fn the_config_panes_roster_reference_focuses_the_login_tab() {
    let mut world = world();
    let ws = world.ws.clone();
    world.model.focus_agent(&ws, "c-1");
    let screen = Screen::new();
    screen.idle(&mut world);
    screen.release(&mut world);
    crate::shell::center::focus(&world.model, &mut world.state, CenterTab::Config);

    let shapes = screen.shapes(&mut world, Vec::new());
    let seat = below_the_heading(&shapes, "Login").expect("the config pane paints its route");
    click(&screen, &mut world, seat);
    assert_eq!(
        world.state.center,
        CenterTab::Login,
        "the config pane's Login control spends the one tab-focus gesture"
    );
}

/// The centre of the galley reading `text` that lands **below** the brazen
/// pane's own heading — how a test names the config pane's own control rather
/// than the tab strip's peer of the same name, which paints above it.
fn below_the_heading(shapes: &[egui::epaint::ClippedShape], text: &str) -> Option<egui::Pos2> {
    let mut painted = Vec::new();
    for clipped in shapes {
        crate::paint_probe::collect(&clipped.shape, &mut painted);
    }
    let heading = painted
        .iter()
        .find(|(said, _)| said == "brazen config.toml")?
        .1
        .top();
    painted
        .iter()
        .find(|(said, rect)| said == text && rect.top() > heading)
        .map(|(_, rect)| rect.center())
}
