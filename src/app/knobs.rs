//! The §11 transcript-density knobs and the whole-UI zoom on [`AppModel`]
//! (DESIGN §4.1, §11).
//!
//! Two booleans decide the *auto-state* a transcript row arrives in — the
//! conversation expanded, the machinery around it contracted — and both are
//! knobs rather than constants so the policy is config, not code
//! (severability: removing a default deletes a `ui.json` key, not a code
//! path). They live in `ui.json` because that is yog's one durable UI-state
//! artifact; no yog config file exists and none is invented for them.
//!
//! A child module of `app` purely for the 300-line budget (§12), like
//! [`super::balls`] and [`super::focus`].

use super::AppModel;
use crate::keymap::ZoomStep;
use crate::transcript::AutoExpand;

/// One zoom gesture's stride — egui's own 0.1 (`egui::gui_zoom`), so yog's
/// bindings feel exactly like the browser's they replace.
const ZOOM_STEP: f32 = 0.1;

impl AppModel {
    /// The transcript's auto-expand pair, as the view-model consumes it.
    pub fn transcript_auto_expand(&self) -> AutoExpand {
        AutoExpand {
            responses: self.ui.transcript_expand_responses(),
            others: self.ui.transcript_expand_others(),
        }
    }

    /// Set whether the conversation — delivered messages, model text, the
    /// live tail — arrives expanded (§11).
    pub fn set_transcript_expand_responses(&mut self, expand: bool) {
        self.ui.set_transcript_expand_responses(expand);
    }

    /// Set whether the machinery — thinking, tool calls and results, raw
    /// bytes — arrives expanded (§11).
    pub fn set_transcript_expand_others(&mut self, expand: bool) {
        self.ui.set_transcript_expand_others(expand);
    }

    /// The §6 desktop-escalation knob (§4.1 `notify_unfocused`, bl-e160) — may
    /// an unfocused window tell the desktop that something new needs the
    /// operator? Read per frame like every other knob; nothing caches it.
    pub fn notify_unfocused(&self) -> bool {
        self.ui.notify_unfocused()
    }

    /// The whole-UI zoom factor — the operator's text size (§4.1 `zoom`). The
    /// one authority: the shell derives egui's live factor from this every
    /// frame, and egui's own keyboard zoom is off (`theme::apply`), so no
    /// second copy of the fact exists to drift or to be lost at exit.
    pub fn zoom(&self) -> f32 {
        self.ui.zoom()
    }

    /// Apply a §11 zoom gesture (Ctrl+`+` / Ctrl+`-` / Ctrl+`0`): one step
    /// either way, or back to 1.0. Clamped, snapped and written through by
    /// `ui_state`, so the size is on disk before the keypress returns.
    pub fn zoom_nudge(&mut self, step: ZoomStep) {
        let zoom = match step {
            ZoomStep::In => self.ui.zoom() + ZOOM_STEP,
            ZoomStep::Out => self.ui.zoom() - ZOOM_STEP,
            ZoomStep::Reset => 1.0,
        };
        self.ui.set_zoom(zoom);
    }
}
