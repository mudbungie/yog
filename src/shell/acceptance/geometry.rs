//! Panel geometry (§11, bl-9669): the conversation panel's width is the
//! operator's, not its content's — a measured regression, split from the
//! surface smoke test for §12's line budget. Its sibling `name_column` is the
//! geometry regression one altitude in, on the row rather than the panel.

use super::super::render;
use super::fixture::world_titled;
use super::input;
use crate::cli_outbound::Cli;
use crate::ui_state::Panel;

/// The conversation panel's width belongs to the operator, not to its content
/// (§11, bl-9669). Two properties, both regressions of a measured defect: a
/// long conversation title cannot widen the panel (a row that overflows
/// widens the panel's rect, which egui *stores as the panel width*, so the
/// column used to creep wider every single frame, unbounded), and the panel
/// can be dragged well below egui's own 96 pt floor.
#[test]
fn a_long_title_cannot_widen_the_conversation_panel_and_it_shrinks_to_a_sliver() {
    let (lernie, bl, bz) = (Cli::new("lernie"), Cli::new("bl"), Cli::new("bz"));
    let mut world = world_titled(
        "a very long conversation title that runs on and on and would once have \
         ratcheted the left column wider on every frame it was painted",
    );
    let ctx = egui::Context::default();
    let panel = egui::Id::new("conversations");
    let mut frame = || {
        let _ = ctx.run(input(), |ctx| {
            render(ctx, &mut world.model, &mut world.state, &lernie, &bl, &bz);
        });
        egui::containers::panel::PanelState::load(&ctx, panel)
            .expect("the panel stores its rect")
            .rect
            .width()
    };
    for _ in 0..2 {
        frame();
    }
    let settled = frame();
    for _ in 0..6 {
        let width = frame();
        assert!(
            (width - settled).abs() < 1.0,
            "the panel must not creep: {settled} → {width}"
        );
    }
    assert!(
        settled <= Panel::Conversations.default_size() + 1.0,
        "a long title must not widen the panel past its default: {settled}"
    );

    // Dragged to nothing (the stored rect a splitter drag to x=0 leaves), the
    // panel settles on a sliver — below egui's stock 96 pt floor.
    ctx.data_mut(|d| {
        d.insert_persisted(
            panel,
            egui::containers::panel::PanelState {
                rect: egui::Rect::from_min_size(
                    egui::Pos2::new(0.0, 25.0),
                    egui::vec2(1.0, 2000.0),
                ),
            },
        );
    });
    for _ in 0..2 {
        frame();
    }
    let shrunk = frame();
    assert!(
        shrunk < 96.0,
        "the panel must shrink past egui's stock floor: {shrunk}"
    );
    for _ in 0..4 {
        let width = frame();
        assert!(
            (width - shrunk).abs() < 1.0,
            "and stay there: {shrunk} → {width}"
        );
    }
}

