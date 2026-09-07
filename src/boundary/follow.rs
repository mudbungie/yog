//! **The follow lane's engine half** (REMOTE §3, §10; DESIGN §7.2; bl-73e7):
//! [`Query::Follow`](super::Query::Follow) answered as a *sequence* — one frame
//! per growth of the conversation's open `response.json`, and no terminator
//! until the stream closes.
//!
//! **It carries two things, and they are one lane** (bl-5305). The prose half
//! is the fold above; the other is the **tool window** — an event as a call is
//! posted and another as its capture lands ([`tools`]). A conversation
//! administering a machine says almost nothing in prose while it runs eight
//! commands on two boxes, and the box is precisely the thing an operator cannot
//! inspect afterwards, so a live view that carried only the prose was blind to
//! the part that matters. The transcript recorded every capture the whole time
//! (`Query::Transcript`); it was this view that dropped them.
//!
//! **The subject is the STEP, not the model call inside it** (bl-5305). This
//! read was keyed on the response file, and the response file is one call: the
//! stream therefore ended the instant a call settled into its `tool_use`
//! blocks — *before* the first command was dispatched — so the lane was dead
//! for exactly the minutes an operator most wants it. Two liveness questions
//! tell the halves apart and both are read per look: a driver holding the lease
//! (`Live | InFlight`) is what makes a step worth following at all, and a model
//! call actually streaming (`InFlight`) is what opens the prose. Between the
//! calls of one step the response file is settled and the committed transcript
//! already carries it, so opening it would paint the answer twice — that rule
//! is unchanged, and it is why the two questions are not one.
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

use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::git_tree::{AgentState, Stream, latest_response_path};
use crate::state::SnapshotCell;

use super::reply::Reply;

/// The same lane answered **once**, for the intake that cannot hold a
/// connection — beside the held read, so the two cannot describe one moment
/// differently.
mod once;
/// The incremental read — offset, partial line, fold.
mod open;
/// The **tool window** (bl-5305): which calls this read has already spoken
/// about, and therefore what a look owes the seat.
mod tools;

pub(crate) use once::once;
use open::Open;

/// How long a quiet follow read holds before it ends the stream and lets the
/// seat re-ask: 1875 looks, 16 ms apart — thirty seconds, the mailbox hold's
/// own bound. The tick is the §7.2 follower's own period, which is what "at
/// write cadence" means in a number: half the §11 pulse, so a repaint that was
/// going to happen carries the newest bytes rather than the previous look's.
///
/// `pub(super)` because the attention lane holds on the **same** bound (REMOTE
/// §14.1: "the follow lane's bounded-hold discipline applies unamended") — one
/// pair of numbers, not two that can drift apart.
pub(super) const HOLD_WAITS: u32 = 1875;
pub(super) const HOLD_TICK: Duration = Duration::from_millis(16);

