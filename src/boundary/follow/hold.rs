//! **The bounded hold** (REMOTE §3, §5.5): what one look at this lane can
//! find, how long a quiet read waits before it lets the seat re-ask, and the
//! parking that turns [`Follow::poll`](super::Follow::poll) into a frame
//! sequence.
//!
//! Its own file beside the readers rather than inside them, on the seam
//! [`super`]'s own doc draws: `poll` is the **mechanism** — one look at the
//! world, taken now, with no clock and no sleep in it, which is why a test can
//! drive the whole lane look by look — and everything here is the *patience*
//! around it. Splitting them keeps that separation a fact of the tree rather
//! than a sentence about it.

use std::time::Duration;

use super::{Follow, Reply};
use crate::git_tree::Stream;

/// How long a quiet follow read holds before it ends the stream and lets the
/// seat re-ask: 1875 looks, 16 ms apart — thirty seconds, the mailbox hold's
/// own bound. The tick is the §7.2 follower's own period, which is what "at
/// write cadence" means in a number: half the §11 pulse, so a repaint that was
/// going to happen carries the newest bytes rather than the previous look's.
///
/// `pub(super)` because the attention lane holds on the **same** bound (REMOTE
/// §14.1: "the follow lane's bounded-hold discipline applies unamended") — one
/// pair of numbers, not two that can drift apart.
pub(in crate::boundary) const HOLD_WAITS: u32 = 1875;
pub(in crate::boundary) const HOLD_TICK: Duration = Duration::from_millis(16);

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
                    return Some(Reply::Follow(crate::boundary::reply::FollowFrame {
                        stream,
                        tools,
                    }));
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
