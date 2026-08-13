//! The §7.3 no-response banner's **grace window** (bl-90bf): the render-layer
//! age gate that keeps a self-healing wound from flashing red on a healthy
//! send.
//!
//! The wound predicate ([`crate::steps_view::latest_step_no_response`]) joins
//! two observations that do **not** share a clock. Its disk half — the step's
//! `response.json`/`meta.json` — is re-read fresh every frame; its liveness
//! half is the agent state carried on the last published
//! [`Snapshot`](super::Snapshot), which only catches up when the §7.2 worker
//! re-derives. A driver taking its flock emits no fs event (§7.2 targeted
//! re-probe), so for the moments between the send and the poll that finds the
//! lock, a brand-new, genuinely-in-flight, genuinely-empty step reads as a
//! wound. That is a structural TOCTOU, not a bad classifier: the predicate is
//! provably right *at the instant it is evaluated on fresh inputs*, and the
//! cache is what is behind.
//!
//! The fix is time, not a third observation: **a wound must persist to be
//! believed.** A wound that clears inside the grace window
//! ([`Cadence::wound_grace`](super::Cadence::wound_grace)) never paints; one
//! that outlives it paints and stays. The predicate itself is untouched —
//! `steps_view::wound` remains pure and Clock-free (§5.1 #13, "nothing
//! stored") — and this gate is RAM the render layer owns, on the same injected
//! [`Clock`] seam the §7.2 schedule uses ([`super::dirty::Schedule`], whose
//! debounce this mirrors).
//!
//! **Only the banner is graced.** The Steps-tab row paints the same flag
//! (§11 Altitude-2) and is not gated: a table cell you opened a tab to read is
//! as fresh as every other cell in it, while the banner is an *alarm* — an
//! unrequested claim shouted at Altitude 1 — and an alarm that retracts itself
//! within a second teaches the operator to distrust it.

use crate::ui_state::Clock;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// The §11 Altitude-1 wound banner's grace state: which conversation the
/// window is open on, and when its wound was first seen.
///
/// One slot, not a table — the banner has one subject at a time (the focused
/// conversation), so a new subject is a new question and the old one's window
/// is not worth keeping. That is also the whole of its pruning: nothing
/// accumulates, and arriving at an already-wounded conversation re-opens the
/// window, which is right — the cached liveness the wound is judged against is
/// exactly as suspect on arrival as it is on a fresh send.
pub struct WoundGrace {
    clock: Arc<dyn Clock>,
    watching: Option<(PathBuf, String, Instant)>,
}

impl WoundGrace {
    /// Over the crate's one injected time source (§7.2) — the frame hands the
    /// same `Arc<dyn Clock>` it gave the model.
    pub fn new(clock: Arc<dyn Clock>) -> Self {
        Self {
            clock,
            watching: None,
        }
    }

    /// Does this frame paint the banner? `wounded` is the freshly-evaluated
    /// predicate; the answer is `true` only once the *same* conversation has
    /// read wounded for `grace`. A `false` closes the window outright, so a
    /// wound that heals leaves nothing behind and a later one starts over.
    ///
    /// `grace` is the live cadence's catch-up bound
    /// ([`Cadence::wound_grace`](super::Cadence::wound_grace), bl-3381) — the
    /// rendered snapshot carries it, so an operator who slows the sweeps
    /// lengthens the window with them and a healthy send still never flashes.
    /// The residual is the frame cadence, not this gate: the banner appears on
    /// the first frame after the window elapses, and frames are floored at the
    /// cheap sweep (I4) like every other rendered fact. A genuinely dead
    /// driver is therefore banner-ed late, never not at all.
    pub fn paints(
        &mut self,
        workspace: &Path,
        agent_id: &str,
        wounded: bool,
        grace: Duration,
    ) -> bool {
        if !wounded {
            self.watching = None;
            return false;
        }
        let now = self.clock.now();
        let since = match &self.watching {
            Some((ws, agent, since)) if ws == workspace && agent == agent_id => *since,
            _ => {
                self.watching = Some((workspace.to_path_buf(), agent_id.to_string(), now));
                now
            }
        };
        now.saturating_duration_since(since) >= grace
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::FakeClock;

    /// The default cadence's catch-up bound — what the frame passes when the
    /// operator has not re-tuned (`Cadence::wound_grace`).
    fn window() -> Duration {
        super::super::Cadence::default().wound_grace()
    }

    const WS: &str = "/w";
    const AGENT: &str = "c-1";

    fn ws() -> PathBuf {
        PathBuf::from(WS)
    }

    /// The bug: a healthy send whose driver holds the lock the cache has not
    /// seen yet reads wounded for under a second, then clears. It must never
    /// have reached the screen.
    #[test]
    fn a_wound_that_clears_inside_the_window_never_paints() {
        let clock = FakeClock::new();
        let mut grace = WoundGrace::new(clock.arc());
        assert!(
            !grace.paints(&ws(), AGENT, true, window()),
            "not on first sight"
        );
        clock.advance(Duration::from_millis(900));
        assert!(
            !grace.paints(&ws(), AGENT, true, window()),
            "still inside the window"
        );
        // The snapshot catches up: the driver was alive all along.
        assert!(!grace.paints(&ws(), AGENT, false, window()));
        // And the healed conversation starts a fresh window, not a half-spent
        // one — else the next transient flashes immediately.
        clock.advance(window());
        assert!(
            !grace.paints(&ws(), AGENT, true, window()),
            "window re-opened"
        );
    }

    /// The honest wound still arrives — delayed by the window, never dropped.
    #[test]
    fn a_wound_that_outlives_the_window_paints_and_stays() {
        let clock = FakeClock::new();
        let mut grace = WoundGrace::new(clock.arc());
        assert!(!grace.paints(&ws(), AGENT, true, window()));
        clock.advance(window());
        assert!(
            grace.paints(&ws(), AGENT, true, window()),
            "the window elapsed"
        );
        assert!(
            grace.paints(&ws(), AGENT, true, window()),
            "and it stays painted"
        );
    }

    /// The window belongs to a conversation, not to the gate: switching the
    /// focused agent (or workspace) asks the question again.
    #[test]
    fn a_new_subject_opens_its_own_window() {
        let clock = FakeClock::new();
        let mut grace = WoundGrace::new(clock.arc());
        assert!(!grace.paints(&ws(), AGENT, true, window()));
        clock.advance(window());
        assert!(
            !grace.paints(&ws(), "c-2", true, window()),
            "another agent, own window"
        );
        assert!(
            !grace.paints(&PathBuf::from("/other"), "c-2", true, window()),
            "another workspace, own window"
        );
        clock.advance(window());
        assert!(grace.paints(&PathBuf::from("/other"), "c-2", true, window()));
    }

    /// A re-tuned cadence lengthens the window it is judged by (bl-3381).
    #[test]
    fn a_longer_grace_holds_the_banner_longer() {
        let clock = FakeClock::new();
        let mut grace = WoundGrace::new(clock.arc());
        let long = Duration::from_secs(30);
        assert!(!grace.paints(&ws(), AGENT, true, long));
        clock.advance(window());
        assert!(
            !grace.paints(&ws(), AGENT, true, long),
            "the default window is not this gate's"
        );
        clock.advance(long);
        assert!(grace.paints(&ws(), AGENT, true, long));
    }
}
