//! **The tool window on the follow lane** (REMOTE §5.5, bl-5305): which calls
//! this read has already spoken about, and therefore what a look owes the seat.
//!
//! The lane carried the model's prose and nothing else, which is defensible for
//! a conversation editing its own worktree — the work is on disk and a diff is a
//! read away — and wrong for one administering a MACHINE, because the box is the
//! thing an operator cannot inspect afterwards and the tool call is the only
//! record that it was touched. The transcript held every capture in full the
//! whole time; it was the LIVE view that dropped them, which is the view an
//! operator has open precisely when a command they did not expect is about to
//! run on their server.
//!
//! **What a record says is [`tool_event`](crate::git_tree::tool_event)'s**, the
//! way what a response file says is [`Stream`](crate::git_tree::Stream)'s. This
//! is the lane's half — [`Open`](super::open::Open)'s counterpart for the tool
//! subtree: an offset into a file there, a per-call watermark here, both minted
//! per held connection and both starting at zero, so a connection that dropped
//! re-asks and is answered the whole window rather than a suffix nobody can
//! address.
//!
//! **A frame is an append here too** (REMOTE §5.5's rule for the prose half,
//! unamended): the events of a read concatenate, so a follower absorbs them in
//! order onto an empty list, and the one-shot answer an intake that cannot hold
//! a connection gets is that same concatenation taken in a single look.
//! Nothing needs a flag saying which kind of answer it is, exactly as nothing
//! does for the fold.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use crate::control::hold::Held;
use crate::git_tree::{ToolEvent, tool_event};

/// What this read has already said about each call: `false` once it reported
/// the call posted, `true` once it reported the capture.
#[derive(Default)]
pub(super) struct Window {
    said: BTreeMap<String, bool>,
    /// The parks already reported, by `tool_use` id ([`Window::parked`]). Its
    /// own set rather than a third value in `said`, because a park is not a
    /// point on the posted→captured ladder: a held call has landed no
    /// `input.json` at all, and if the operator passes it the ordinary two
    /// events follow under this same id.
    parks: BTreeSet<String>,
}

impl Window {
    /// The events this look discovers under `step_dir`, oldest call first.
    ///
    /// A call seen for the first time yields its opening event, and its closing
    /// one too when the capture is already down — a look slower than the tool
    /// is the ordinary case, and reporting an opening the operator can no
    /// longer act on *without* the status it earned would be worse than not
    /// reporting it. A call already reported closed yields nothing ever again.
    pub(super) fn look(&mut self, step_dir: &Path) -> Vec<ToolEvent> {
        let mut fresh = Vec::new();
        for (tool_use, dir) in tool_event::calls(step_dir) {
            let captured = tool_event::captured(&dir);
            match self.said.get(&tool_use) {
                // Both halves said already: nothing this call can add.
                Some(true) => continue,
                // Opening said, capture not down: still nothing new.
                Some(false) if !captured => continue,
                Some(false) => {}
                None => fresh.push(tool_event::posted(&tool_use, &dir)),
            }
            if captured {
                fresh.push(tool_event::complete(&tool_use, &dir));
            }
            self.said.insert(tool_use, captured);
        }
        fresh
    }

    /// **The park, said once** (bl-58bb): the capability control's hold as this
    /// lane's third transition, the first time this read sees it.
    ///
    /// It comes off the published row rather than off the step directory,
    /// because the seam parks the invocation *before* the executor is entered
    /// and so the window's two files say nothing about it — which is the whole
    /// defect. A conversation waiting on the operator was reported by this lane
    /// as at rest, and the operator watching it is the blocker.
    pub(super) fn parked(&mut self, held: &Held) -> Option<ToolEvent> {
        self.parks
            .insert(held.tool_use_id.clone())
            .then(|| tool_event::parked(held))
    }
}

/// Every event of one step, taken in one look — the answer the chokepoint
/// gives an intake that cannot hold a connection, which is this lane's general
/// path with one frame rather than a degraded reading of it.
pub(crate) fn whole(step_dir: &Path, held: Option<&Held>) -> Vec<ToolEvent> {
    let mut window = Window::default();
    let mut out = window.look(step_dir);
    out.extend(held.and_then(|held| window.parked(held)));
    out
}

#[cfg(test)]
mod tests;