/// A panel that has already grown cannot **stay** grown (§11, bl-ac3d).
///
/// egui keeps a panel's content rect as its width, and one row that lays past
/// the edge writes a width nobody dragged — the balls section's new-ball
/// header did exactly that, because `CollapsingHeader` lays its text
/// `TextWrapMode::Extend` whatever the panel's own wrap mode says, so the
/// column opened at the width of an absolute project path (~690 pt of a
/// 1150 pt window, measured) and never came back. The panel now carries a
/// ceiling in its `width_range`, which egui re-applies to the stored rect on
/// **every** frame: whatever put a runaway width there, the next frame is
/// already under half the window, and it settles there rather than creeping.
#[test]
fn a_panel_grown_past_its_ceiling_is_back_under_it_on_the_next_frame() {
    let (lernie, bl, bz) = (Cli::new("lernie"), Cli::new("bl"), Cli::new("bz"));
    let mut world = world_titled("hello");
    let ctx = egui::Context::default();
    let panel = egui::Id::new("conversations");
    let mut frame = || {
        let _ = ctx.run(input(), |ctx| {
            render(ctx, &mut world.model, &mut world.state, &lernie, &bl, &bz);
        });
        egui::containers::panel::PanelState::load(&ctx, panel)
            .expect("the panel stores its rect")
            .rect
            .width()
    };
    for _ in 0..2 {
        frame();
    }
    // The rect a runaway row leaves behind: far past the window's own width.
    ctx.data_mut(|d| {
        d.insert_persisted(
            panel,
            egui::containers::panel::PanelState {
                rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(3000.0, 2000.0)),
            },
        );
    });
    let window = input()
        .screen_rect
        .expect("the probe sizes the screen")
        .width();
    let ceiling = Panel::Conversations.max_size(window);
    let recovered = frame();
    assert!(
        recovered <= ceiling + 1.0,
        "the panel must fold back under its ceiling: {recovered} > {ceiling}"
    );
    for _ in 0..4 {
        let width = frame();
        assert!(
            (width - recovered).abs() < 1.0,
            "and stay there: {recovered} → {width}"
        );
    }
}

/// The regression bl-9ad4 was filed for, end to end through the real window:
/// a panel boundary the operator drags **stays** where it was dropped, and the
/// size reaches `ui.json` so the next launch opens there (§4.1 `panels`).
///
/// Both halves were broken in different ways. The conversation column held its
/// drag for the session and forgot it at exit. The activity trail could not be
/// dragged at all — and once resizable, egui re-opens a panel at the rect its
/// *content* last occupied, so a trail with a few rows in it collapsed to its
/// 48 pt floor on the very next frame; the shell pins the content to the panel
/// (`pin_to_panel`), and this asserts the pin holds against an EMPTY trail,
/// the worst case.
#[test]
fn a_dragged_boundary_stays_dropped_and_reaches_ui_json() {
    let (lernie, bl, bz) = (Cli::new("lernie"), Cli::new("bl"), Cli::new("bz"));
    let mut world = world_titled("hello");
    world.state.activity_open = true;
    let ctx = egui::Context::default();
    let ui_json = world.model.state_root().join("ui.json");
    let mut frame = |id: &str| {
        let _ = ctx.run(input(), |ctx| {
            render(ctx, &mut world.model, &mut world.state, &lernie, &bl, &bz);
        });
        let rect = egui::containers::panel::PanelState::load(&ctx, egui::Id::new(id))
            .expect("the panel stores its rect")
            .rect;
        (rect.width(), rect.height())
    };

    // The trail opens at its default and holds it though it has nothing in it.
    for _ in 0..3 {
        frame("activity-trail");
    }
    let opened = frame("activity-trail").1;
    assert!(
        (opened - Panel::ActivityTrail.default_size()).abs() < 1.0,
        "an empty trail must hold its opening height, not collapse: {opened}"
    );

    // Drag both boundaries — the rect a released splitter leaves behind.
    for (id, size) in [("conversations", 420.0), ("activity-trail", 330.0)] {
        ctx.data_mut(|d| {
            d.insert_persisted(
                egui::Id::new(id),
                egui::containers::panel::PanelState {
                    rect: egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(size, size)),
                },
            );
        });
    }
    for _ in 0..3 {
        frame("conversations");
    }
    let (width, _) = frame("conversations");
    let (_, height) = frame("activity-trail");
    assert!(
        (width - 420.0).abs() < 1.0,
        "the column stays dropped: {width}"
    );
    assert!(
        (height - 330.0).abs() < 1.0,
        "the trail stays dropped: {height}"
    );

    // And the next launch opens there: the document, read back off disk.
    let reopened = crate::ui_state::UiState::open(ui_json);
    assert_eq!(reopened.panel_size(Panel::Conversations), Some(420.0));
    assert_eq!(reopened.panel_size(Panel::ActivityTrail), Some(330.0));
}
