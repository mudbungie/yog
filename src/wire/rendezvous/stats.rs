//! **What the loop has done, and what a seat may read of it** (REMOTE §13.4,
//! bl-355c). [`Stats`] is the loop's own tally, shared by handle with the
//! engine's [`Listening`](crate::wire::Listening) so `/doctor` reads this
//! instant's counters rather than a copy; [`Standing`] is the part of it that
//! crosses the wire, on `reply/doctor` as its `rendezvous` object.
//!
//! The failure counters stay here and off the wire: a seat reads a failing
//! commons as `last_poll_unix` moving while `published` and `calls` do not,
//! and the engine's stderr says each failure as it happens.

use crate::wire::Listening;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};

/// The loop's counters, written by its thread and the punches it spawns.
#[derive(Default)]
pub(crate) struct Stats {
    /// Whether a loop is running in this process — false on a loopback-only
    /// box, which started none (REMOTE §13.4), and after the loop's `Drop`.
    pub(crate) active: AtomicBool,
    pub(crate) published: AtomicUsize,
    pub(crate) publish_failures: AtomicUsize,
    pub(crate) polls: AtomicUsize,
    pub(crate) poll_failures: AtomicUsize,
    /// Calls that verified, unsealed and were new — each one a punch.
    pub(crate) calls: AtomicUsize,
    /// Punches that landed at least one stream.
    pub(crate) punched: AtomicUsize,
    /// Streams handed to the serving code — a punch's and the acceptor's.
    pub(crate) served: AtomicUsize,
    /// Of those, streams the punch port's acceptor took with no window
    /// toward their peer (bl-5276): a re-punch through a still-live mapping, a plain
    /// connect through a live mapping, a SYN that beat the engine's poll.
    pub(crate) accepted: AtomicUsize,
    /// When the inbox was last read, answered or not; `0` is never.
    pub(crate) last_poll_unix: AtomicU64,
}

/// The loop's standing as a seat reads it — `reply/doctor`'s `rendezvous`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Standing {
    /// A loop is running in the answering engine.
    pub active: bool,
    /// Presence publishes that stored somewhere.
    pub published: u64,
    /// New calls opened, each one a punch started.
    pub calls: u64,
    /// Punches that landed a stream.
    pub punched: u64,
    /// Streams served.
    pub served: u64,
    /// Of those, streams served with no call's window toward their peer —
    /// absent from an engine before edition 22, and read as `0`.
    pub accepted: u64,
    /// Unix seconds of the last inbox read; `0` is never.
    pub last_poll_unix: u64,
}

impl Stats {
    /// This instant's counters, in the wire's shape.
    pub(crate) fn standing(&self) -> Standing {
        let n = |counter: &AtomicUsize| counter.load(Ordering::Relaxed) as u64;
        Standing {
            active: self.active.load(Ordering::Relaxed),
            published: n(&self.published),
            calls: n(&self.calls),
            punched: n(&self.punched),
            served: n(&self.served),
            accepted: n(&self.accepted),
            last_poll_unix: self.last_poll_unix.load(Ordering::Relaxed),
        }
    }
}

/// `Listening`'s two rendezvous readers — here rather than beside it, since
/// an addition to that `impl` draws llvm-cov's phantom onto its header.
impl Listening {
    /// The handle the rendezvous loop counts into.
    pub(crate) fn rendezvous(&self) -> Arc<Stats> {
        Arc::clone(&self.rendezvous)
    }

    /// The loop's standing as `/doctor` hands it over — `None` where nothing
    /// bound, since only an engine has a loop to speak of.
    pub(crate) fn standing(&self) -> Option<Standing> {
        self.address().map(|_| self.rendezvous.standing())
    }
}
