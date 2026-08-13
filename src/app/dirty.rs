//! Debounce + sweep scheduling (DESIGN §7.2, §15 Y6): the clock-gated timing
//! decisions the frame consults each tick.
//!
//! Two rhythms, one injected clock:
//!
//! - **Debounce** — a dirty root opens a 100 ms coalescing window
//!   ([`DEBOUNCE`]); [`Schedule::due`] yields it only once the window elapses,
//!   collapsing a streaming-append storm to ≤10 rebuilds/s of a workspace
//!   (§7.2). Rebuild is always correct, so coalescing only ever delays, never
//!   drops.
//! - **Sweeps** — [`Schedule::sweep`] is the pure `should_run(now, last)`
//!   decision behind the 2 s cheap sweep and the 15 s full sweep (§7.2). The
//!   full sweep supersedes the cheap one. The *effects* (enumerate + reconcile
//!   the [`WatchSet`](crate::watch::WatchSet), re-probe liveness, mark roots
//!   dirty) live in [`AppModel`](super::AppModel) — this module is timing only,
//!   so every branch is testable with an injected clock and no sleeps.
//!
//! Time is [`crate::ui_state::Clock`], reused verbatim (§7.2: "the same
//! injection pattern") — Y6 mints no second clock trait.

use crate::ui_state::Clock;
use crate::watch::Mark;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Default coalescing window: a dirty root re-derives at most once per 100 ms
/// (§7.2). Since bl-3381 the *live* value is [`Cadence`](super::Cadence) —
/// these consts are what an absent `cadence.yaml` means.
pub const DEBOUNCE: Duration = Duration::from_millis(100);
/// Default cheap sweep cadence: enumerations + reconcile + targeted liveness
/// (§7.2). Also the frame's `request_repaint_after` poll floor (I4).
pub const CHEAP_SWEEP: Duration = Duration::from_secs(2);
/// Default full sweep cadence: re-derive everything, bounding staleness (§7.2).
pub const FULL_SWEEP: Duration = Duration::from_secs(15);

/// Which periodic sweep a tick owes (§7.2). `Full` implies (and supersedes)
/// `Cheap`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sweep {
    None,
    Cheap,
    Full,
}

/// Debounce windows + sweep deadlines over one injected clock. The clock is a
/// trait object (`Arc<dyn Clock>`, cold-path virtual dispatch) so the schedule
/// carries no `Clock` generic; the `Arc` shares one time source between the
/// caller that injected it and this schedule (§7.2).
pub struct Schedule {
    clock: Arc<dyn Clock>,
    /// The live periods (bl-3381): the operator's `cadence.yaml` tuning, or the
    /// defaults. The worker re-reads the file on its own announced change and
    /// [`set_cadence`](Self::set_cadence)s the schedule — never per tick.
    cadence: super::Cadence,
    /// Root → (release deadline, why it was marked). The [`Mark`] rides through
    /// the coalescing window so the re-derivation that consumes it knows whether
    /// anything actually announced the change (§7.2 instrumentation).
    pending: HashMap<PathBuf, (Instant, Mark)>,
    last_cheap: Instant,
    last_full: Instant,
}

impl Schedule {
    pub fn new(clock: Arc<dyn Clock>, cadence: super::Cadence) -> Self {
        let now = clock.now();
        Self {
            clock,
            cadence,
            pending: HashMap::new(),
            last_cheap: now,
            last_full: now,
        }
    }

    /// Adopt new periods (bl-3381). Forward-only on purpose: an open debounce
    /// window keeps the deadline it was promised, and the sweep baselines
    /// stand — the next `sweep()` measures the new period from the last fire,
    /// which is the level-triggered reading (a re-tune converges, VISION §4.3).
    pub fn set_cadence(&mut self, cadence: super::Cadence) {
        self.cadence = cadence;
    }

    /// Open a coalescing window for each root, stamped with why it was marked.
    /// A root already pending keeps its earliest deadline, so repeated marks
    /// within a window still fire once, and keeps the **strongest** mark, so a
    /// sweep's blanket mark never masks a watcher announcement that lands in the
    /// same window (nor the reverse).
    pub(crate) fn mark<I: IntoIterator<Item = (PathBuf, Mark)>>(&mut self, roots: I) {
        let deadline = self.clock.now() + self.cadence.debounce;
        for (root, mark) in roots {
            let slot = self.pending.entry(root).or_insert((deadline, mark));
            slot.1 = slot.1.max(mark);
        }
    }

    /// Roots whose window has elapsed, each with its mark — ready to re-derive
    /// now. Removed from the pending set; a later change reopens a fresh window.
    pub fn due(&mut self) -> Vec<(PathBuf, Mark)> {
        let mut ready = Vec::new();
        let now = self.clock.now();
        for (root, &(deadline, mark)) in &self.pending {
            if now >= deadline {
                ready.push((root.clone(), mark));
            }
        }
        for (root, _) in &ready {
            self.pending.remove(root);
        }
        ready
    }

