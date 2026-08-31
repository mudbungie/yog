//! **The follow lane's engine half** (REMOTE §3, §10; DESIGN §7.2; bl-73e7):
//! [`Query::Follow`](super::Query::Follow) answered as a *sequence* — one frame
//! per growth of the conversation's open `response.json`, and no terminator
//! until the stream closes.
//!
//! **It is a cadence, not a second reading.** Whether there is a tail at all is
//! [`live_tail`](super::answer::inspector::live_tail) — bl-6233's one describer,
//! unmoved, so a follow frame and the tail folded into a
//! [`Transcript`](crate::transcript::Transcript) cannot disagree about a
//! moment. What this adds is *where the bytes are read from*: the derivation
//! folds the whole file on the worker's schedule, and this folds the suffix on
//! the writer's ([`open`]). The two agree by
//! [`absorb`](crate::git_tree::Stream::absorb)'s contract rather than by
//! coincidence, and `follow::tests` pins that.
//!
//! **A frame is an append, and the reassembly is the fold's own contract**
//! (REMOTE §5's follow-lane ruling, bl-3655). Each frame carries what landed
//! since the frame before it, and a seat folds them onto what it holds with
//! [`Stream::absorb`](crate::git_tree::Stream::absorb) — the same operation
//! this file uses to gather them, whose contract
//! (`fold(a).absorb(fold(b)) == fold(a ++ b)`) is what makes the two one
//! description. A read starts holding nothing and its reader opens at byte
//! zero, so the *first* frame is the whole tail and the rule needs no case for
//! joining late: absorb every frame of a read, in order, onto an empty fold.
//!
//! A frame used to carry the whole accumulated answer instead, re-sent from the
//! beginning each time. That bought idempotence and cost **quadratic** wire
//! bytes in the answer's length — measured at 20x amplification on a
//! two-sentence reply, and that ratio is the floor. The property it bought is
//! kept where it was actually needed (a seat that dropped a connection re-asks
//! and is answered from zero) and paid for where it was not: this lane exists
//! for a phone on a mobile link watching a long answer write itself.
//!
//! **The stream is one step's.** A response file belongs to exactly one step,
//! so the step advancing is not an accumulator to reset — it is this stream
//! ending, which the frame protocol already spells (a zero-length frame). The
//! seat then swaps to the committed entry the pull `Query::Transcript` carries,
//! with nothing to reconcile, and re-asks for the next step's stream. That
//! dissolves the follower's old three-part reset rule into one fact.
//!
//! **The hold is bounded, and frames are what prove the peer.** A quiet look
//! counts against [`HOLD_WAITS`]; a frame resets the count, because writing one
//! is what discovers a peer that went away (the connection thread's write
//! fails and the answer is dropped). So a conversation streaming for an hour
//! holds its lane for an hour, and a peer that vanished mid-think costs a
//! thread for thirty seconds — the [`Mailbox`](crate::registry::mailbox::Mailbox)
//! hold's own trade, with the same two knobs.
//!
//! **The snapshot is read live, not carried.** Every other boundary read takes
//! the derivation off its [`Deps`](super::dispatch::Deps), which is a clone
//! taken when the request arrived; a read that deliberately outlives its
//! request cannot use one — a tail gated on a snapshot frozen at connect would
//! never notice the step commit that ends it. So this holds the cell the worker
//! publishes into and reads it per look. The **address** is resolved once, at
//! connect, under the caller's scope (REMOTE §4) — so what is re-read per look
//! is the state of a conversation this caller was already authorized for.

use std::path::PathBuf;
use std::time::Duration;

use crate::git_tree::{Stream, latest_response_path};
use crate::state::SnapshotCell;

use super::reply::Reply;

/// The incremental read — offset, partial line, fold.
mod open;

use open::Open;

/// How long a quiet follow read holds before it ends the stream and lets the
/// seat re-ask: 1875 looks, 16 ms apart — thirty seconds, the mailbox hold's
/// own bound. The tick is the §7.2 follower's own period, which is what "at
/// write cadence" means in a number: half the §11 pulse, so a repaint that was
/// going to happen carries the newest bytes rather than the previous look's.
const HOLD_WAITS: u32 = 1875;
const HOLD_TICK: Duration = Duration::from_millis(16);

