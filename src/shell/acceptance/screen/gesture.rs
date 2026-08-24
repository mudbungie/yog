//! **How a beat spells an input into the window** (§11): a key press and the
//! modifier plane it arrives on, a click paid out in the three frames egui
//! needs, the idle frame that says nothing, the Escape that puts the keyboard
//! back down, and the stand-in for a pointer at rest.
//!
//! Split from [`super`] at §12's budget on the same seam [`super::aim`] was cut
//! on, one door over: the driver there is how a frame is *run*, `aim` is how a
//! coordinate in one is *found*, and this is what a beat *says* to it.

use super::super::fixture::World;
use super::Screen;

impl Screen {
    /// Force every tooltip to paint, hover or no (`super::hover::live`'s drive
    /// and the paint-layer half both need it): the operator's hover is a
    /// pointer position no walk of the keyboard floor can be in two places for.
    pub(in crate::shell::acceptance) fn reveal(&self) {
        self.ctx.memory_mut(|m| m.set_everything_is_visible(true));
    }

    /// A frame with no input.
    pub(in crate::shell::acceptance) fn idle(&self, world: &mut World) -> bool {
        self.frame(world, Vec::new())
    }

    /// Escape — egui spends it surrendering text focus (§11), which is how a
    /// test puts the keyboard back down before asking whether an operation
    /// picks it up again.
    pub(in crate::shell::acceptance) fn release(&self, world: &mut World) {
        assert!(
            !self.frame(world, vec![press(egui::Key::Escape, egui::Modifiers::NONE)]),
            "Escape is the release gesture: the box must let go"
        );
    }
}

/// The modifier plane a frame's presses arrive on.
pub(super) fn modifiers_of(events: &[egui::Event]) -> egui::Modifiers {
    events
        .iter()
        .find_map(|e| match e {
            egui::Event::Key { modifiers, .. } => Some(*modifiers),
            _ => None,
        })
        .unwrap_or(egui::Modifiers::NONE)
}

/// The §11 Ctrl+Shift plane (⌘⇧ on macOS) — the `new workspace` combo.
pub(in crate::shell::acceptance) fn command_shift() -> egui::Modifiers {
    egui::Modifiers {
        shift: true,
        ..egui::Modifiers::COMMAND
    }
}

/// One full click at `pos`: move, press, release — three frames, because egui
/// hit-tests against the *previous* frame's widget rects, so a press in the
/// frame that first sees the pointer would test against nothing.
pub(in crate::shell::acceptance) fn click(screen: &Screen, world: &mut World, pos: egui::Pos2) {
    screen.frame(world, vec![egui::Event::PointerMoved(pos)]);
    for pressed in [true, false] {
        screen.frame(
            world,
            vec![egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: egui::Modifiers::NONE,
            }],
        );
    }
}

pub(in crate::shell::acceptance) fn press(
    key: egui::Key,
    modifiers: egui::Modifiers,
) -> egui::Event {
    egui::Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers,
    }
}
