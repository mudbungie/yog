//! The **flag** — VISION §4.9's signal-out verb, and the §6 signal it raises
//! (bl-7aef, joined to attention by bl-6f2f).
//!
//! A flag is not a check. A monitor row is a verdict about a sha; a flag is
//! *"a human should look at this"*, and anyone granted the verb may write one —
//! which is why it has its own pseudo-binary, its own row shape and, since
//! bl-6f2f, its own file beside [`super::row`]'s.
//!
//! **The defect this closes.** `/flag` wrote one exit-0 ops row and stopped
//! there, while VISION §4.9's ladder table, `Reply::Flagged`'s doc and the
//! gesture's own help all promise *"attention item + ops row"*. §6's predicate
//! reads `refs/litany/*` marks and the inbox listing and **no ops row**, so the
//! two halves were never joined: the verb that exists so the machine can raise
//! its hand raised it where nobody looks. That matters beyond the verb —
//! `flag` is the alignment monitor's **floor grant**, so a responder wired as
//! "a pure judge" signalled into silence.
//!
//! **Why the fact rides the agent.** Every other §6 signal is something the
//! agent carries, and that is the predicate's shape, not an accident of it:
//! [`attention`](crate::attention::attention) and
//! [`evidence`](crate::attention::evidence) both take `&Agent` and nothing
//! else. A seventh signal reached by a seventh parameter would have to be
//! threaded through the rank sort, both rollups, the roster walk and every
//! caller of each — six signatures to say one thing — and the acknowledgement
//! path could not follow it at all. So the flag becomes a fact on the agent,
//! folded on by the one place the ops trail and the derived trees are both
//! final ([`fold`], called from the snapshot's publish), exactly as the §7.2
//! live-tail follower supersedes a rendered agent's stream.
//!
//! **The watermark is the row's timestamp.** Rules 1–4 acknowledge an oid and
//! re-fire when the ref moves; a flag has no ref, and its `ts` behaves
//! identically — `/seen` records it, and the next flag is a new stamp and a new
//! firing. Nothing is stored: the row is the home, this is a query over it.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::git_tree::GitTree;
use crate::opslog::{OpEntry, OpRow, Origin};

/// `argv[0]` of a **flag** row (VISION §4.9, bl-7aef): an attention item raised
/// on one conversation, with its reason. Its own pseudo-binary rather than a
/// monitor row, because it asserts something different — a monitor row is a
/// verdict about a sha, a flag is "a human should look at this" — and because
/// anyone granted the verb may write one, not only the check.
pub const YOG_FLAG: &str = "yog-flag";

/// One raised flag as the agent carries it: **when** it was raised — the §6
/// watermark evidence — and **why**, in the raiser's own words.
///
/// The value, not a bare stamp, on `held`'s precedent (§8.6): what an operator
/// needs from a signal that says "look at this" is what the raiser said about
/// it, and a second query to recover the sentence would be a second query to
/// forget.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Flag {
    pub at: String,
    pub reason: String,
}

/// The ops entry a flag appends. `argv[1]` is the conversation, `stdout` the
/// reason, exit `0`: raising attention is not a failure, and must not banner as
/// one (§7.3's ichor is for actions that failed).
pub fn raised(ts: String, workspace: &Path, agent: &str, reason: &str) -> OpEntry {
    OpEntry {
        ts,
        argv: vec![YOG_FLAG.to_owned(), agent.to_owned()],
        cwd: crate::nav::ws_key(workspace),
        exit: 0,
        stdout: super::row::clip(reason),
        stderr: String::new(),
        // The subject is a conversation, which is what §7.3 attribution names —
        // not the surface the hand that raised it happened to be on.
        origin: Origin::Conversation,
    }
}

/// The newest flag on one `(workspace-key, agent)` in a durable tail, or `None`
/// when that conversation carries none. File order, so the last match wins —
/// [`super::row::latest`]'s own reading one noun over.
///
/// Reading is forgiving, like every other `ops.jsonl` read: a hand-mangled row,
/// or one from a future yog with more argv, simply is not a flag.
pub fn latest(rows: &[OpRow], workspace: &str, agent: &str) -> Option<Flag> {
    rows.iter()
        .rev()
        .find(|r| r.cwd == workspace && is_flag_on(&r.argv, agent))
        .map(|r| Flag {
            at: r.ts.clone(),
            reason: r.stdout.clone(),
        })
}

/// Is this row's pre-joined `argv` a flag raised on `agent`? [`OpRow`] joins
/// `argv` for display and both fields a flag row puts there are space-free by
/// construction (the pseudo-binary and an agent id), so the split is lossless.
fn is_flag_on(argv: &str, agent: &str) -> bool {
    let mut words = argv.split(' ');
    words.next() == Some(YOG_FLAG) && words.next() == Some(agent)
}

/// Stamp every derived agent with its newest flag — the one place the ops trail
/// and the derived trees are both final (the snapshot's publish).
///
/// Consumes and returns the map rather than mutating a borrow, because the
/// caller is already paying a clone to freeze the snapshot and folding into
/// that clone costs nothing more. A world with no flag at all walks the rows
/// once and finds nothing, which is the ordinary case.
pub(crate) fn fold(
    mut trees: HashMap<PathBuf, GitTree>,
    rows: &[OpRow],
) -> HashMap<PathBuf, GitTree> {
    if !rows.iter().any(|r| r.argv.starts_with(YOG_FLAG)) {
        return trees;
    }
    for (path, tree) in &mut trees {
        let key = crate::nav::ws_key(path);
        for agent in &mut tree.agents {
            agent.flagged = latest(rows, &key, &agent.agent_id);
        }
    }
    trees
}

#[cfg(test)]
mod tests;