    /// The sweep this tick owes, resetting the fired deadline(s). A full sweep
    /// also resets the cheap deadline (it did the cheap work and more).
    pub fn sweep(&mut self) -> Sweep {
        let now = self.clock.now();
        if elapsed(now, self.last_full, self.cadence.full_sweep) {
            self.last_full = now;
            self.last_cheap = now;
            Sweep::Full
        } else if elapsed(now, self.last_cheap, self.cadence.cheap_sweep) {
            self.last_cheap = now;
            Sweep::Cheap
        } else {
            Sweep::None
        }
    }
}

/// `now - last >= period`, saturating (a non-monotonic injected clock can't
/// underflow).
fn elapsed(now: Instant, last: Instant, period: Duration) -> bool {
    now.saturating_duration_since(last) >= period
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::FakeClock;

    fn root(s: &str) -> PathBuf {
        PathBuf::from(s)
    }

    #[test]
    fn debounce_holds_a_root_until_the_window_elapses() {
        let clock = FakeClock::new();
        let mut sched = Schedule::new(clock.arc(), super::super::Cadence::default());
        sched.mark([(root("/w"), Mark::Watch)]);
        assert!(sched.due().is_empty(), "held during the window");
        clock.advance(DEBOUNCE);
        assert_eq!(
            sched.due(),
            vec![(root("/w"), Mark::Watch)],
            "released after the window"
        );
        assert!(sched.due().is_empty(), "consumed — not re-emitted");
    }

    #[test]
    fn debounce_coalesces_repeated_marks_to_one_release() {
        let clock = FakeClock::new();
        let mut sched = Schedule::new(clock.arc(), super::super::Cadence::default());
        // A storm within the window keeps the earliest deadline.
        sched.mark([(root("/w"), Mark::Watch)]);
        clock.advance(Duration::from_millis(40));
        sched.mark([(root("/w"), Mark::Watch)]);
        clock.advance(Duration::from_millis(60)); // 100 ms since first mark
        assert_eq!(sched.due(), vec![(root("/w"), Mark::Watch)]);
        // A fresh mark reopens a new window.
        sched.mark([(root("/w"), Mark::Watch)]);
        assert!(sched.due().is_empty());
    }

    #[test]
    fn the_strongest_mark_wins_inside_one_window() {
        let clock = FakeClock::new();
        let mut sched = Schedule::new(clock.arc(), super::super::Cadence::default());
        // The 15 s sweep marks blindly; the watcher's announcement lands in the
        // same window and must win, else a caught change reads as a dropped one.
        sched.mark([(root("/w"), Mark::Sweep)]);
        sched.mark([(root("/w"), Mark::Watch)]);
        clock.advance(DEBOUNCE);
        assert_eq!(sched.due(), vec![(root("/w"), Mark::Watch)]);
        // And the reverse order: a later blanket mark cannot demote it.
        sched.mark([(root("/w"), Mark::Desync)]);
        sched.mark([(root("/w"), Mark::Sweep)]);
        clock.advance(DEBOUNCE);
        assert_eq!(sched.due(), vec![(root("/w"), Mark::Desync)]);
    }

    #[test]
    fn a_re_tuned_cadence_drives_the_next_windows_and_sweeps() {
        let clock = FakeClock::new();
        let mut sched = Schedule::new(clock.arc(), super::super::Cadence::default());
        sched.set_cadence(super::super::Cadence {
            debounce: Duration::from_millis(500),
            cheap_sweep: Duration::from_secs(10),
            full_sweep: Duration::from_mins(1),
        });
        // The old debounce no longer releases; the new one does.
        sched.mark([(root("/w"), Mark::Watch)]);
        clock.advance(DEBOUNCE);
        assert!(
            sched.due().is_empty(),
            "the shipped 100 ms is not the window"
        );
        clock.advance(Duration::from_millis(400));
        assert_eq!(sched.due(), vec![(root("/w"), Mark::Watch)]);
        // And the sweeps measure the new periods from the same baselines.
        clock.advance(CHEAP_SWEEP);
        assert_eq!(sched.sweep(), Sweep::None, "2 s is no longer a cheap tick");
        clock.advance(Duration::from_secs(10));
        assert_eq!(sched.sweep(), Sweep::Cheap);
        clock.advance(Duration::from_mins(1));
        assert_eq!(sched.sweep(), Sweep::Full);
    }

    #[test]
    fn sweep_none_then_cheap_then_full() {
        let clock = FakeClock::new();
        let mut sched = Schedule::new(clock.arc(), super::super::Cadence::default());
        assert_eq!(sched.sweep(), Sweep::None);
        clock.advance(CHEAP_SWEEP);
        assert_eq!(sched.sweep(), Sweep::Cheap);
        assert_eq!(sched.sweep(), Sweep::None, "cheap deadline reset");
        clock.advance(FULL_SWEEP);
        assert_eq!(sched.sweep(), Sweep::Full);
        assert_eq!(sched.sweep(), Sweep::None, "full reset both deadlines");
        clock.advance(CHEAP_SWEEP);
        assert_eq!(sched.sweep(), Sweep::Cheap, "cheap resumes after a full");
    }
}
