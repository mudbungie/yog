//! **Accepted, rejected, reworked** — the projection's outcome, derived from
//! git and nothing else (VISION §4.10 items 5–7; §3.9, bl-40ab).
//!
//! Nothing is stored, no verb writes any of these, and there is no winner field
//! to read: *"the accepted candidate is the attempt whose delivery the target's
//! history records; … provenance is ancestry; rejection is the absence of a
//! delivery"* (§4.10 item 6). Three facts answer the whole enumeration:
//!
//! - **the derived acceptance mark** — the `[<id>]` tag-scan
//!   ([`crate::fan::delivered_commit`]) the work-diff row already wears;
//! - **whether the source ref still resolves** — the diff row's own
//!   [`Change::Absent`] naming it, which is what a discarded attempt reads as;
//! - **ancestry** — whether the source already contains the target
//!   ([`crate::git_tree::is_ancestor`], the §9.3 fold's own read spent one repo
//!   over: a second spelling of `merge-base --is-ancestor` would be two places
//!   for one git command to drift).
//!
//! **The rework test is the delivery law's own precondition, read backwards —
//! and that is a reframe of the ball's wording** (bl-40ab said *"the source
//! advanced after a refusal or verdict"*). A clock-based reading needs two
//! clocks yog does not share: the §4.2 trail counts unix seconds, a `messages/`
//! entry orders by a filename counter, and the source's own advance is a commit
//! date in a third repo. What that reading is *for* is knowing whether a
//! superseded attempt can deliver again — and §4.10 item 5 already says exactly
//! what that means: *"every sibling is stale by construction and must rework
//! (incorporate the new target in its own worktree) before it can deliver"*. So
//! reworked **is** "the target is an ancestor of the source", which is the very
//! test balls' delivery makes before it merges anything. One git question,
//! exact, no clock — and it stays true of a rework the operator did by hand,
//! which a trail-ordering test would have missed.
//!
//! **A refusal is the occasion for a rework, never its evidence.** Only the
//! incorporation is evidence, and only the incorporation is derivable.
//!
//! **The base commit lives here too, and it is not an outcome.** §4.10 item 7
//! names three OIDs beside the delivered one, and the third — the commit the two
//! ends departed from — is [`base`]. It sits beside [`reworked`] because it is
//! the same question in the same shape: one ancestry read over this row's own
//! two ends, degrading the same way (no resolved pair, or no locatable repo, is
//! no claim). Item 6 calls balls the authority for it, and balls' authority is a
//! *formula* — *"merge-base(target, source), derived, never stored"* — which yog
//! spells itself, because the only way to ask balls is to resume the attempt and
//! that re-materializes a worktree. A read must not write. The git call is
//! [`crate::git_tree::merge_base`], the same one the config fold spends.

use std::path::Path;

use crate::workdiff::{Attempt, Change};

/// What became of one attempt (§4.10 items 5–7).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// The target's history records **this** attempt's delivery, at this
    /// commit. The only acceptance there is.
    Accepted { commit: String },
    /// This attempt's delivery never happened and something else's did — `by`
    /// names the sibling whose delivery the target records, or is `None` when
    /// the attempt was discarded outright and its source ref is gone. Both are
    /// rejection in §4.10 item 6's sense, which is the *absence* of a delivery
    /// and never a mark.
    Rejected { by: Option<String> },
    /// Rejected above, and then reworked: the source has incorporated the
    /// target the sibling advanced, so balls' delivery would no longer refuse it
    /// as stale. A live attempt again, which is why it is not `Rejected`.
    Reworked,
    /// No delivery of this attempt, and no sibling's either — the ordinary
    /// standing of work in progress. It is the absence of the three above
    /// rather than a fourth policy, and naming it is what keeps them
    /// statements: an attempt yog cannot say anything about must not read as
    /// rejected.
    Pending,
}

/// The outcome of `attempt`, given every attempt this workspace holds and the
/// project repo to ask ancestry of. `siblings` is the whole row set — cohort
/// membership is derived from it below rather than being a set anything keeps
/// (§4.10 item 6: *"cohort = attempts sharing (target, base)"*).
pub(super) fn of(attempt: &Attempt, siblings: &[Attempt], repo: Option<&Path>) -> Outcome {
    if let Some(commit) = &attempt.delivered {
        return Outcome::Accepted {
            commit: commit.clone(),
        };
    }
    let landed = accepted_sibling(attempt, siblings);
    if landed.is_none() && !discarded(attempt) {
        return Outcome::Pending;
    }
    if reworked(attempt, repo) {
        return Outcome::Reworked;
    }
    Outcome::Rejected { by: landed }
}

/// The handle of a sibling whose delivery this attempt's target records.
///
/// **Siblings share the target, not just the ball.** A fan's candidates and the
/// claim they were fanned off all wear the obligation's ball id, and they are
/// not one cohort: the candidates target `work/<id>` and the claim targets the
/// branch that ball closes into, so *"attempts sharing (target, base)"* is a
/// comparison of targets and a ball-only test would call the claim's own close
/// a sibling's win.
fn accepted_sibling(attempt: &Attempt, siblings: &[Attempt]) -> Option<String> {
    let target = target_of(attempt)?;
    siblings
        .iter()
        .filter(|other| other.handle != attempt.handle)
        .filter(|other| other.project == attempt.project && other.ball_id == attempt.ball_id)
        .filter(|other| target_of(other) == Some(target))
        .find_map(|other| {
            other.delivered.is_some().then(|| {
                other
                    .handle
                    .clone()
                    .unwrap_or_else(|| other.ball_id.clone())
            })
        })
}

/// This attempt's own **source ref is gone** — the retirement that discarded it
/// ([`crate::fan::discard`]). The diff row already says so by naming the source
/// among the refs that did not resolve, so this is a read of that answer and not
/// a second probe of the repo.
fn discarded(attempt: &Attempt) -> bool {
    match &attempt.change {
        Change::Absent {
            source, missing, ..
        } => missing.contains(source),
        _ => false,
    }
}

/// Whether the source has incorporated the target — the delivery law's own
/// staleness test. `false` for a row with no resolved pair to compare, for a
/// project that cannot be located, and for a repo that will not answer: none of
/// the three is a claim that a rework happened.
fn reworked(attempt: &Attempt, repo: Option<&Path>) -> bool {
    let Change::Diff {
        target_oid,
        source_oid,
        ..
    } = &attempt.change
    else {
        return false;
    };
    repo.is_some_and(|repo| {
        crate::git_tree::is_ancestor(repo, target_oid, source_oid).unwrap_or(false)
    })
}

/// The commit this attempt's two ends departed from — half of item 6's cohort
/// key, and the frozen starting point of the experiment. `None` for a row with
/// no resolved pair, for a project that cannot be located, and for two ends with
/// no shared ancestor: all three are "there is no commit to name", never a guess
/// at one.
pub(super) fn base(attempt: &Attempt, repo: Option<&Path>) -> Option<String> {
    let Change::Diff { target, source, .. } = &attempt.change else {
        return None;
    };
    crate::git_tree::merge_base(repo?, target, source)
        .ok()
        .flatten()
}

/// The target ref this row was read at, when it has one.
fn target_of(attempt: &Attempt) -> Option<&str> {
    match &attempt.change {
        Change::Unreadable => None,
        Change::Absent { target, .. } | Change::Diff { target, .. } => Some(target),
    }
}
