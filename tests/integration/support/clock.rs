//! The harness's own [`Clock`] (DESIGN §7.2): a shared instant the test moves
//! by hand, so a beat about what yog does *over time* drives time instead of
//! racing the machine it runs on.
//!
//! **Why this exists** (bl-9006). `AppModel::boot` takes `Arc<dyn Clock>`
//! precisely so a test can supply one, and the crate's own unit suite already
//! does (`test_support::FakeClock`) — but that fake is `pub(crate)` and this is
//! a separate crate, so the seam was reachable and unused here. Every
//! integration beat therefore booted on `SystemClock` and measured **real**
//! elapsed time, which under a loaded gate is not the time the beat's author
//! ever saw: `stories_inv1` read a nine-tarpaulin machine as a yog that
//! mutated at idle, went red inside an unrelated agent's close, and read as
//! that agent's fault.
//!
//! The instant advances **only** when a test says so. That is the whole
//! contract, and it is what separates the two facts a real clock fuses: time
//! passing *between* passes (which is what makes §7.2's periodic sweeps fall
//! due, and is legitimate to simulate) from time passing *inside* one pass
//! (which is lateness, §7.2's `yog-drift late`, and belongs to the beat that
//! asserts lateness — `app::tests::worker`, on the crate's own lurching fake).

use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, Instant};
use yog::ui_state::Clock;

/// A clock frozen at construction, moved only by [`TestClock::advance`].
/// Handles share one instant, so the model's copy and the test's are the same
/// clock.
pub struct TestClock {
    at: Arc<Mutex<Instant>>,
}

impl TestClock {
    pub fn new() -> Self {
        Self {
            at: Arc::new(Mutex::new(Instant::now())),
        }
    }

    /// This clock as the shared trait object `AppModel::boot` and the §7.2
    /// sweep schedule take.
    pub fn arc(&self) -> Arc<dyn Clock> {
        Arc::new(Self {
            at: Arc::clone(&self.at),
        })
    }

    /// Move every holder's clock forward.
    pub fn advance(&self, delta: Duration) {
        *self.at.lock().unwrap_or_else(PoisonError::into_inner) += delta;
    }
}

impl Default for TestClock {
    fn default() -> Self {
        Self::new()
    }
}

impl Clock for TestClock {
    fn now(&self) -> Instant {
        *self.at.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// A fixed stamp: `ops.jsonl` treats `ts` as opaque (§4.2), and the crate's
    /// own fake spells the same literal.
    fn stamp(&self) -> String {
        "TS".to_string()
    }
}