/// What one look found — [`Follow::poll`]'s answer, and the whole vocabulary a
/// held read has. [`Iterator::next`] is this plus the parking, which is why a
/// test can drive the mechanism with no clock and no sleep at all.
pub(crate) enum Frame {
    /// The lane moved: a frame to write, carrying **what landed since the last
    /// one** (bl-3655) — the fold of the appended prose, and the tool-window
    /// events discovered since the previous look (bl-5305). It carries the two
    /// values rather than the [`Reply`] wrapping them, so the vocabulary a held
    /// read has is the vocabulary of the things it follows.
    ///
    /// **Either half alone is a frame.** A step that runs a `find /` says
    /// nothing in prose for a minute, and a frame gated on the fold moving
    /// would drop exactly the events this lane was widened to carry.
    Ready(Stream, Vec<crate::git_tree::ToolEvent>),
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
    /// **The step this read is following** — the stream's identity, and the
    /// subject of the whole lane (bl-5305). It used to be the response file,
    /// which is a *model call*: the two are the same thing right up until the
    /// call settles and the step's tools start running, at which point a read
    /// keyed on the file ends and the operator loses the lane exactly where
    /// the commands are. `None` until this read has settled on one.
    step: Option<PathBuf>,
    /// The file the prose half is reading, once a call is in flight. `None`
    /// before one begins and between the calls of one step — a settled
    /// response is the committed transcript's answer, and opening it would
    /// paint it twice.
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
    /// The tool window's own watermark — [`Open`]'s counterpart for the step's
    /// `tools/` subtree, minted per read and starting empty for its reason
    /// exactly ([`tools`]).
    window: tools::Window,
    /// The events discovered but not yet written, on `pending`'s own terms: a
    /// look that found both the prose and an event owes one frame, not two.
    pending_tools: Vec<crate::git_tree::ToolEvent>,
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
            step: None,
            open: None,
            pending: Stream::default(),
            window: tools::Window::default(),
            pending_tools: Vec::new(),
            waits,
            quiet: 0,
            tick,
        }
    }

    /// What the published derivation says this conversation is doing. Read per
    /// look rather than carried, for the module doc's reason: a read that
    /// deliberately outlives its request cannot be gated on a snapshot frozen
    /// at connect.
    fn standing(&self) -> AgentState {
        let snap = crate::state::latest_snapshot(&self.cell);
        super::answer::inspector::state_of(&snap, &self.ws, &self.agent)
    }

    /// **One look at the world, taken now.** Public to the crate because it is
    /// the mechanism and [`next`](Iterator::next) is only the patience around
    /// it: a test drives this and asserts on bytes, and the acceptance world
    /// stands in for the transport with it exactly as it stands in for the
    /// asker's pass.
    ///
    /// **Two liveness questions, and telling them apart is the whole of
    /// bl-5305.** A driver holding the lease (`Live | InFlight`) is what makes
    /// a step worth following at all — it is the span in which its tools run.
    /// A model call actually streaming (`InFlight`) is the narrower fact, and
    /// only it opens the prose half: between the calls of one step the response
    /// file is settled and the committed transcript already carries it.
    pub(crate) fn poll(&mut self) -> Frame {
        let standing = self.standing();
        let working = matches!(standing, AgentState::Live | AgentState::InFlight);
        let now = latest_response_path(&self.ws, &self.agent);
        let step = now.as_deref().and_then(Path::parent).map(Path::to_path_buf);
        // The step advancing — or the tree going away — is this stream ending,
        // which is the contract §5.5 already spells: the seat swaps to the
        // committed entry and re-asks for the next step's.
        if self.step.is_some() && self.step != step {
            return Frame::Over;
        }
        // No step, or one nobody is working that this read never opened: a
        // hold, not an answer.
        let Some(step_dir) = step else {
            return Frame::Waiting;
        };
        if !working && self.step.is_none() {
            return Frame::Waiting;
        }
        self.step = Some(step_dir.clone());
        if standing == AgentState::InFlight && self.open.is_none() {
            self.open = now.map(Open::at);
        }
        if let Some(open) = self.open.as_mut()
            && let Some(appended) = open.read_appended()
        {
            self.pending.absorb(appended);
        }
        // The tool window is the step's, so it is looked at beside the response
        // file rather than on a cadence of its own — and it outlives the call,
        // which is why it is not inside the branch above.
        self.pending_tools.extend(self.window.look(&step_dir));
        // The final bytes come out before the close: a step that committed
        // between two looks still wrote what it wrote.
        if self.pending != Stream::default() || !self.pending_tools.is_empty() {
            return Frame::Ready(
                std::mem::take(&mut self.pending),
                std::mem::take(&mut self.pending_tools),
            );
        }
        if working { Frame::Waiting } else { Frame::Over }
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
                Frame::Ready(stream, tools) => {
                    self.quiet = 0;
                    return Some(Reply::Follow(super::reply::FollowFrame { stream, tools }));
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
