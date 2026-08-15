//! The suite's deterministic [`Clock`]: a shared instant the test advances by
//! hand, so every §7.2 debounce and sweep branch is exercised without sleeping.
//!
//! Split from [`super`] at §12's cap, on the seam that file already had — the
//! spawn discipline is about *forking*, and this is a value the crate reads.

use crate::ui_state::Clock;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Deterministic [`Clock`] over a shared instant the test advances by hand, so
/// every debounce/sweep branch (§7.2) is exercised without sleeping. Handles
/// cloned via [`FakeClock::handle`] (or `Clone`) share one instant — advancing
/// any handle moves the clock every holder sees (a model and the test both read
/// it, and an [`AppModel`](crate::AppModel) hands one to its sweep schedule).
#[derive(Clone)]
pub(crate) struct FakeClock {
    at: Arc<Mutex<Instant>>,
    /// How far each [`Clock::now`] read moves the clock **by itself**. Zero for
    /// an ordinary fake, where only [`advance`](FakeClock::advance) moves time.
    /// Non-zero makes the work *between* two reads take real time — the one way
    /// to exercise §7.2's late-pass drift without a slow machine, since that
    /// branch is precisely "the clock moved while a pass ran".
    lurch: Duration,
}

impl FakeClock {
    pub(crate) fn new() -> Self {
        Self {
            at: Arc::new(Mutex::new(Instant::now())),
            lurch: Duration::ZERO,
        }
    }

    /// A clock where every read costs `lurch` — a machine under load, in a
    /// deterministic form.
    pub(crate) fn lurching(lurch: Duration) -> Self {
        Self {
            lurch,
            ..Self::new()
        }
    }

    /// A second handle sharing this clock's instant.
    pub(crate) fn handle(&self) -> Self {
        Self {
            at: Arc::clone(&self.at),
            lurch: self.lurch,
        }
    }

    /// This clock as a shared trait object for the `Arc<dyn Clock>` seam
    /// ([`AppModel`](crate::AppModel), `Schedule`). Shares the instant, so
    /// advancing the original still moves the clock the model holds.
    pub(crate) fn arc(&self) -> Arc<dyn Clock> {
        Arc::new(self.handle())
    }

    pub(crate) fn advance(&self, delta: Duration) {
        *self
            .at
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) += delta;
    }
}

impl Clock for FakeClock {
    fn now(&self) -> Instant {
        let mut at = self
            .at
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let read = *at;
        *at += self.lurch;
        read
    }

    /// A fixed wall-clock stamp. `ops.jsonl` treats `ts` as opaque (§4.2), so a
    /// constant is the deterministic reading — and it is the literal every
    /// drift/ops assertion in the suite already spells.
    fn stamp(&self) -> String {
        "TS".to_string()
    }
}