/// What one look found — [`Follow::poll`]'s answer, and the whole vocabulary a
/// held read has. [`Iterator::next`] is this plus the parking, which is why a
/// test can drive the mechanism with no clock and no sleep at all.
pub(crate) enum Frame {
    /// The fold moved: a frame to write, carrying **what landed since the last
    /// one** (bl-3655). It carries a [`Stream`] rather than the [`Reply`]
    /// wrapping it, so the vocabulary a held read has is the vocabulary of the
    /// thing it follows — and so the arms stay the same size.
    Ready(Stream),
    /// Nothing new yet. The hold's own answer, and never an end.
    Waiting,
    /// The stream ended — the step committed, advanced, or the tree went away.
    Over,
}

/// One conversation's live tail, as a frame sequence.
pub(crate) struct Follow {
    cell: SnapshotCell,
    ws: PathBuf,
    agent: String,
    /// The file this stream is, once a call is in flight. `None` before one
    /// begins — which is a hold, not an answer.
    open: Option<Open>,
    /// **What is owed to the seat**: the fold of everything that has landed
    /// since the last frame went out, and the body of the next one (bl-3655).
    ///
    /// It starts **empty rather than absent**, and it is emptied by every frame
    /// — so the first frame of a read carries the whole file's fold (a reader
    /// is minted per held connection and opens at byte zero, [`open`]) and each
    /// later one carries only the appended part. It is also what decides
    /// *whether* there is a frame: bytes moving is not the same as the tail
    /// moving, and a `message_start` or a tool-argument delta folds to nothing,
    /// advancing the offset while saying nothing an operator can see.
    pending: Stream,
    waits: u32,
    quiet: u32,
    tick: Duration,
}

impl Follow {
    /// Follow `agent`'s live tail in `ws`, on the production hold.
    pub(crate) fn new(cell: SnapshotCell, ws: PathBuf, agent: String) -> Self {
        Self::holding(cell, ws, agent, HOLD_WAITS, HOLD_TICK)
    }

    /// The same, on a stated hold — the production bound is [`new`](Self::new),
    /// and a test names a short one rather than sleeping for real
    /// ([`Mailbox::holding`](crate::registry::mailbox::Mailbox::holding)'s own
    /// shape).
    pub(crate) fn holding(
        cell: SnapshotCell,
        ws: PathBuf,
        agent: String,
        waits: u32,
        tick: Duration,
    ) -> Self {
        Self {
            cell,
            ws,
            agent,
            open: None,
            pending: Stream::default(),
            waits,
            quiet: 0,
            tick,
        }
    }

    /// Whether this conversation has a tail at all right now — bl-6233's own
    /// gate, asked of the snapshot the worker last published.
    fn in_flight(&self) -> bool {
        let snap = crate::state::latest_snapshot(&self.cell);
        super::answer::inspector::live_tail(&snap, &self.ws, &self.agent).is_some()
    }

    /// **One look at the world, taken now.** Public to the crate because it is
    /// the mechanism and [`next`](Iterator::next) is only the patience around
    /// it: a test drives this and asserts on bytes, and the acceptance world
    /// stands in for the transport with it exactly as it stands in for the
    /// asker's pass.
    pub(crate) fn poll(&mut self) -> Frame {
        let live = self.in_flight();
        let now = latest_response_path(&self.ws, &self.agent);
        let mut open = match (self.open.take(), now) {
            // The stream continues.
            (Some(open), Some(now)) if open.path == now => open,
            // The step advanced, or the tree went away: this stream is over.
            (Some(_), _) => return Frame::Over,
            // A call has begun. A response file with nothing in flight is the
            // last step's settled answer — the committed transcript's, not a
            // tail — so it is not opened.
            (None, Some(now)) if live => Open::at(now),
            (None, _) => return Frame::Waiting,
        };
        if let Some(appended) = open.read_appended() {
            self.pending.absorb(appended);
        }
        self.open = Some(open);
        // The final bytes come out before the close: a step that committed
        // between two looks still wrote what it wrote.
        if self.pending != Stream::default() {
            return Frame::Ready(std::mem::take(&mut self.pending));
        }
        if live { Frame::Waiting } else { Frame::Over }
    }
}

impl Iterator for Follow {
    type Item = Reply;

    /// The next frame, or the end of the stream. Parks between looks, which is
    /// the whole of what makes this a held read — the caller is a connection
    /// thread and nothing else waits on it.
    fn next(&mut self) -> Option<Reply> {
        loop {
            match self.poll() {
                Frame::Ready(stream) => {
                    self.quiet = 0;
                    return Some(Reply::Follow(stream));
                }
                Frame::Over => return None,
                Frame::Waiting => {
                    self.quiet += 1;
                    if self.quiet >= self.waits {
                        return None;
                    }
                    std::thread::sleep(self.tick);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests;
