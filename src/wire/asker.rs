//! **The window's off-frame asker** (REMOTE §1.2, §3; bl-ae05): the
//! thread that turns the window's standing questions into wire asks and lands
//! decoded replies where the frame can read them.
//!
//! This is the read path the operator ruling of 2026-08-14 chose: *one
//! boundary, everything through the front door* — the local window operates as
//! a remote client on localhost, over the real socket, presenting a real
//! certificate. Not an in-process envelope shim, which REMOTE §11 kept
//! rejected. So an answer painted here has crossed mTLS, been scoped by
//! [`answer_as`](crate::boundary::consumer::ConsumerCtx::answer_as) against the
//! window's registrations, and been decoded by
//! [`reply::decode`](crate::boundary::reply::decode) — every step a phone seat
//! takes, minus the distance.
//!
//! **The frame never waits on it.** The frame declares what it wants through
//! [`Link`](super::link::Link) and paints whatever has landed; this thread does
//! the dialling, the handshake and the decode. That is the §8.5 searcher's
//! shape applied to the transport, and it is what keeps §7.2 true — a slow or
//! dead engine costs a surface its content, never the window its frame rate.
//!
//! **Cadence is human, and connection-per-ask still stands** (REMOTE §10). One
//! pass per [`ASK_PERIOD`] over the standing set, each ask its own connection
//! and its own handshake, which is what §10 ruled and what its criterion for
//! revisiting — *"when a seat's ask rate exceeds human cadence"* — still leaves
//! in force: a window re-reading its roster twice a second is not a machine
//! polling a machine. Holding the connection is `Seat::ask` keeping the stream
//! it drops, the day a follow-class read has a consumer here.
//!
//! **It also seats the window** ([`Asker::seat_window`]). Authorization is
//! registration (REMOTE §4), and the window carries a certificate now, so it is
//! scoped like any client — it would see nothing at all unless something wrote
//! its registrations. The engine that enumerates the workspaces is what knows
//! them, so it seats its own window's leaf in each, and re-seats as the
//! enumeration grows: a workspace founded while the window is up is registered
//! within one pass, with no create to detect.

use super::client::Seat;
use super::link::LinkEnd;
use crate::state::{SnapshotCell, latest_snapshot};
use crate::watch::Repaint;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;
use std::time::Duration;

/// How often the standing set is re-asked — **human cadence** (REMOTE §3). A
/// latency knob on how stale a painted answer may be, and the number REMOTE
/// §10's ask-rate criterion is measured against.
pub const ASK_PERIOD: Duration = Duration::from_millis(500);

/// One window's asker: its seat on the wire, its end of the frame's link, and
/// the enumeration it seats itself against.
pub struct Asker {
    seat: Seat,
    end: LinkEnd,
    snap: SnapshotCell,
    state_root: PathBuf,
    repaint: Arc<dyn Repaint>,
}

impl Asker {
    /// Assemble the asker. Built by [`Engine::asker`](crate::engine::Engine::asker)
    /// so a test can drive [`pass`](Self::pass) by hand — the same reason
    /// `AppModel::searcher` hands back a value rather than spawning.
    pub fn new(
        seat: Seat,
        end: LinkEnd,
        snap: SnapshotCell,
        state_root: PathBuf,
        repaint: Arc<dyn Repaint>,
    ) -> Self {
        Self {
            seat,
            end,
            snap,
            state_root,
            repaint,
        }
    }

    /// One pass: seat the window in whatever the engine now enumerates, then
    /// ask every standing question and publish what came back. Returns how many
    /// answers landed — zero being the resting state of a window that is asking
    /// nothing.
    pub fn pass(&mut self) -> usize {
        self.seat_window();
        let mut landed = 0;
        for question in self.end.standing() {
            let answer = self.seat.answered(&question);
            if !self.end.publish(&question, answer) {
                break;
            }
            landed += 1;
        }
        if landed > 0 {
            self.repaint.request();
        }
        landed
    }

    /// Register the window's identity in every workspace the published
    /// derivation enumerates (REMOTE §4, §4.1). Idempotent and quiet: a
    /// registration that is already there is one directory read, and one that
    /// cannot be written is a state root that is broken in ways this pass
    /// cannot answer for.
    fn seat_window(&self) {
        let client = crate::registry::window();
        let snap = latest_snapshot(&self.snap);
        let seated = crate::registry::registered(&self.state_root, &client);
        for workspace in &snap.workspaces {
            let name = snap.ws_name(&workspace.path);
            if !seated.contains(&name) {
                let _ = crate::registry::register(&self.state_root, &client, &name);
            }
        }
    }

    /// Run [`pass`](Self::pass) forever, parked between looks — the
    /// [`Searcher`](crate::search::Searcher) shutdown shape exactly: a stop
    /// flag, an unpark, a join.
    pub fn spawn(mut self) -> AskerThread {
        let stop = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&stop);
        let handle = std::thread::spawn(move || {
            while !flag.load(Ordering::Relaxed) {
                self.pass();
                std::thread::park_timeout(ASK_PERIOD);
            }
        });
        AskerThread {
            stop,
            handle: Some(handle),
        }
    }
}

/// The asker thread's handle; [`Drop`] signals stop, unparks and joins.
pub struct AskerThread {
    stop: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl Drop for AskerThread {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            handle.thread().unpark();
            let _ = handle.join();
        }
    }
}

#[cfg(test)]
mod tests;
