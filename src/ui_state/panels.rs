//! The `panels` object of `ui.json` (DESIGN §4.1, §11): the sizes the operator
//! dragged the resizable panel boundaries to.
//!
//! One key per *class* of fact, not one per panel — the document grows a
//! `panels` object whose members are named by [`Panel`], so adding a draggable
//! boundary is one enum variant, never a new top-level key plus a constant plus
//! a branch. The variant is that panel's one home for its key, its opening size
//! and its floor.
//!
//! A child module so [`super`] stays inside its line budget (§12), on the same
//! terms as [`super::knobs`] and [`super::fields`]: privacy is unaffected (a
//! child sees its ancestor's private fields), and the parent keeps only the
//! file mechanics — forgiving load, echo hash, atomic write.

use super::{UiState, descend};
use serde_json::Value;

/// The `ui.json` key holding every dragged panel size.
const PANELS: &str = "panels";

/// A panel whose boundary the operator may drag (§11). Sizes are in **logical
/// points**, so they are independent of the §4.1 `zoom`: a text-size change
/// rescales what a panel holds, never how wide the operator made it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Panel {
    /// The conversation-list side panel — its **width**.
    Conversations,
    /// The expanded activity trail, the §11 demoted ops pane — its **height**.
    /// The collapsed chip is not this panel (see `shell`): a chip's height is
    /// its content's, and only the trail is sized by the operator.
    ActivityTrail,
    /// The editable start-goal composer (§8.1) — its **height**.
    StartGoal,
}

impl Panel {
    /// This panel's member name inside the `panels` object.
    fn key(self) -> &'static str {
        match self {
            Panel::Conversations => "conversations",
            Panel::ActivityTrail => "activity_trail",
            Panel::StartGoal => "start_goal",
        }
    }

    /// The size an operator who has never dragged this boundary gets. The
    /// conversation column is bounded so altitude-1 (the center) has room at
    /// the default window; the two bottom panes open on a few rows of content.
    pub fn default_size(self) -> f32 {
        match self {
            Panel::Conversations => 260.0,
            Panel::ActivityTrail => 200.0,
            Panel::StartGoal => 240.0,
        }
    }

    /// How small the boundary may be dragged. The conversation column's floor
    /// is a grabbable sliver rather than egui's stock 96 pt, so the roster can
    /// be put away entirely (bl-9669); a bottom pane's floor is one row plus
    /// its grip, below which there is nothing left to read.
    pub fn min_size(self) -> f32 {
        match self {
            Panel::Conversations => 24.0,
            Panel::ActivityTrail | Panel::StartGoal => 48.0,
        }
    }

    /// How large this panel may grow, given the window's extent along the
    /// panel's **own** axis — width for the side panel, height for the two
    /// bottom ones. Never below the floor: a window too small to hold twice a
    /// sliver still owes the operator a boundary to grab.
    /// The share itself lives in [`crate::layout`] (§11 rule 5 as amended,
    /// bl-9551) — one home, so a boundary the operator drags and an accessory
    /// the pane docks cannot disagree about what half means. The floor
    /// ([`Panel::min_size`]) stays in points for the opposite reason to the
    /// ceiling's share: a grabbable sliver is a physical size.
    pub fn max_size(self, window: f32) -> f32 {
        crate::layout::panel_ceiling(window).max(self.min_size())
    }

    /// `size` folded into this panel's floor…ceiling at `window` — **the one
    /// home of the clamp**. The size a panel opens at, the size a released
    /// boundary stores, and the widget's own `max_width`/`max_height` all come
    /// through here, so no reading of the document, no window resize and no
    /// runaway row can produce a panel that eats the window.
    pub fn clamp(self, size: f32, window: f32) -> f32 {
        size.max(self.min_size()).min(self.max_size(window))
    }
}

impl UiState {
    /// The stored size of `panel`, or `None` when the operator has never
    /// dragged it — absent, non-numeric, or a `panels` that is not an object
    /// all read as never-dragged (the forgiving read, §4.1). The default and
    /// the floor are the model's fold, not this accessor's: one home each.
    pub fn panel_size(&self, panel: Panel) -> Option<f32> {
        self.root
            .get(PANELS)
            .and_then(Value::as_object)
            .and_then(|m| m.get(panel.key()))
            .and_then(Value::as_f64)
            .map(|size| size as f32)
    }

    /// Record where a boundary came to rest, snapped to a whole point — layout
    /// is measured in points and sub-point precision is noise that would make
    /// the document's bytes churn without moving anything the eye can see.
    pub fn set_panel_size(&mut self, panel: Panel, size: f32) {
        let snapped = f64::from(size).round();
        let slot = descend(&mut self.root, PANELS.to_string());
        slot.insert(panel.key().to_string(), Value::from(snapped));
        self.save();
    }
}

#[cfg(test)]
mod tests;
