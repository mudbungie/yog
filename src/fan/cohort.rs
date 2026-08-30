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
//! **The trail, never the agent's mark.** litany's `refs/litany/cwd/<agent-id>`
//! also names a bound directory, and it is the wrong source for exactly the
//! reason [`crate::control::root`] gives for the writable root: the mark is
//! rewritten by the agent's own `cd`, so a cohort read from it would be a set
//! the candidates could edit themselves. The trail row is yog's own act.

use std::path::{Path, PathBuf};

use balls::delivery_path::attempt_path;
use balls::layout::Xdg;

use crate::opslog::OpEntry;

/// The logical `argv[0]` of a fire row (§8.2's logical-vs-physical argv).
const LITANY: &str = "litany";
/// The subcommand a start fires (§8.1).
const PROMPT: &str = "prompt";
/// The minted conversation name's flag (§3.3).
const NAME: &str = "--name";
/// bl-6654's typed work-target binding — the pointer this join follows.
const CWD: &str = "--cwd";
/// §3.7's instruction freeze, one per frozen document (bl-aa8b).
const PIN: &str = "--pin";

/// One **bound fire** as the trail records it (§3.9, bl-40ab): the
/// conversation yog minted, the directory it was bound to, and the instruction
/// documents that fire froze.
///
/// It is the row [`members`] filters and the row the science projection joins
/// on, which is why the argv is parsed once here rather than twice: a fire's
/// `--name`/`--cwd`/`--pin` triple is one reading of one row, and the two
/// consumers differ only in what they then ask of the directory — the cohort
/// asks whether it is an *attempt* ([`handle_of`]), the projection asks which
/// attempt of any kind it is (`science::bound`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fire {
    /// The minted conversation name the fire passed as `--name` (§3.3).
    pub conversation: String,
    /// The typed work-target binding the fire passed as `--cwd` (bl-6654).
    pub worktree: PathBuf,
    /// The `<dest>=<src>` specs the fire froze onto its dispatch commit (§3.7)
    /// — *"every pin survives into the trail and IS the provenance record"*, so
    /// this list is the frozen-input column and not a second copy of one.
    pub pins: Vec<String>,
}

/// Every bound fire in `workspace`, oldest first — the one parse of the §4.2
/// trail's fire rows. A row that is not a `litany prompt`, or one that named no
/// `--cwd`, contributes nothing: it bound no work target, so there is no
/// attempt for it to be about.
pub fn fires(entries: &[OpEntry], workspace: &Path) -> Vec<Fire> {
    let here = workspace.to_string_lossy();
    entries
        .iter()
        .filter(|e| e.cwd == here)
        .filter_map(fire_of)
        .collect()
}

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
    let mut out: Vec<Member> = Vec::new();
    for member in fires(entries, workspace)
        .into_iter()
        .filter_map(|fire| member_of(fire, xdg, project))
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

/// The bound fire one trail row names. Any other row — a `bl` verb, a fire
/// that bound no work target — contributes nothing.
fn fire_of(entry: &OpEntry) -> Option<Fire> {
    let argv: Vec<&str> = entry.argv.iter().map(String::as_str).collect();
    let [bin, verb, rest @ ..] = argv.as_slice() else {
        return None;
    };
    if *bin != LITANY || *verb != PROMPT {
        return None;
    }
    Some(Fire {
        conversation: flag(rest, NAME)?,
        worktree: PathBuf::from(flag(rest, CWD)?),
        pins: flags(rest, PIN),
    })
}

/// The member one bound fire names, when its directory is a candidate of
/// `project`. A fire bound to the ordinary `work/<id>` claim contributes
/// nothing here — that is the projection's row, not the cohort's.
fn member_of(fire: Fire, xdg: &Xdg, project: &Path) -> Option<Member> {
    Some(Member {
        handle: handle_of(xdg, project, &fire.worktree)?,
        conversation: fire.conversation,
        worktree: fire.worktree,
    })
}

/// The value following `name` in an argv tail, when it is there. Owned on the
/// way out (rule 1): the tail is borrowed, elided, and what comes back names
/// nothing of it.
fn flag(argv: &[&str], name: &str) -> Option<String> {
    flags(argv, name).into_iter().next()
}

/// **Every** value following `name`, in argv order — the repeating-flag
/// reading, which `--pin` needs and which [`flag`] is the first element of. One
/// scan, so a flag that may repeat and one that may not are read by one rule.
fn flags(argv: &[&str], name: &str) -> Vec<String> {
    argv.windows(2)
        .filter(|w| w.first() == Some(&name))
        .filter_map(|w| w.get(1))
        .map(|value| (*value).to_owned())
        .collect()
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
