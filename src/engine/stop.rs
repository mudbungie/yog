//! **What a SIGTERM means to a running yog** (DESIGN §8.5, bl-269a) — the one
//! mechanism, consulted by both faces.
//!
//! A stop here is *"stop taking new work, finish what is running, drop the
//! engine"* — and it is almost entirely a subtraction, because dropping the
//! engine is already the whole of it. Every thread [`Engine::boot`] spawns
//! owns the §7.2 shutdown shape (stop flag, park loop, a [`Drop`] that unparks
//! and joins), so an engine dropped at the end of a consumer pass is an engine
//! that refused the next deposit and completed the last one; and every child
//! yog holds a pipe on is behind a [`Stream`](crate::cli_outbound::Stream)
//! whose own drop posts a polite `SIGTERM`, waits a grace, then escalates.
//! Nothing in this module drains, orchestrates or waits on its own account.
//!
//! **What was missing was only the catch.** `yog serve` parked forever with
//! the default `SIGTERM` disposition, so the signal killed the process where
//! it stood: no `Drop` ran, the `Engine` was never dropped, and the polite
//! `SIGTERM` above was never posted. So the fix is a disposition
//! ([`cli_outbound::sys::term_disposition`](crate::cli_outbound)), a flag, and
//! a loop that ends.
//!
//! **The bound is systemd's, not ours.** A consumer pass may be inside a `bl
//! close`; a stop lets it finish and adds no deadline of its own, because the
//! unit already states one (`TimeoutStopSec=30s`, until now unused) and the
//! kernel's `SIGKILL` at the end of it is the only bound that cannot be
//! out-waited. A second timeout in yog would be a worse copy of it.
//!
//! **The substrate settles itself.** A turn in flight is a *detached* `litany`
//! driver in its own process group (§8.1) — deliberately not yog's to kill,
//! and unaffected by yog's exit either way. Under the unit it takes the
//! cgroup's own `SIGTERM` beside yog's, and the pinned litany catches that and
//! deposits its branch's result message with a `stopped` epitaph on the way
//! out (its ARCH §2.9). So yog neither drains nor signals a turn: it stops
//! being in the way of one.
//!
//! **Both faces, one mechanism** (VISION V5.4). They differ only in the loop
//! that consults it, exactly as they already differ in a repaint hook: the
//! windowless face's loop is [`Engine::park_until_stopped`] below, and the
//! window's is eframe's, which asks [`requested`] each frame and closes its
//! viewport — the same close a human's click makes, so the engine drops down
//! the same path either way. Neither face adds a verb, a flag or a config
//! key: the signal is the signal.

use super::Engine;
use crate::cli_outbound::sys;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

/// How often a parked face looks at the flag. A latency knob and not a
/// correctness one (the consumer's own poll is spelled the same way): the flag
/// is level, so a look that is late costs shutdown latency and nothing else.
/// A handler may not `unpark` — `Thread::unpark` is not async-signal-safe — so
/// looking is what there is.
const LOOK: Duration = Duration::from_millis(250);

/// Point `SIGTERM` at the flag instead of at process death. Called once per
/// face at the process edge (`main.rs`), above eframe and beside the other
/// edge-bound arms — and *only* by the two faces that boot an engine, never by
/// `yog env`/`exec`/`tool-control`, which have no engine to drop and for which
/// dying on the spot is the right answer.
pub fn catch() {
    sys::term_disposition(true);
}

/// Has a stop been asked for? Level, not an edge: a face may look as often or
/// as rarely as its own loop allows.
pub fn requested() -> bool {
    sys::term_flag().load(Ordering::SeqCst)
}

impl Engine {
    /// **The windowless face's whole loop** — park until a stop is asked for,
    /// then drop the engine. Takes `self` by value so that returning from it
    /// and stopping the engine are the same act: there is no way to park
    /// without the drop, which is exactly the defect this closes.
    pub fn park_until_stopped(self) {
        park_until(sys::term_flag());
        // The drop **is** the stop (§7.2): each thread's `Drop` sets its flag,
        // unparks and joins, and any live `Stream` posts its child's SIGTERM.
        drop(self);
    }
}

/// Park until `flag` is raised. Injected rather than read from the static so
/// the loop is driven with a flag a test owns; the one production caller hands
/// it the process's own.
fn park_until(flag: &AtomicBool) {
    while !flag.load(Ordering::SeqCst) {
        std::thread::park_timeout(LOOK);
    }
}

#[cfg(test)]
mod tests;
