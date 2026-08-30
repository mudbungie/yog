//! **No full-cover overlays** (bl-1ca2), driven end to end through the real
//! window.
//!
//! Several surfaces — config among them — were interface overlays toggled on
//! rather than tabs, and since they cover everything they should simply be tab
//! focuses. Three surfaces were reseated — Config, the §8.3 Login pane and the
//! §8.5 search results — and what has to hold of each is the same four things,
//! which is why they are one file: the strip is always on screen so every peer
//! is **reachable**, a combo reaches them from inside the composer where the
//! keyboard rests (**keyboard-addressable**), Escape comes home
//! (**dismissable**, QUALITY F3) losing nothing typed, and no tab paints over
//! the conversation's own accessories.
//!
//! What is true of the Search tab **alone** — that it is offered and retired,
//! and on which fact — lives in [`super::search_tab`], split off at §12's
//! per-file budget on that seam.

use super::fixture::world;
use super::screen::{Screen, click, command_shift, locate, press, rect_of};
use crate::keymap::CenterTab;

/// Ctrl+Shift+2 focuses Config **while the composer holds the keyboard** —
/// which is the state the window rests in since the focus ruling,
/// so a binding that needed the box released would be no binding at all — and
/// Escape brings the center home with the draft intact.
#[test]
fn a_combo_focuses_config_from_inside_the_box_and_escape_comes_home() {
    let mut world = world();
    let ws = world.ws.clone();
    world.model.focus_agent(&ws, "c-1");
    let screen = Screen::new();
    assert!(
        screen.idle(&mut world),
        "the keyboard rests in the composer"
    );
    // Something typed, to prove dismissal loses nothing (F3).
    world.state.actions.drafts.set(
        crate::actions::DraftKey::Message("c-1".to_owned()),
        "half a sentence".to_owned(),
    );

    screen.frame(&mut world, vec![press(egui::Key::Num2, command_shift())]);
    assert_eq!(
        world.state.center,
        CenterTab::Config,
        "Ctrl+Shift+2 focuses the Config tab without the box letting go first"
    );

    // Escape: egui spends the first on the text focus only if a box holds it —
    // the Config tab paints no composer, so the press reaches the table at
    // once and the center comes home.
    screen.frame(
        &mut world,
        vec![press(egui::Key::Escape, egui::Modifiers::NONE)],
    );
    assert_eq!(
        world.state.center,
        CenterTab::Conversation,
        "Escape dismisses the tab (QUALITY F3)"
    );
    assert_eq!(
        world
            .state
            .actions
            .drafts
            .text(&crate::actions::DraftKey::Message("c-1".to_owned())),
        "half a sentence",
        "and loses nothing typed"
    );
}

/// The strip is **co-visible with whatever it heads**: from the Config tab
/// every peer is one click away, which is the whole difference between a tab
/// and a mode toggled on over everything. And the conversation's own
/// accessories — the composer and its verbs — do not paint under a tab that is
/// not the conversation's, so nothing is painted over and nothing is stranded
/// beneath.
#[test]
fn every_peer_is_reachable_from_every_tab_and_no_tab_paints_over_the_composer() {
    let (litany, bl) = (
        crate::cli_outbound::Cli::new("litany"),
        crate::cli_outbound::Cli::new("bl"),
    );
    let mut world = world();
    let ws = world.ws.clone();
    world.model.focus_agent(&ws, "c-1");

    let home = super::painted(&mut world, &litany, &bl);
    for tab in ["Conversation", "Config", "Login"] {
        assert!(
            home.lines().any(|line| line == tab),
            "the strip offers {tab} from the conversation:\n{home}"
        );
    }
    assert!(
        home.contains("Message"),
        "the conversation carries its composer:\n{home}"
    );

    crate::shell::center::focus(&world.model, &mut world.state, CenterTab::Config);
    let config = super::painted(&mut world, &litany, &bl);
    for tab in ["Conversation", "Config", "Login"] {
        assert!(
            config.lines().any(|line| line == tab),
            "and still offers {tab} from the Config tab:\n{config}"
        );
    }
    assert!(
        config.contains("brazen config.toml"),
        "which is showing the editors:\n{config}"
    );
    assert!(
        !config.contains("→ message hello"),
        "the composer belongs to the conversation, so it is absent — not \
         buried under a full-cover pane:\n{config}"
    );
    assert!(
        !config.contains("pong reply"),
        "and the transcript is not painted over, it is simply not this tab"
    );
}

/// Login left the roster column. It used to be a `ui.collapsing` **inside** the
/// left panel, so unfolding it put ten provider rows and a live command stream
/// into a column sized for conversation titles; it is a center tab now, and
/// this asserts the seat by geometry — where the rows actually land — rather
/// than by their text, which the auth-failed banner also paints.
#[test]
fn the_login_rows_paint_in_the_center_not_in_the_roster_column() {
    let mut world = world();
    let screen = Screen::new();
    screen.idle(&mut world);
    screen.release(&mut world);
    let seat = locate(&screen.shapes(&mut world, Vec::new()), "Login")
        .expect("the left panel's Login entry reaches the paint layer");
    click(&screen, &mut world, seat);
    assert_eq!(world.state.center, CenterTab::Login, "the entry focuses it");

    let shapes = screen.shapes(&mut world, Vec::new());
    let column = rect_of(&shapes, "new conversation")
        .expect("the roster's own header marks the column")
        .right();
    let rows = rect_of(&shapes, "Login (bz browser sign-in)")
        .expect("the Login surface reaches the paint layer");
    assert!(
        rows.left() > column,
        "the provider rows must sit right of the roster column ({} vs {column})",
        rows.left()
    );
}

/// QUALITY G1/G4: **every peer is on screen at every supported size.** A strip
/// whose last tab lays off-window is a peer the operator cannot reach, which is
/// the reseat undone — so the row wraps rather than running off, and this
/// asserts it at the documented minimum (`src/main.rs` `min_inner_size`,
/// 420x320) as well as maximized. The rest of that window has its own open
/// defects (bl-b531, bl-9551); this asserts only the strip.
#[test]
fn no_tab_lays_off_window_at_either_capture() {
    for (w, h) in [(420.0, 320.0), (1600.0, 2400.0)] {
        let (litany, bl, bz) = (
            crate::cli_outbound::Cli::new("/yog-absent-litany"),
            crate::cli_outbound::Cli::new("/yog-absent-bl"),
            crate::cli_outbound::Cli::new("/yog-absent-bz"),
        );
        let mut world = world();
        let ws = world.ws.clone();
        world.model.focus_agent(&ws, "c-1");
        // With an answer landed the strip is at its widest — all four peers —
        // which is the case the minimum window has to hold.
        world.model.search("hello");
        world.searches();
        let ctx = egui::Context::default();
        let raw = crate::paint_probe::screen_sized(w, h);
        let mut out = None;
        for _ in 0..4 {
            out = Some(ctx.run(raw.clone(), |ctx| {
                crate::shell::render(ctx, &mut world.model, &mut world.state, &litany, &bl, &bz);
            }));
        }
        let shapes = out.expect("four frames ran").shapes;
        for tab in CenterTab::all() {
            let rect = rect_of(&shapes, tab.label())
                .unwrap_or_else(|| panic!("{tab:?} paints nothing at {w}x{h}"));
            assert!(
                rect.right() <= w + 1.0 && rect.bottom() <= h + 1.0 && rect.left() >= -1.0,
                "{tab:?} lays off a {w}x{h} window: {rect:?}"
            );
        }
    }
}
