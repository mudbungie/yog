//! **The window's second asker lane** (REMOTE §3, §10; DESIGN §7.2; bl-73e7):
//! one connection and one thread, held open on the focused conversation's live
//! tail while the [`asker`](super::asker) keeps making its serial pass over
//! everything else.
//!
//! **Why a lane and not a question on the standing set.** The asker's pass is
//! serial — ask, wait, publish, next — so a read that deliberately never
//! finishes would stall every other surface for its whole duration. REMOTE §9.7
//! priced exactly that when it declined the graduation, and it is the price the
//!2026-08-22 ruling accepted: a lane, so the two cadences never touch. One
//! connection each, one thread each, and no shared state between them but the
//! frame that reads both.
//!
//! **Re-ask is the whole reconnect ladder.** A stream ends for three reasons and
//! the lane treats them alike: the step committed (the engine terminated it),
//! the subject moved (the lane hung up), or the dial failed. Each is followed by
//! a fresh ask. There is no backoff schedule and no liveness protocol, because
//! the fallback is not "nothing" — while the lane is down the seat paints the
//! tail the **pull** `Query::Transcript` folds, at
//! [`ASK_PERIOD`](super::asker::ASK_PERIOD). That is the migration's own
//! behaviour, kept deliberately, so the lane is an improvement that can fail
//! rather than a mechanism that can break the chat.
//!
//! **Two channels, and deliberately no lock** — [`link`](super::link)'s reason
//! exactly: nothing here is shared mutable state, it is a hand-off in each
//! direction. What crosses is the **whole** fold per frame, never a delta, so a
//! frame the lane misses costs nothing and a seat never reassembles anything.
//!
//! **The question is its own key**, again as `link`'s is: a frame carries the
//! envelope text it answers, so a subject that moved cannot land the previous
//! conversation's tail on the new one.
//!
//! **One lane, dialled at whichever channel hosts the focused conversation**
//! (REMOTE §8.2, bl-670c). The lane does not fan out, because one conversation
//! is focused: it *resolves*, exactly as the poster routes an act — the
//! subject names a workspace, an entry is the answer to its leaf, and every
//! other name is this window's own engine's ([`Dial`](super::dial::Dial)). A
//! subject that moves from a local conversation to a remote one is the same
//! re-ask the lane already performs when a subject moves at all; the seat it
//! re-asks on is the only thing that differs, and nothing above this line
//! notices.

/// **The frame's hand-off, with no wire in it** (REMOTE §3) — the two channels
/// this module's own doc calls *"a hand-off in each direction"*, in their own
/// file at §12's per-file budget. Re-exported whole: `Tail` is what the model
/// holds and `TailEnd` is what a lane is built from, and neither is a second
/// spelling of the other's path.
mod tail;

pub use self::tail::{Tail, TailEnd, pair};

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;

use crate::boundary::reply::Reply;
use crate::watch::Repaint;

use super::asker::ASK_PERIOD;
use super::dial::Dial;

/// One window's follow lane: its own seats on every channel the window holds,
/// its end of the frame's hand-off, and the face to wake when a frame lands.
pub struct Lane {
    dial: Dial,
    end: TailEnd,
    repaint: Arc<dyn Repaint>,
}

impl Lane {
    /// Assemble the lane. Built by [`Engine::lane`](crate::engine::Engine::lane)
    /// so a test can drive [`turn`](Self::turn) by hand — the
    /// [`Asker`](super::asker::Asker) precedent exactly.
    pub fn new(dial: Dial, end: TailEnd, repaint: Arc<dyn Repaint>) -> Self {
        Self { dial, end, repaint }
    }

    /// One turn: hold the line on whatever the frame is following, until the
    /// engine ends the stream, the subject moves, or the dial fails. Answers
    /// whether any frame landed — which is what tells the caller whether the
    /// far end is answering at all, and therefore whether to re-ask at once or
    /// to wait a period first.
    pub fn turn(&mut self) -> bool {
        let Self { dial, end, repaint } = self;
        let Some(question) = end.standing() else {
            return false;
        };
        let key = question.to_string();
        let mut landed = false;
        let _ = dial.followed(&question, &mut |frame| {
            let Ok(Reply::Follow(stream)) = frame else {
                return false;
            };
            end.publish(&key, Some(stream));
            repaint.request();
            landed = true;
            // Stay only while the frame still wants this conversation.
            end.standing().as_ref() == Some(&question)
        });
        // However it ended, the seat is told the stream is over: a tail that
        // stopped growing must not stand painted over a transcript that has
        // since committed it.
        end.publish(&key, None);
        repaint.request();
        landed
    }

    /// Run [`turn`](Self::turn) forever. A turn that landed frames re-asks at
    /// once — the engine holds the next stream open itself, so there is nothing
    /// to pace — and one that landed none waits a period, which is the whole of
    /// the backoff.
    pub fn start(mut self) -> LaneThread {
        let stop = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&stop);
        let handle = std::thread::spawn(move || {
            while !flag.load(Ordering::Relaxed) {
                if !self.turn() {
                    std::thread::park_timeout(ASK_PERIOD);
                }
            }
        });
        LaneThread {
            stop,
            handle: Some(handle),
        }
    }
}

/// The lane thread's handle; [`Drop`] signals stop and unparks.
///
/// **It does not join, and that is the one place this differs from every other
/// thread yog owns** (the worker, the asker, the consumer, the listener: stop,
/// unpark, join). Those are parked on a *local* period, so a join costs at most
/// one tick. This one is parked on a socket read whose bound is the **engine's**
/// hold — thirty seconds of a quiet conversation — so joining it would make
/// closing a window wait on a remote timer. It owns nothing but its own
/// connection and its end of two channels; the frame end dropping is what makes
/// [`TailEnd::standing`] answer `None`, so a lane whose window is gone stops
/// asking on its own turn even if the flag never reached it.
pub struct LaneThread {
    stop: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl Drop for LaneThread {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            handle.thread().unpark();
        }
    }
}

#[cfg(test)]
mod tests;
