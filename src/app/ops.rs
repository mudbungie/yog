//! The frame's two **writes** to the ops trail (DESIGN §4.2 as amended, §7.3,
//! §11, bl-c417) — the operator's ack and the clear verb.
//!
//! Every other `ops.jsonl` line is written by the action it records, from the
//! path that ran it ([`crate::actions::verbs`], the start flow, the §7.2 sweep).
//! These two are written by the frame because the *action is the operator's
//! gesture itself*: dismissing an alarm and ending a trail are things done to
//! the trail, not things the trail observed. They live here rather than in the
//! shell so they are covered — `src/shell/*` is excluded, and the discipline is
//! that everything a click calls is a tested unit.
//!
//! Both are one write plus one dirty mark: the state root is the ops file's
//! root (§7.1), so marking it is the same convergence path a dispatched verb
//! takes — the worker re-reads the tail on its next pass and the alarms
//! re-derive from it. Nothing about the ack or the clear is held in RAM; the
//! log stays the single source of truth (§4.2).

use super::AppModel;
use crate::opslog;

impl AppModel {
    /// **Acknowledge every alarm on screen** (§7.3, §11): append the §4.2 ack
    /// line, which becomes the global seen-watermark every failure-derived
    /// alarm reads past ([`opslog::since_ack`]). The banners go quiet and the
    /// chip drops its ⚠ / drift counts; the trail keeps every row it had, plus
    /// this one. A failure logged *after* the ack re-alarms.
    ///
    /// A write that cannot land is dropped, deliberately and exactly as the
    /// §7.2 drift append drops its own: the log is the surface an error would
    /// be reported on, so a state root yog cannot write has no second channel
    /// to complain through — and it is already the §7.2 staleness line's story.
    pub fn ack_failures(&mut self) {
        let _ = opslog::ack(&self.roots.yog_state, &self.clock.stamp());
        self.mark_dirty([self.roots.yog_state.clone()]);
    }

    /// **Start a fresh trail** (§4.2 as amended, §11): truncate `ops.jsonl` and
    /// log the clear as the new trail's first row, so the discard is itself an
    /// action with a record. The alarms clear as a consequence of the rows
    /// being gone, not by a second rule.
    ///
    /// Dropped-on-failure for [`ack_failures`](Self::ack_failures)' reason.
    pub fn clear_trail(&mut self) {
        let _ = opslog::clear(&self.roots.yog_state, &self.clock.stamp());
        self.mark_dirty([self.roots.yog_state.clone()]);
    }

    /// Whether anything on the trail is still alarming — the §11 predicate that
    /// decides whether the ops pane offers its Dismiss at all (a control that
    /// would write a line and change nothing is a control that should not be
    /// there). Derived from the same [`activity`](Self::activity) summary the
    /// chip paints, so the button and the chip's ichor appear together.
    pub fn has_alarms(&self) -> bool {
        let summary = self.activity();
        summary.errors > 0 || summary.drifts > 0
    }
}
