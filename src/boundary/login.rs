//! **The sign-in at the boundary** (REMOTE §8.3, DESIGN §8.3 as amended by
//! bl-61bf; bl-c285): the act that starts one, the read that answers what it
//! has said, and the lane that keeps saying it.
//!
//! The act is [`Action::Login`](super::Action::Login) and it does exactly two
//! things — start `bz --login --provider <row>`, in the flow that row's own
//! `device` column declares (§8.3 rule 1 as amended by bl-61bf), on the ENGINE
//! inside the *named* workspace's wall, and answer that run's standing. It never
//! waits: a sign-in is minutes of a human's attention and the intake is
//! one thread for the whole world (REMOTE §3), so an act that waited it out
//! would stop every deposit converging. The receipt is therefore the standing
//! **re-read** rather than an echo of what was asked — the `Marks` discipline
//! — which is also why it needs no shape of its own: an act's receipt and a
//! lane's first frame are the same value at the same moment.
//!
//! The read is [`Query::LoginTail`](super::Query::LoginTail), and it is
//! follow-class for `Query::Follow`'s reason exactly (REMOTE §10): the answer
//! is written at the provider's pace, not at the asker's. An intake that can
//! hold a connection gets [`Lane`]'s frames — everything buffered so far, then
//! each arrival, then the settled exit as the last one; an intake that cannot
//! gets [`standing`], which is the same fold answered once. **Re-ask replays**,
//! and that is not a special case: a lane starts holding nothing, so its first
//! frame is the whole buffer and a seat that lost its connection absorbs from
//! empty exactly as a seat that joined at the start did.
//!
//! **A pair with no run is emptiness, never a refusal.** Nobody has signed in
//! to this row in this workspace is a reading of the world; the empty view says
//! it, and the lane opens with one frame of it rather than closing on silence.
//!
//! The run holder itself — replacement, sweep, the reader thread — is
//! [`login::runs`](crate::login::runs); nothing about a child process lives
//! here.

use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::login::{LoginView, runs::Runs};

use super::dispatch::Deps;
use super::reply::Reply;

/// The lane's hold: 1875 looks 16 ms apart — thirty seconds, the §5.3 mailbox
/// bound `follow` already holds a connection for, at the same tick, so a line
/// reaches a seat within one look of the reader thread buffering it.
const HOLD_WAITS: u32 = 1875;
const HOLD_TICK: Duration = Duration::from_millis(16);

/// Start (or restart) the sign-in for `workspace` × `provider` and answer its
/// standing (REMOTE §8.3). `workspace` is the chokepoint's own resolved path,
/// so the wall the child runs in is the one the gesture named (bl-fcd5).
pub(super) fn start(
    deps: &Deps,
    ts: &str,
    workspace: &Path,
    provider: &str,
) -> Result<Reply, String> {
    deps.caller
        .logins
        .start(&deps.world, workspace, provider, &deps.state_root, ts)
        .map(Reply::Login)
}

/// Everything this pair's run has said — the one-frame answer (§8.5).
pub(super) fn standing(deps: &Deps, workspace: &Path, provider: &str) -> Reply {
    Reply::Login(deps.caller.logins.standing(workspace, provider))
}

/// What one look at the run found — [`Lane::poll`]'s answer, and the whole
/// vocabulary a held read has. `Follow`'s own three, at this subject.
pub(crate) enum Frame {
    /// A frame to write: what the standing gained since the last one.
    Ready(LoginView),
    /// Nothing new yet. The hold's own answer, and never an end.
    Waiting,
    /// The stream is over — the run settled, or it stopped being this run.
    Over,
}

/// One sign-in's output, as a frame sequence.
pub(crate) struct Lane {
    runs: Runs,
    workspace: PathBuf,
    provider: String,
    /// How many lines this read has already handed over — the append cursor,
    /// per read and never stored, so a re-ask starts at zero and replays.
    sent: usize,
    opened: bool,
    done: bool,
    waits: u32,
    quiet: u32,
    tick: Duration,
}

impl Lane {
    /// Follow `provider`'s sign-in in `workspace`, on the production hold.
    pub(crate) fn new(runs: Runs, workspace: PathBuf, provider: String) -> Self {
        Self::holding(runs, workspace, provider, HOLD_WAITS, HOLD_TICK)
    }

    /// The same on a stated hold — a test names a short one rather than
    /// sleeping for real ([`Mailbox::holding`](crate::registry::mailbox::Mailbox::holding)'s
    /// own shape).
    pub(crate) fn holding(
        runs: Runs,
        workspace: PathBuf,
        provider: String,
        waits: u32,
        tick: Duration,
    ) -> Self {
        Self {
            runs,
            workspace,
            provider,
            sent: 0,
            opened: false,
            done: false,
            waits,
            quiet: 0,
            tick,
        }
    }

    /// **One look, taken now** — the mechanism, with [`next`](Iterator::next)
    /// only the patience around it, so a test drives this and asserts on frames
    /// with no clock and no sleep.
    pub(crate) fn poll(&mut self) -> Frame {
        if self.done {
            return Frame::Over;
        }
        let first = !std::mem::replace(&mut self.opened, true);
        let Some(view) = self.runs.frame(&self.workspace, &self.provider, self.sent) else {
            // No run stands. Opening on one is the legible emptiness; losing
            // one mid-read is a replacement or a sweep, and this lane is over.
            return if first {
                Frame::Ready(LoginView::default())
            } else {
                Frame::Over
            };
        };
        self.sent += view.lines.len();
        // The settled exit is the last frame, and it carries whatever the run
        // said on its way out — a run that finished between two looks still
        // wrote what it wrote.
        if view.outcome.is_some() {
            self.done = true;
            return Frame::Ready(view);
        }
        if first || !view.lines.is_empty() {
            return Frame::Ready(view);
        }
        Frame::Waiting
    }
}

impl Iterator for Lane {
    type Item = Reply;

    /// The next frame, or the end of the stream. Parks between looks — the
    /// caller is a connection thread and nothing else waits on it — and a
    /// frame resets the quiet count, because writing one is what discovers a
    /// peer that went away.
    fn next(&mut self) -> Option<Reply> {
        loop {
            match self.poll() {
                Frame::Ready(view) => {
                    self.quiet = 0;
                    return Some(Reply::Login(view));
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
