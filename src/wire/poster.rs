//! **The window's off-frame poster** (REMOTE §1.2, §9.8; bl-4841): the thread
//! that sends the window's acts over the wire and lands their receipts where
//! the frame can read them.
//!
//! It is the [`asker`](super::asker)'s twin on the write side, and it differs in
//! exactly two ways, both of which follow from what an act *is*.
//!
//! **It is pushed, never polled.** A read is standing — the asker re-asks the
//! set on a period because the world moves under it. An act happens once, when
//! an operator clicks, so this thread parks on its channel and dials the moment
//! something is posted. That is also its whole lifetime: the recv ends when the
//! frame's [`Post`](super::post::Post) drops, so there is no stop flag, no
//! unpark and no join — the channel is the thread.
//!
//! **It is a thread of its own, and that is the point.** An act can take as long
//! as the verb behind it: a `bl close` runs a gate, and a piped `lernie` verb
//! runs until it is done. On the asker's thread that would stall every standing
//! read for the duration, which is the frame going blind because the operator
//! clicked something. Acts are serialized among themselves — one connection at a
//! time, in the order they were clicked — which is strictly less blocking than
//! the in-process dispatch this replaces, where the *frame itself* waited.

use super::client::Seat;
use super::post::Outbox;
use crate::watch::Repaint;
use std::sync::Arc;
use std::thread::JoinHandle;

/// One window's poster: its seat on the wire, its end of the frame's outbox,
/// and the repaint that wakes the glass when a receipt lands.
pub struct Poster {
    seat: Seat,
    outbox: Outbox,
    repaint: Arc<dyn Repaint>,
}

impl Poster {
    /// Assemble the poster. Built by
    /// [`Engine::poster`](crate::engine::Engine::poster) so a test can drive
    /// [`pass`](Self::pass) by hand — the asker's own reason exactly.
    pub fn new(seat: Seat, outbox: Outbox, repaint: Arc<dyn Repaint>) -> Self {
        Self {
            seat,
            outbox,
            repaint,
        }
    }

    /// Take one act, send it, publish what it earned. Blocks until there is an
    /// act to send; answers `false` when the window has gone away, which is the
    /// only way the loop below ends.
    pub fn pass(&mut self) -> bool {
        let Some((ticket, act)) = self.outbox.next() else {
            return false;
        };
        let landed = self.seat.answered(&act);
        let alive = self.outbox.publish(ticket, landed);
        self.repaint.request();
        alive
    }

    /// Run [`pass`](Self::pass) until the window's end of the channel closes.
    /// The handle is handed back rather than detached so a caller *can* join;
    /// dropping it is the ordinary exit, because the thread ends on its own the
    /// moment there is nobody left to post.
    pub fn spawn(mut self) -> JoinHandle<()> {
        std::thread::spawn(move || while self.pass() {})
    }
}

#[cfg(test)]
mod tests;
