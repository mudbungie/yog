//! The **live tail** (DESIGN §7.2, bl-54f7): the focused conversation's open
//! `response.json`, followed at frame cadence instead of at the watcher's.
//!
//! Text did not stream back in while the model was thinking or writing: yog
//! reads directly from the stream-in file in the lernie workspace and shows
//! every character as it lands. The mechanism was already there — the fold, the
//! virtual transcript entry, the `Doing` split — but its **cadence** was the
//! whole §7.2 chain: a watcher poll, a `DirtySet` announcement, the 100 ms
//! debounce, a whole-workspace re-derivation. Characters therefore arrived in
//! clumps at the watcher's rhythm.
//!
//! ## What this is allowed to be
//!
//! This is a deliberate, explicit violation of the no-state-in-memory rule:
//! it is purely display, a dead end. So the accumulated text lives in RAM and is **not** re-derivable. What that
//! licence does **not** touch is the frame: it still does no IO. This is a
//! follower on its own thread ([`Follower::spawn`]) publishing into a cell the
//! frame paints from — the derivation worker's own shape applied to one file.
//!
//! Three rules keep the dead end a dead end:
//!
//! - **Only the fold sees it.** The follower publishes into
//!   [`TailCell`](crate::state::TailCell); the *only* reader is
//!   [`echo::compose`](super::echo::compose), which builds the snapshot the
//!   frame paints (`AppModel::snap`). Every gesture, every §8.5 dispatch and
//!   every machine-facing reply takes `AppModel::derived` instead — the §7.2
//!   partition, *paint reads the fold, gestures read the derivation*. There is
//!   no accessor from the model to the tail, so a second consumer cannot be
//!   added without deleting that absence first.
//! - **Superseded, never merged.** When the step commits, `NNN-<model>.json`
//!   lands and the derivation carries it; the tail is dropped whole at the next
//!   subject change or step advance. The two texts are never reconciled
//!   character-by-character — the committed entry is the truth and this was a
//!   preview of it.
//! - **One subject.** The follower tails the **focused** conversation and
//!   nothing else. Tailing every agent in every workspace at frame rate is the
//!   version of this that burns the machine.
//!
//! ## Following, not re-reading
//!
//! The follower holds the response file's path, a byte offset and the
//! trailing partial line ([`follow`]). Each pass reads only what was appended, folds the complete
//! lines through the one shared parser
//! ([`fold_stream`](crate::git_tree::fold_stream)) and
//! [`absorb`](crate::git_tree::Stream::absorb)s the result — so the cost is the
//! new bytes, not the response, and a 40 KB answer does not get re-parsed sixty
//! times a second as it grows. Partial-write tolerance is structural twice
//! over: the remainder before the last newline is held back, and the parser
//! skips a line it cannot read.
//!
//! Three things reset the accumulator to empty, and they are one rule — *this
//! is a different stream now*: the focus moved, the agent's latest step
//! advanced, or the file shrank (a truncate or a replaced step dir). Nothing
//! here expires on a clock.

use std::path::PathBuf;
use std::sync::Arc;

use super::Snapshot;
use crate::git_tree::Stream;

mod follow;

pub use follow::{FollowThread, Follower};

/// The focused conversation's live stream, as fresh as the last pass — the one
/// value this whole module exists to publish.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiveTail {
    pub ws: PathBuf,
    pub agent: String,
    pub stream: Stream,
}

/// Fold the live tail onto the snapshot a frame paints (§7.2). **The only
/// reader of the tail there is**, and it writes into the one field the
/// derivation fills from the same file — so no seat learns a new vocabulary,
/// and the §11 live mark, the flight strip's `N chars streamed`, the roster
/// preview and the transcript's live rows all quicken together off one
/// assignment.
///
/// A tail for an agent this snapshot does not carry writes nothing: the
/// conversation was deleted or has not been enumerated yet, and inventing a row
/// for it is the pending *echo*'s job (§3.4), not this one's.
pub(super) fn overlay(snap: &mut Snapshot, tail: &LiveTail) {
    let Some(tree) = snap.trees.get_mut(&tail.ws) else {
        return;
    };
    if let Some(agent) = tree.agents.iter_mut().find(|a| a.agent_id == tail.agent) {
        agent.stream = tail.stream.clone();
    }
}

