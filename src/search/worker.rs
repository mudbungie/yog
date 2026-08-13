//! The thread a window's searches run on (§8.5) — never the frame.
//!
//! The frame renders published values and derives nothing (§7.2); a search
//! walks every transcript in the world, so it is the derivation worker's shape
//! applied to a second question: the frame *asks* through the
//! [`SearchCell`](crate::state::SearchCell) and renders whatever answer has
//! landed, exactly as it renders whatever snapshot has landed. There is no
//! frame-side wait to make the window stutter, and no request channel — the
//! cell is both directions.
//!
//! It is a thread of its own rather than a stage of the derivation pass because
//! a long search must not delay a re-derivation: staleness is what the §7.3
//! wound banner is *for*, and a search is not a wound.
//!
//! The other two seats need none of this. `yog gesture` and the deposit
//! consumer are already off-frame, and a process that asked has nothing else to
//! do, so they run [`run`](super::run) straight through
//! [`answer`](crate::boundary::answer::answer). One engine, three seats.

use crate::state::{SearchCell, SnapshotCell, latest_snapshot};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;
use std::time::Duration;

/// How often the searcher looks for an ask. A latency knob only: an ask waits,
/// it never rots — the same reading the §8.5 consumer poll gets.
const SEARCH_POLL: Duration = Duration::from_millis(50);

/// One window's searcher: the published snapshot to search over, and the cell
/// it is asked through. Built by [`AppModel::searcher`](crate::AppModel::searcher)
/// so the model never spawns its own thread — a test drives [`pass`](Self::pass)
/// by hand, which is the same reason `boot` hands back a `Deriver`.
pub struct Searcher {
    snap: SnapshotCell,
    asks: SearchCell,
}

impl Searcher {
    pub(crate) fn new(snap: SnapshotCell, asks: SearchCell) -> Self {
        Self { snap, asks }
    }

    /// One pass: answer the outstanding ask, if there is one. Returns whether
    /// an answer was published — `false` both when nothing was asked and when
    /// the run was superseded mid-flight, which are the same fact from here
    /// (the current question is unanswered either way, and the next pass takes
    /// it).
    pub fn pass(&self) -> bool {
        let Some((seq, text)) = self.asks.pending() else {
            return false;
        };
        let snap = latest_snapshot(&self.snap);
        let found = super::run(&snap, &text, &|| self.asks.seq() == seq);
        self.asks.publish(seq, found);
        self.asks.seq() == seq
    }

    /// Run [`pass`](Self::pass) forever, parked between looks — the
    /// [`Consumer`](crate::boundary::consumer::Consumer) shutdown shape exactly:
    /// a stop flag, an unpark, a join.
    pub fn spawn(self) -> SearchThread {
        let stop = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&stop);
        let handle = std::thread::spawn(move || {
            while !flag.load(Ordering::Relaxed) {
                self.pass();
                std::thread::park_timeout(SEARCH_POLL);
            }
        });
        SearchThread {
            stop,
            handle: Some(handle),
        }
    }
}

/// The searcher thread's handle; [`Drop`] signals stop, unparks and joins.
pub struct SearchThread {
    stop: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl Drop for SearchThread {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            handle.thread().unpark();
            let _ = handle.join();
        }
    }
}
