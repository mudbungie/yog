//! **The seat that can grow may not eat its pane** (QUALITY G4): the §9.4
//! picker expands inline at the model line, and an accessory that takes the
//! whole conversation pane is the overlap defect bl-9551 filed. Split from
//! [`super`] at §12's budget on the seam between the seat's **contents** —
//! which facts belong in it, which must not be left in the header above — and
//! the seat's **size**, which is a bound rather than a fact and is held at both
//! ends of the supported window range.

use super::super::fixture::world;
use super::super::input;
use super::frame::Window;

/// QUALITY G4, held against the seat that can grow: the §9.4 picker expands
/// inline at the model line, and an accessory that eats its own pane is the
/// overlap defect bl-9551 filed. The region is capped at half the pane and
/// scrolls past it, so an open picker cannot push the transcript off screen.
#[test]
fn an_expanded_picker_cannot_grow_the_seat_past_half_the_pane() {
    let mut world = world();
    let ws = world.ws.clone();
    world.model.focus_agent(&ws, "c-1");
    let win = Window::new();

    let collapsed = {
        win.settled(&mut world);
        win.panel("conversation-settings").height()
    };
    world.state.wall.picker.open = true;
    win.settled(&mut world);
    let expanded = win.panel("conversation-settings");
    assert!(
        expanded.height() > collapsed,
        "the picker does open inline at the line: {collapsed} → {}",
        expanded.height()
    );
    let window = input().screen_rect.expect("the probe sizes the screen");
    assert!(
        expanded.height() <= window.height() / 2.0,
        "and the seat stays under half the pane: {} of {}",
        expanded.height(),
        window.height()
    );
}

/// G4 at the documented minimum window (`src/main.rs` `min_inner_size`,
/// 420x320): the cap is a *share* of the pane, not a pixel count, so the seat
/// that can expand a picker inline is still bounded where there is least room
/// to spare. The rest of that window has its own open defects (bl-b531,
/// bl-9551); this asserts only the accessory this ball added.
#[test]
fn the_seat_is_bounded_at_the_smallest_supported_window_too() {
    let mut world = world();
    let ws = world.ws.clone();
    world.model.focus_agent(&ws, "c-1");
    let win = Window::new();
    // One frame first: the picker is the wall's RAM (bl-5894), so the flag goes
    // on the focused sphere's own picker rather than the launch bundle's.
    win.settled_on(&mut world, crate::paint_probe::screen_sized(420.0, 320.0));
    world.state.wall.picker.open = true;
    win.settled_on(&mut world, crate::paint_probe::screen_sized(420.0, 320.0));
    let seat = win.panel("conversation-settings");
    assert!(
        seat.height() <= 320.0 / 2.0,
        "an open picker may not take more than half the pane: {}",
        seat.height()
    );
}
