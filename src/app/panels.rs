//! The resizable panel sizes on [`AppModel`] (DESIGN §4.1 `panels`, §11).
//!
//! Two calls carry the whole feature: the size a panel **opens** at, and the
//! size a released boundary **settles** on. The document is the authority and
//! the egui context is its projection — the same ruling the §4.1 `zoom` is held
//! under: the shell hands each panel its size every frame and reports back what
//! egui measured, so no second copy of the fact exists to drift or to be lost
//! at exit.
//!
//! A child module of `app` purely for the 300-line budget (§12), like
//! [`super::knobs`] and [`super::focus`].

use super::AppModel;
use crate::ui_state::Panel;

/// The smallest boundary move worth recording: one logical point. Below it a
/// change is either the operator's hand shaking or a layout rounding wobble,
/// and writing it would churn the document every frame a window sat still.
const SETTLE_EPSILON: f32 = 1.0;

impl AppModel {
    /// The size to open `panel` at in a window of extent `window` (along the
    /// panel's own axis): what the operator last dragged it to, else the
    /// panel's default — folded into the panel's floor…ceiling
    /// ([`Panel::clamp`]), so neither a hand-edited `ui.json` nor a width
    /// stored on a larger screen can open a panel with no boundary to grab or
    /// no centre beside it. The fold is on **read**, never on the stored fact:
    /// a wide window still opens at the operator's own width (bl-ac3d).
    pub fn panel_size(&self, panel: Panel, window: f32) -> f32 {
        panel.clamp(
            self.ui
                .panel_size(panel)
                .unwrap_or_else(|| panel.default_size()),
            window,
        )
    }

    /// Record where a panel boundary came to rest.
    ///
    /// `settled` is the gesture boundary — `false` while the pointer is still
    /// down — so one drag is one write, at its rest, rather than one per frame
    /// of it. A boundary that has not actually moved writes nothing at all,
    /// which is what keeps a still window from touching the disk 60 times a
    /// second: the same discipline as the §4.1 held arrow key, which writes
    /// only on the steps that change something.
    ///
    /// The reported size is what egui *measured*, which is the panel's content
    /// rect — so a row that overflowed reports a width nobody dragged. Folding
    /// it through [`Panel::clamp`] first bounds that at the ceiling: the worst
    /// a runaway row can now do to the document is half the window (bl-ac3d).
    pub fn settle_panel_size(&mut self, panel: Panel, size: f32, window: f32, settled: bool) {
        let size = panel.clamp(size, window);
        if !settled || (size - self.panel_size(panel, window)).abs() < SETTLE_EPSILON {
            return;
        }
        self.ui.set_panel_size(panel, size);
    }
}
