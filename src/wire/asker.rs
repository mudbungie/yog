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
//! **One asker per channel** (REMOTE §8.2, bl-670c). The window is a client of
//! the engine in its own process *plus one per [entry](super::entries)*, and
//! this is one of them: one seat, one [`Link`](super::link::Link) end, one
//! thread. That is what makes an unreachable entry cost only its own slice —
//! the passes never touch, so a channel parked on a dead host's connect is not
//! a channel the local roster is waiting behind. A seat that will not open is
//! the same shape one step earlier: the sentence is held and answered in place
//! of every question standing on that channel, so the slice says why instead of
//! staying empty.
//!
//! **It also seats the window, on the loopback channel and nowhere else**
//! ([`Seating`]). Authorization is registration (REMOTE §4), and the window
//! carries a certificate now, so it is scoped like any client — it would see
//! nothing at all unless something wrote its registrations. The engine that
//! enumerates the workspaces is what knows them, so it seats its own window's
//! leaf in each, and re-seats as the enumeration grows: a workspace founded
//! while the window is up is registered within one pass, with no create to
//! detect. **An entry channel seats nothing**, and that is REMOTE §4.1 rather
//! than a limitation: a registration is a file on the engine's own disk, minted
//! by the operator who owns that box (§1.4), and nothing on this side of the
//! wire may write one. The window's leaf reaches a host the way its certificate
//! did — by hand, out of channel.

/// **The one act an asker performs that is not a question** (REMOTE §4, §4.1):
/// seating this window's leaf in the workspaces its own engine enumerates.
/// Split off at §12's band, and the split is the rule: it is the loopback
/// channel's alone.
mod seating;
use seating::Seating;

use super::client::Seat;
use super::link::{Landed, LinkEnd};
use crate::state::SnapshotCell;
use crate::watch::Repaint;
use serde_json::Value;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;
use std::time::Duration;

/// How often the standing set is re-asked — **human cadence** (REMOTE §3). A
/// latency knob on how stale a painted answer may be, and the number REMOTE
/// §10's ask-rate criterion is measured against.
pub const ASK_PERIOD: Duration = Duration::from_millis(500);

/// One channel's asker: its seat on that channel — or the sentence saying why
/// this box has none for it — its end of the frame's link, and the registry it
/// seats the window in, on the one channel where that is this engine's to do.
pub struct Asker {
    seat: Result<Seat, String>,
    end: LinkEnd,
    seating: Option<Seating>,
    repaint: Arc<dyn Repaint>,
}

impl Asker {
    /// **The loopback channel's asker** — this engine's own, and the only one
    /// that seats the window (see the module doc). Built by
    /// [`Engine::asker`](crate::engine::Engine::asker) so a test can drive
    /// [`pass`](Self::pass) by hand — the same reason `Engine::searcher` hands
    /// back a value rather than spawning.
    pub fn new(
        seat: Seat,
        end: LinkEnd,
        snap: SnapshotCell,
        state_root: PathBuf,
        repaint: Arc<dyn Repaint>,
    ) -> Self {
        Self {
            seat: Ok(seat),
            end,
            seating: Some(Seating::new(snap, state_root)),
            repaint,
        }
    }

    /// **One §8.2 entry channel's asker**, on that entry's own material. It
    /// seats nothing and enumerates nothing: what it holds is one seat and one
    /// slice. `seat` carries the entry's refusal where the channel cannot be
    /// dialled, which every standing question is then answered with.
    pub fn entry(seat: Result<Seat, String>, end: LinkEnd, repaint: Arc<dyn Repaint>) -> Self {
        Self {
            seat,
            end,
            seating: None,
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
            let answer = self.answered(&question);
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

    /// What one standing question earned — the seat's answer, or this
    /// channel's refusal in place of it. A channel that cannot be dialled says
    /// so on every question standing on it, which is the one `Err` every read
    /// surface already paints (`shell::wire`).
    fn answered(&self, question: &Value) -> Landed {
        match &self.seat {
            Ok(seat) => seat.answered(question),
            Err(said) => Err(said.clone()),
        }
    }

    /// Register the window's identity in every workspace this engine
    /// enumerates ([`Seating::seat_window`]). **Nothing on an entry channel** —
    /// a registration is the host operator's file (§1.4), so an asker with no
    /// [`Seating`] does not skip a step, it has no step to take.
    fn seat_window(&self) {
        if let Some(seating) = &self.seating {
            seating.seat_window();
        }
    }

    /// Run [`pass`](Self::pass) forever, parked between looks — the
    /// [`Searcher`](crate::search::Searcher) shutdown shape exactly: a stop
    /// flag, an unpark, a join.
    pub fn start(mut self) -> AskerThread {
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
