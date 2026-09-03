//! The knobs of `ui.json` (DESIGN §4.1) — the view filters, the §11
//! transcript-density automatics, and the whole-UI zoom (text size).
//!
//! **`notify_unfocused` is not among them** (bl-09ef): the §6 escalation it
//! gated was derived here and announced by nobody, and the ruling put the
//! announcing at the **seat** — a desktop notification belongs on the box the
//! operator is looking at, which is wherever the seat is and never the engine
//! box. So the knob is a seat's own, stored where that seat stores its glass
//! facts; a stale `"notify_unfocused"` in an existing document round-trips as
//! an unknown key and means nothing.
//!
//! A child module so [`super`] stays inside its line budget (§12); privacy is
//! unaffected (a child sees its ancestor's private fields), and every knob
//! reads through the one forgiving [`UiState::flag`] accessor rather than
//! restating the `get`/`as_bool`/`unwrap_or` fold per key.

use super::{EXPAND_OTHERS, EXPAND_RESPONSES, UiState};
use serde_json::Value;

/// The whole-UI zoom key (§4.1): egui's scale factor, i.e. the text size.
const ZOOM: &str = "zoom";

/// The zoom domain — egui's own clamp (`egui::gui_zoom`), restated here
/// because yog owns the gesture (§11) and this document, not the egui
/// context, is the authority every frame derives the live factor from.
const ZOOM_MIN: f32 = 0.2;
const ZOOM_MAX: f32 = 5.0;

impl UiState {
    /// A boolean knob: the stored value, else `default` when absent or of the
    /// wrong type (the forgiving read, §4.1). One home for every flag below.
    fn flag(&self, key: &str, default: bool) -> bool {
        self.pane
            .root
            .get(key)
            .and_then(Value::as_bool)
            .unwrap_or(default)
    }

    fn set_flag(&mut self, key: &str, value: bool) {
        self.pane.root.insert(key.to_string(), Value::Bool(value));
        self.pane.save();
    }

    /// §11 transcript-density knob: does the conversation (delivered
    /// messages, model text, the live tail) arrive expanded? Default `true`
    /// (the operator's ruling).
    pub fn transcript_expand_responses(&self) -> bool {
        self.flag(EXPAND_RESPONSES, true)
    }

    pub fn set_transcript_expand_responses(&mut self, expand: bool) {
        self.set_flag(EXPAND_RESPONSES, expand);
    }

    /// §11 transcript-density knob: does the machinery (thinking, tool calls
    /// and results, raw bytes) arrive expanded? Default `false`.
    pub fn transcript_expand_others(&self) -> bool {
        self.flag(EXPAND_OTHERS, false)
    }

    pub fn set_transcript_expand_others(&mut self, expand: bool) {
        self.set_flag(EXPAND_OTHERS, expand);
    }

    /// The whole-UI zoom factor — the operator's text size (§4.1). `1.0` when
    /// absent or non-numeric (the forgiving read), always inside the domain,
    /// so a hand-edited `ui.json` can never open a window nobody can read.
    pub fn zoom(&self) -> f32 {
        self.pane
            .root
            .get(ZOOM)
            .and_then(Value::as_f64)
            .map_or(1.0, |z| (z as f32).clamp(ZOOM_MIN, ZOOM_MAX))
    }

    /// Persist the zoom factor, clamped to the domain and snapped to a
    /// hundredth — the snap keeps the document readable and makes the `f32`
    /// round-trip exact, so a relaunch reopens at the size it closed at.
    pub fn set_zoom(&mut self, zoom: f32) {
        let snapped = (f64::from(zoom.clamp(ZOOM_MIN, ZOOM_MAX)) * 100.0).round() / 100.0;
        self.pane
            .root
            .insert(ZOOM.to_string(), Value::from(snapped));
        self.pane.save();
    }
}
