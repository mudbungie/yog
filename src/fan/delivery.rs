//! **Deliver candidate** — the fan's acceptance, and the derived outcome that
//! renders it (VISION §5 V3 items 2 and 3; §4.10 items 5–6; bl-c2bd).
//!
//! §4.10 item 5, verbatim: *"Accepting a fan candidate is this same delivery;
//! after one lands, every sibling is stale by construction and must rework
//! (incorporate the new target in its own worktree) before it can deliver —
//! sequential synthesis falls out of the law instead of needing a primitive."*
//! So [`deliver`] adds nothing to balls' one delivery law
//! ([`Attempt::deliver`]): the target must already be incorporated (a stale
//! source refuses before anything merges, gates or moves), the repo's own
//! `pre-commit` hook gates the exact source tree, the squash lands tagged
//! `[<handle>]`, and the target CAS-advances. yog names the candidate and
//! writes the summary; everything else is upstream's.
//!
//! **Acceptance is never a stored mark** (§4.10 item 6). The accepted candidate
//! is the attempt whose delivery the target's history records, so
//! [`delivered_commit`] is a *read* — the `[<handle>]` tag-scan over the target
//! ref, the same tag balls' own retry-standing greps for — and rejection is the
//! absence of a delivery: no verb spells it, nothing marks it, and a loser's
//! ref stays addressable until [`retention`](super::retention) expires it.

use std::io;
use std::path::Path;

use balls::delivery_path::marker;
use balls::layout::Xdg;

use crate::git_tree::log_marker;

use super::Obligation;

/// The identities one delivery acted on, as yog carries them — balls'
/// `Delivered`, restated field-for-field so the boundary's [`Reply`]
/// (crate::boundary::Reply) does not surface an upstream type. Every field is a
/// value the delivery already computed; yog stores none of them.
///
/// The two `Option`s mean one thing between them, upstream's own words: *"the
/// target already contained everything the source had"* — `source: None` is a
/// source ref that was never made, `commit: None` an empty or fully-merged
/// source. A converged retry answers the standing delivery commit, because
/// provenance wants the commit that exists.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Delivery {
    /// The target ref this delivery advanced (a branch name).
    pub target: String,
    /// The pinned target tip the delivery validated against.
    pub base: String,
    /// The source tip at delivery, when the source ref existed.
    pub source: Option<String>,
    /// The delivery commit, when one landed.
    pub commit: Option<String>,
}

impl Delivery {
    /// Read balls' answer into the four identities yog carries.
    fn of(delivered: balls::delivery::Delivered) -> Delivery {
        Delivery {
            target: delivered.target,
            base: delivered.base,
            source: delivered.source,
            commit: delivered.commit,
        }
    }
}

/// **Deliver one candidate onto its obligation's target** (VISION V3.2): the
/// ordinary recursive source-to-target delivery, by handle. `summary` becomes
/// the delivery subject, tagged `[<handle>]` by balls itself — the tag
/// [`delivered_commit`] later reads acceptance back out of.
///
/// It neither closes the obligation's ball nor changes which branch that
/// ball's later close delivers: a ball obligation's target is `work/<id>`,
/// so acceptance advances the ball's own branch and the ball's close is the
/// same operation one level up (§4.10 item 1).
pub fn deliver(
    obligation: &Obligation,
    repo: &Path,
    xdg: &Xdg,
    handle: &str,
    summary: &str,
) -> io::Result<Delivery> {
    let attempt = super::resume(obligation, repo, xdg, handle)?;
    Ok(Delivery::of(attempt.deliver(summary, None)?))
}

/// The delivery commit `target`'s history records for the attempt `id` — the
/// derived acceptance mark (§4.10 item 6), and the only kind there is. `id` is
/// an attempt handle or a ball id: both deliver under the same `[<id>]` subject
/// tag, so one scan reads both.
///
/// `None` is *"this target records no such delivery"*, which covers the
/// pending candidate, the rejected one, and a target ref that no longer
/// resolves — in every case there is no delivery to point at, and whether the
/// ref itself is readable is the diff surface's own answer
/// ([`crate::workdiff::Change`]), not this one's.
pub fn delivered_commit(repo: &Path, target: &str, id: &str) -> Option<String> {
    log_marker(repo, target, &marker(id)).ok().flatten()
}
