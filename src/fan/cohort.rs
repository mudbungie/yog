//! The **cohort**, derived (VISION §4.10 items 3 and 6; DESIGN §3.8).
//!
//! §4.10 item 6, verbatim: *"There is no Adopt verb and no stored winner: the
//! accepted candidate is the attempt whose delivery the target's history records
//! … cohort = attempts sharing (target, base); provenance is ancestry; rejection
//! is the absence of a delivery."* So there is no fan registry, no membership
//! field and no yog project index here — only a fold over facts yog already
//! wrote down for other reasons.
//!
//! **The join is by pointer** (§4.10 item 4). The pointer is the fire's own
//! `--cwd` argv on the §4.2 trail: yog logged which conversation it bound to
//! which directory, and a directory is a candidate exactly when balls' own
//! [`attempt_path`] formula reproduces it from the handle its leaf names. yog
//! parses no meaning out of the handle and builds no path — it re-derives balls'
//! and compares, so a formula change upstream shows up as an empty cohort rather
//! than as a wrong one.
//!
//! **The trail, never the agent's mark.** lernie's `refs/lernie/cwd/<agent-id>`
//! also names a bound directory, and it is the wrong source for exactly the
//! reason [`crate::control::root`] gives for the writable root: the mark is
//! rewritten by the agent's own `cd`, so a cohort read from it would be a set
//! the candidates could edit themselves. The trail row is yog's own act.

use std::path::{Path, PathBuf};

use balls::delivery_path::attempt_path;
use balls::layout::Xdg;

use crate::opslog::OpEntry;

/// The logical `argv[0]` of a fire row (§8.2's logical-vs-physical argv).
const LERNIE: &str = "lernie";
/// The subcommand a start fires (§8.1).
const PROMPT: &str = "prompt";
/// The minted conversation name's flag (§3.3).
const NAME: &str = "--name";
/// bl-6654's typed work-target binding — the pointer this join follows.
const CWD: &str = "--cwd";

/// One candidate of a cohort, as the trail records it: the conversation yog
/// fired and the attempt it was bound to. Held nowhere — re-derived on each
/// read, like every other §5.1 projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Member {
    /// The minted conversation name the fire passed as `--name` (§3.3).
    pub conversation: String,
    /// balls' opaque attempt handle, recovered from the binding and verified
    /// against balls' own path formula.
    pub handle: String,
    /// The candidate's private worktree — the binding itself.
    pub worktree: PathBuf,
}

/// Every candidate of `project` that a fire in `workspace` bound an agent to,
/// oldest first, one member per attempt (a re-fire onto one candidate is one
/// candidate — the **last** row wins, the same rule the writable root's claim
/// join keeps).
///
/// A workspace that only ever started ordinary N = 1 balls has no members: its
/// fires bound `work/<id>` worktrees, which balls' attempt formula does not
/// reproduce. That is the derivation stating "no fan here", not an empty
/// special case.
pub fn members(entries: &[OpEntry], xdg: &Xdg, project: &Path, workspace: &Path) -> Vec<Member> {
    let here = workspace.to_string_lossy();
    let mut out: Vec<Member> = Vec::new();
    for member in entries
        .iter()
        .filter(|e| e.cwd == here)
        .filter_map(|e| member_of(e, xdg, project))
    {
        out.retain(|m| m.handle != member.handle);
        out.push(member);
    }
    out
}

/// The candidates alone — the writable-root question (§4.11 item 3), which asks
/// *where* and never *who*.
pub fn worktrees(entries: &[OpEntry], xdg: &Xdg, project: &Path, workspace: &Path) -> Vec<PathBuf> {
    members(entries, xdg, project, workspace)
        .into_iter()
        .map(|m| m.worktree)
        .collect()
}

/// The member one trail row names, when it is a fire bound to a candidate of
/// `project`. Any other row — a `bl` verb, an unbound fire, a fire bound to the
/// ordinary `work/<id>` claim — contributes nothing.
fn member_of(entry: &OpEntry, xdg: &Xdg, project: &Path) -> Option<Member> {
    let argv: Vec<&str> = entry.argv.iter().map(String::as_str).collect();
    let [bin, verb, rest @ ..] = argv.as_slice() else {
        return None;
    };
    if *bin != LERNIE || *verb != PROMPT {
        return None;
    }
    let worktree = PathBuf::from(flag(rest, CWD)?);
    Some(Member {
        conversation: flag(rest, NAME)?,
        handle: handle_of(xdg, project, &worktree)?,
        worktree,
    })
}

/// The value following `name` in an argv tail, when it is there. Owned on the
/// way out (rule 1): the tail is borrowed, elided, and what comes back names
/// nothing of it.
fn flag(argv: &[&str], name: &str) -> Option<String> {
    argv.windows(2)
        .find(|w| w.first() == Some(&name))
        .and_then(|w| w.get(1))
        .map(|value| (*value).to_owned())
}

/// The handle `bound` belongs to, iff balls' own [`attempt_path`] over that
/// handle reproduces `bound` exactly. The leaf is a guess; the reproduction is
/// the proof — which is what keeps the path formula balls' single fact.
fn handle_of(xdg: &Xdg, project: &Path, bound: &Path) -> Option<String> {
    let handle = bound.file_name()?.to_str()?;
    let derived = attempt_path(xdg, &project.to_string_lossy(), handle);
    (derived == bound).then(|| handle.to_owned())
}

#[cfg(test)]
mod tests;