/// The model's side of the hand-off — the ask, the follower, and the read-only
/// window onto the derivation the memos key on. They live beside the tail
/// rather than in [`AppModel`](crate::AppModel)'s root because that root is
/// deliberately declaration-light (§12) and because this *is* the tail's
/// interface: everything the rest of yog may do with it is these three lines.
impl super::AppModel {
    /// The conversation the live tail follows: **the focused one, and only it**
    /// (§7.2). Focus moving is therefore the whole of what retires a tail — the
    /// follower drops its accumulator and opens the new subject's file, and the
    /// conversation just left reverts to the derivation's own fold of the same
    /// bytes, on the sweep's cadence. Nothing is lost by that: the tail was
    /// only ever a preview of what the next derivation commits, and nobody is
    /// watching a conversation they navigated away from at character rate.
    pub(super) fn followed_subject(&self) -> Option<(PathBuf, String)> {
        Some((self.focused_workspace()?, self.focus.agent.clone()?))
    }

    /// The worker's derivation, for the **memos** (§7.2 `SnapMemo`): a memo
    /// caches a read of disk, and the fold adds nothing disk knows — so keying
    /// one on the *rendered* snapshot would rebuild every disk read whenever an
    /// echo or a live character moved, which is bl-e90a's cost restored with a
    /// new trigger. Read-only and `pub(crate)`: this is not a second render
    /// source, it is the invalidation signal for a cache.
    pub(crate) fn derivation(&self) -> &Arc<Snapshot> {
        &self.derived
    }

    /// Take the engine's end of the wire read path (REMOTE §1.2, bl-ae05).
    /// Handed over rather than taken at [`boot`](Self::boot), for
    /// [`follower`](Self::follower)'s reason exactly: the model owns no thread
    /// and mints no handle the engine is the one owner of.
    pub fn adopt_wire(&mut self, link: crate::wire::link::Link) {
        self.wire = link;
    }

    /// Record why this window's wire is absent (bl-dc14), keeping the FIRST
    /// reason: the engine's own bind refusal outranks the "no seat" that
    /// follows from it, and a wired window never records one at all.
    pub fn refuse_wire(&mut self, reason: String) {
        self.wire_refusal.get_or_insert(reason);
    }

    /// Why this window has no wire — `None` on a wired window. The frame
    /// paints this INSTEAD of the shell (`shell::refusal`): every read and act
    /// crosses the wire (REMOTE §1.2), so controls painted without one only
    /// look actionable, which is the inert window bl-dc14 refuses.
    pub fn wire_refusal(&self) -> Option<String> {
        self.wire_refusal.clone()
    }

    /// **Ask the wire** (REMOTE §1.2, §3): declare `question` standing and read
    /// whatever answer has landed for it. Never blocks and never dials — the
    /// [`Asker`](crate::wire::asker::Asker) does both, off-frame, at human
    /// cadence — so a surface built on this paints one cadence period behind
    /// the world and the frame stays at its rate no matter what the engine is
    /// doing.
    pub fn wire_ask(&mut self, question: &serde_json::Value) -> Option<crate::wire::link::Landed> {
        self.wire.ask(question)
    }

    /// Whether any standing question is still unanswered — a **driven** frame's
    /// settle condition and nothing else's (bl-44e9); the window itself never
    /// asks, because a surface paints what it has.
    #[cfg(test)]
    pub fn awaiting(&self) -> bool {
        self.wire.awaiting()
    }

    /// This instance's [`Follower`], for the engine to spawn — the model never
    /// starts its own thread, so a test drives [`Follower::pass`] by hand (the
    /// same reason [`boot`](Self::boot) hands back a `Deriver`).
    pub fn follower(&self) -> Follower {
        Follower::new(Arc::clone(&self.tail))
    }
}

#[cfg(test)]
mod tests;
