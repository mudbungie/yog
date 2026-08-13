//! The thread the derivation runs on (DESIGN §7.2, bl-ee0a).
//!
//! yog's frame thread does two things: render the latest completed
//! [`Snapshot`](super::super::Snapshot) and capture input. Everything else — the
//! watchers, the dirty-root routing, both sweeps, every re-derivation, the ball
//! and ops fetches, the liveness probes — is [`Deriver::step`], and this is
//! where it is driven.
//!
//! The shell is deliberately three lines of loop. All the logic is in
//! [`Deriver`], which is a plain value over an injected clock, so every branch
//! is exercised by calling `step()` by hand — no thread, no sleeps, no
//! timing-dependent assertions. What is left here is the one thing a test
//! cannot fake, and it gets one test that runs the real thread.
//!
//! **The frame never waits on this.** The only shared state is
//! [`crate::state`]'s two hand-offs, each locked for a single pointer move; a
//! pass that takes ten seconds delays the *next snapshot*, never a frame.
//! Correctness does not ride on the poll interval either — [`WORKER_POLL`] is a
//! latency knob, the sweeps are the correctness floor (§7.2).

use super::Deriver;
use crate::watch::Repaint;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;
use std::time::Duration;

/// How often the worker looks for work. Short enough that a disk change
/// surfaces well inside a frame's own budget, and cheap when idle: an empty
/// pass is a drain of two empty maps and two deadline comparisons.
const WORKER_POLL: Duration = Duration::from_millis(25);

/// The derivation thread. Owns its join handle and a stop flag; [`Drop`]
/// signals stop, unparks, and joins for a clean shutdown — the same shape the
/// watch bridge had before it folded into this one thread (§7.2).
pub struct Worker {
    stop: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl Worker {
    /// Take ownership of `deriver` and run its pass forever. `repaint` wakes the
    /// egui event loop when — and only when — a new snapshot was published, so
    /// a quiet world costs zero frames.
    pub fn spawn(mut deriver: Deriver, repaint: impl Repaint + 'static) -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&stop);
        let handle = std::thread::spawn(move || {
            while !flag.load(Ordering::Relaxed) {
                if deriver.step() {
                    repaint.request();
                }
                std::thread::park_timeout(WORKER_POLL);
            }
        });
        Self {
            stop,
            handle: Some(handle),
        }
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            handle.thread().unpark();
            let _ = handle.join();
        }
    }
}
