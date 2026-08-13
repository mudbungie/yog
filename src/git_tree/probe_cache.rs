//! A 2 s TTL cache over any liveness probe (DESIGN §10).
//!
//! `lsof` is expensive (§10: "lsof is slow, so macOS probe results carry a
//! 2 s TTL cache (RAM, §5.3)"), so the macOS backend ([`super::lsof`]) is
//! wrapped in this decorator before the classifier ever observes through it.
//! The cache is a *pure* struct — clock-injected ([`Clock`], the §7.2 timing
//! seam reused verbatim) and table-tested without sleeps — that wraps **any**
//! [`LockProbe`] / [`WriterProbe`]: results are keyed by the observed target
//! path, so one cache serves both the inbox-dir lock question and the
//! `response.json` writer question without collision.
//!
//! Contract (§10): a result younger than [`TTL`] is reused; an older one is
//! re-observed; [`TtlCache::invalidate`] evicts a target so its next read
//! recomputes — the eager refresh performed on a watcher event touching the
//! agent. Re-probing *only* Live/InFlight agents on the sweep (§7.2) is the
//! caller's policy; the cache merely answers freshly or from store when asked.
//!
//! Interior mutability is a poison-immune [`Mutex`] (via
//! [`PoisonError::into_inner`](std::sync::PoisonError::into_inner) — no panic
//! path) because the probe traits observe through `&self`. yog derives on the
//! single frame thread (§7.2), so the lock is uncontended and never actually
//! shared across threads. This lock is the one deliberate exception to the
//! `Mutex`-in-`state.rs` chokepoint (`rules/locks-outside-state.yml` ignores
//! this file): it is single-thread interior mutability local to the probe
//! stack, not the cross-thread shared state `state.rs` audits — and folding a
//! generic, monomorphized decorator into `state.rs` breaks llvm-cov's
//! per-line coverage on its `impl` headers (the phantom-region hazard `lib.rs`
//! documents), which would make the chokepoint file's 100% floor fragile to
//! the very growth the rule invites.
//!
//! Linux never caches (its `/proc` probes are cheap and always definite, §10),
//! so this module is compiled only under `cfg(test)` (for its coverage) and on
//! macOS (its one production consumer) — see the gate in [`super`].

use super::probe::{LockProbe, Probe, WriterProbe};
use crate::ui_state::Clock;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

/// Freshness bound for a cached observation (DESIGN §10: "2 s TTL cache").
const TTL: Duration = Duration::from_secs(2);

/// A TTL cache wrapping any probe `P`, timed by an injected clock `C`.
pub(super) struct TtlCache<P, C: Clock> {
    inner: P,
    clock: C,
    cache: Mutex<HashMap<PathBuf, (Instant, Probe)>>,
}

impl<P, C: Clock> TtlCache<P, C> {
    pub(super) fn new(inner: P, clock: C) -> Self {
        let cache = Mutex::new(HashMap::new());
        Self {
            inner,
            clock,
            cache,
        }
    }

    /// The cache map, locked (poison-immune — no panic path).
    fn entries(&self) -> MutexGuard<'_, HashMap<PathBuf, (Instant, Probe)>> {
        self.cache.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// A cached result for `target` younger than [`TTL`], else `compute` it and
    /// store it stamped now. The read lock is scoped closed (in [`Self::fresh`])
    /// before the write lock is taken.
    fn cached(&self, target: &Path, compute: impl FnOnce() -> Probe) -> Probe {
        let now = self.clock.now();
        if let Some(probe) = self.fresh(target, now) {
            return probe;
        }
        let probe = compute();
        self.entries().insert(target.to_path_buf(), (now, probe));
        probe
    }

    /// The stored result for `target` if still within [`TTL`] of `now`, else
    /// `None` (absent or expired). Locks the map read-only and drops it.
    fn fresh(&self, target: &Path, now: Instant) -> Option<Probe> {
        let &(at, probe) = self.entries().get(target)?;
        (now.saturating_duration_since(at) < TTL).then_some(probe)
    }

