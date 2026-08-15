//! **One frame's model duty** (§7.2): take the newest derivation, adopt what
//! changed under it, and settle the two off-frame hand-offs — the wire's read
//! path and, since bl-4841, its act path.
//!
//! Split out of [`app`](super)'s root at §12's budget. It is the whole of what
//! a frame asks the model to *do*: everything else the model offers is a read,
//! and everything it derives belongs to the worker.

use super::echo;
use crate::state::latest_snapshot;
use std::sync::Arc;

impl crate::AppModel {
    /// One frame's whole model duty (§7.2): take the newest completed snapshot
    /// if the worker published one, adopt an externally-changed `ui.json` from
    /// it, and hold the §6 acknowledgement. Returns whether the render source
    /// moved.
    ///
    /// It never blocks on a derivation — the only wait is the pointer swap in
    /// [`crate::state`] — and it never *starts* one. A frame that arrives
    /// mid-pass renders the previous snapshot, which is the whole point.
    pub fn refresh(&mut self) -> bool {
        let latest = latest_snapshot(&self.cell);
        let landed = !Arc::ptr_eq(&self.derived, &latest);
        if landed {
            self.derived = latest;
            self.adopt_ui();
        }
        // A fired start focuses the conversation it started (§3.4), the first
        // frame whose roster carries its root, and the pending echo it carries
        // retires on the same predicate (§7.2). Over the roster, not the
        // pointer swap — so it is asked every frame, and free with nothing
        // pending.
        self.adopt_started();
        // Tell the follower which conversation is on screen and take whatever
        // it has folded since the last frame (§7.2 live tail). Both are one
        // lock and one compare, which is what a frame is allowed to cost.
        crate::state::follow(&self.tail, self.followed_subject());
        let followed = crate::state::taken_tail(&self.tail);
        // The one fold of derivation + the non-derived facts (§7.2), run only
        // when one of its inputs moved so the rendered `Arc` stays stable under
        // `SnapMemo`.
        if landed || self.started != self.folded || followed != self.followed {
            self.folded = self.started.clone();
            self.followed = followed;
            self.snap = echo::compose(
                &self.derived,
                self.started.as_ref(),
                self.followed.as_deref(),
            );
        }
        // The wire read path's one frame duty (REMOTE §1.2, bl-ae05): take the
        // answers the asker landed and tell it what this window is standing on
        // if that changed. Two channel drains and one set compare — no lock, no
        // dial, and nothing here can wait on a socket.
        self.wire.settle();
        // The act path's own frame duty (REMOTE §9.8, bl-4841): take the
        // receipts the poster landed and mark each act's root dirty now that
        // the engine is done with it — the aftermath a dispatched verb used to
        // run the instant it returned, run at the moment it actually happened.
        self.settle_acts();
        // The §6 ack is a state, not a gesture (bl-aa1f): re-stamp the focused
        // agent's evidence every frame, so a signal that landed on the
        // conversation the operator is reading is already seen. Free — §4.1
        // elides a write whose bytes are unchanged.
        self.ack_focused();
        landed
    }

    /// Adopt an external `ui.json` change the worker read for us (§4.1, I5):
    /// unless it is our own echo (content-hash match), wholesale-adopt it — the
    /// converging seen/pins path both instances share.
    fn adopt_ui(&mut self) {
        if let Some(bytes) = self.derived.ui_bytes.clone()
            && !self.ui.is_echo(&bytes)
        {
            self.ui.adopt(&bytes);
        }
    }
}
