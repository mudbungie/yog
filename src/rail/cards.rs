//! One child's card: where it hangs, what its fork point is called, and the
//! tail of what it is saying (VISION V1.4/V1.5).
//!
//! Split from [`super`] at §12's cap on the seam between the spine's *shape*
//! (the types and the notch spine) and the *placement* of a child on it. Every
//! function here is pure over facts the snapshot already carries — no git call
//! reaches this file.
//!
//! **Both of VISION V1.3's edges are computed here and one of them is spent
//! immediately** (bl-1802): the context edge decides the fork label's wording
//! and is then done, because the label is what the chat renders. Keeping its
//! notch index on the card as well would be a second home for a fact the words
//! already carry.

use super::{ChildCard, ChildInput, Notch};
use crate::git_tree::StepCommit;

/// Longest streaming tail a card carries — a card is a glance, not a reader.
const TAIL_CHARS: usize = 140;
/// How many trailing lines of the child's in-flight text the tail keeps
/// (VISION V1.4: "the last line or two").
const TAIL_LINES: usize = 2;

/// One child's card, or `None` when it cannot hang from a notch: a parent with
/// no steps has no rail, and a child whose branch carries no commit past its
/// config lineage has no birth to place.
pub(super) fn card(
    parent_name: &str,
    parent_commits: &[StepCommit],
    notches: &[Notch],
    child: &ChildInput,
) -> Option<ChildCard> {
    notches.first()?;
    let shared = shared_prefix(parent_commits, &child.commits);
    let context = shared
        .checked_sub(1)
        .and_then(|fork_pos| notch_at_or_before(notches, parent_commits, fork_pos));
    let birth = child.commits.get(shared).or_else(|| child.commits.last())?;
    let provenance_notch = notch_by_time(notches, parent_commits, birth.timestamp_unix);
    Some(ChildCard {
        agent_id: child.agent_id.clone(),
        name: child.name.clone(),
        fork: fork_label(parent_name, context.as_ref(), provenance_notch, child),
        state: child.state,
        tokens: child.tokens,
        tail: child.streaming_text.as_deref().and_then(tail_of),
        provenance_notch,
    })
}

/// How many leading commits the two branches share — the fork point's position
/// in the parent's list, and zero for a clean child that shares nothing.
fn shared_prefix(parent: &[StepCommit], child: &[StepCommit]) -> usize {
    parent
        .iter()
        .zip(child)
        .take_while(|(a, b)| a.oid == b.oid)
        .count()
}

/// Where a notch's commit sits in the parent's commit list, or `None` when the
/// notch recorded no commit (or names one off this branch).
fn position(parent_commits: &[StepCommit], notch: &Notch) -> Option<usize> {
    let oid = notch.commit.as_deref()?;
    parent_commits.iter().position(|c| c.oid == oid)
}

/// The last notch whose commit sits at or before `pos` in the parent's list —
/// which notch's read state a fork point falls under — paired with that
/// notch's label, taken while the notch is in hand so no caller looks it up a
/// second time.
fn notch_at_or_before(
    notches: &[Notch],
    parent_commits: &[StepCommit],
    pos: usize,
) -> Option<(usize, String)> {
    notches
        .iter()
        .enumerate()
        .filter(|(_, notch)| position(parent_commits, notch).is_some_and(|at| at <= pos))
        .map(|(index, notch)| (index, notch.short()))
        .next_back()
}

/// The dispatch notch: the last notch whose read-state commit is no later than
/// the child's birth. Floors at the first notch — a child born before any step
/// recorded a commit still hangs from the rail's head rather than nowhere.
fn notch_by_time(notches: &[Notch], parent_commits: &[StepCommit], birth: i64) -> usize {
    notches
        .iter()
        .enumerate()
        .filter(|(_, notch)| {
            position(parent_commits, notch)
                .and_then(|at| parent_commits.get(at))
                .is_some_and(|commit| commit.timestamp_unix <= birth)
        })
        .map(|(index, _)| index)
        .next_back()
        .unwrap_or(0)
}

/// The fork-point label (VISION V1.4). A fork whose context and provenance
/// notches coincide came from the notch the card hangs on — `from here`. One
/// that forked further back names the parent and the oid. A clean child names
/// the config branch it started from, or says plainly that it started from a
/// config commit when no branch still points at one.
fn fork_label(
    parent_name: &str,
    context: Option<&(usize, String)>,
    provenance_notch: usize,
    child: &ChildInput,
) -> String {
    match context {
        Some(&(index, _)) if index == provenance_notch => "from here".to_owned(),
        Some((_, short)) => format!("from {parent_name}@{short}"),
        None => child.config_label.as_ref().map_or_else(
            || "from a config commit".to_owned(),
            |name| format!("from config/{name}"),
        ),
    }
}

/// The card's streaming tail: the last [`TAIL_LINES`] non-blank lines of the
/// child's in-flight text, joined and clipped to [`TAIL_CHARS`]. Text made
/// entirely of blanks has no tail to show.
fn tail_of(text: &str) -> Option<String> {
    let mut lines: Vec<&str> = text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect();
    let keep = lines.len().saturating_sub(TAIL_LINES);
    lines.drain(..keep);
    let joined = lines.join(" ");
    if joined.is_empty() {
        return None;
    }
    Some(joined.chars().take(TAIL_CHARS).collect())
}