    /// Evict `target` so its next observation recomputes — the eager refresh on
    /// the §7.2 targeted liveness re-probe (DESIGN §10). Evicting an absent key
    /// is a no-op. Wired through [`ProbeStack::invalidate_liveness`](super::ProbeStack::invalidate_liveness),
    /// which the app-layer cheap sweep drives.
    pub(super) fn invalidate(&self, target: &Path) {
        self.entries().remove(target);
    }
}

impl<P: LockProbe, C: Clock> LockProbe for TtlCache<P, C> {
    fn lock_state(&self, inbox_dir: &Path) -> Probe {
        self.cached(inbox_dir, || self.inner.lock_state(inbox_dir))
    }
}

impl<P: WriterProbe, C: Clock> WriterProbe for TtlCache<P, C> {
    fn writer_state(&self, path: &Path) -> Probe {
        self.cached(path, || self.inner.writer_state(path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::FakeClock;
    use std::cell::Cell;

    /// A probe counting its observations, so a test can prove a cache hit
    /// skipped the inner call. Answers a fixed [`Probe`] for both questions.
    struct CountingProbe {
        answer: Probe,
        lock_calls: Cell<usize>,
        writer_calls: Cell<usize>,
    }

    impl CountingProbe {
        fn new(answer: Probe) -> Self {
            Self {
                answer,
                lock_calls: Cell::new(0),
                writer_calls: Cell::new(0),
            }
        }
    }

    impl LockProbe for CountingProbe {
        fn lock_state(&self, _dir: &Path) -> Probe {
            self.lock_calls.set(self.lock_calls.get() + 1);
            self.answer
        }
    }

    impl WriterProbe for CountingProbe {
        fn writer_state(&self, _path: &Path) -> Probe {
            self.writer_calls.set(self.writer_calls.get() + 1);
            self.answer
        }
    }

    fn dir() -> &'static Path {
        Path::new("/ws/inbox/agent-1")
    }

    #[test]
    fn first_observation_is_a_miss_then_within_ttl_is_a_hit() {
        let clock = FakeClock::new();
        let cache = TtlCache::new(CountingProbe::new(Probe::Held), clock.handle());
        assert_eq!(cache.lock_state(dir()), Probe::Held);
        // A second read inside the window returns the stored value without
        // re-observing.
        clock.advance(Duration::from_millis(1999));
        assert_eq!(cache.lock_state(dir()), Probe::Held);
        assert_eq!(cache.inner.lock_calls.get(), 1, "one observation, one hit");
    }

    #[test]
    fn expired_entry_is_recomputed() {
        let clock = FakeClock::new();
        let cache = TtlCache::new(CountingProbe::new(Probe::Free), clock.handle());
        assert_eq!(cache.lock_state(dir()), Probe::Free);
        // Exactly TTL later the entry is no longer fresh (`< TTL` is false).
        clock.advance(TTL);
        assert_eq!(cache.lock_state(dir()), Probe::Free);
        assert_eq!(cache.inner.lock_calls.get(), 2, "stale entry re-observed");
    }

    #[test]
    fn invalidate_forces_the_next_read_to_recompute() {
        let clock = FakeClock::new();
        let cache = TtlCache::new(CountingProbe::new(Probe::Held), clock.handle());
        assert_eq!(cache.lock_state(dir()), Probe::Held);
        cache.invalidate(dir());
        // Still inside the TTL window, but the eager refresh evicted the entry.
        assert_eq!(cache.lock_state(dir()), Probe::Held);
        assert_eq!(cache.inner.lock_calls.get(), 2, "eviction re-observed");
        // Evicting an absent key is a harmless no-op.
        cache.invalidate(Path::new("/never/cached"));
    }

    #[test]
    fn writer_question_is_cached_independently_of_the_lock() {
        let clock = FakeClock::new();
        let cache = TtlCache::new(CountingProbe::new(Probe::Held), clock.handle());
        let file = Path::new("/ws/steps/agent-1/003/response.json");
        // Distinct target keys: the writer entry is separate from the lock's.
        assert_eq!(cache.writer_state(file), Probe::Held);
        assert_eq!(cache.writer_state(file), Probe::Held);
        assert_eq!(
            cache.inner.writer_calls.get(),
            1,
            "writer cached by its key"
        );
        assert_eq!(cache.inner.lock_calls.get(), 0, "lock question untouched");
    }
}
