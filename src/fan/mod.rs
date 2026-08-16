//! The **mutating fan** — N ≥ 1 isolated delivery attempts over one delivery
//! obligation (VISION §4.10, bl-2b8c; DESIGN §3.8).
//!
//! §4.10 item 1, verbatim: *"Every write-capable attempt is a balls-materialized
//! private source (ref + index + worktree). The N = 1 ordinary ball path is
//! exactly today's `work/<id>` claim; N > 1 alternatives use the same capability
//! in a namespace distinct from `work/*` (balls bl-4eac) — one mechanism, no
//! special candidate path."* That is this module: [`spread`] is the whole
//! gesture, and `n <= 1` materializes nothing at all — it hands back the
//! ordinary claim binding, which is the general path with N of one, never a
//! branch for a "single" case.
//!
//! **balls owns the names and the paths; yog constructs neither.** The target is
//! asked for ([`Project::target`] — `work/<id>` for a ball, the project's own
//! integration branch for a bare repo, never a literal here), the handle is
//! minted by balls and opaque, and the worktree is placed by balls. yog's whole
//! contribution is **N**, the per-variant overrides the fires carry, and the
//! policy that retires a loser ([`retention`]).
//!
//! **Every attempt of one fan starts at one commit.** balls' [`Attempt::open`]
//! takes an opaque [`Target`](balls::attempt::Target) — a *ref*, resolved per
//! call — so the shared start is not structural upstream; [`open`] therefore
//! proves it, refusing a fan whose members do not report one
//! [`base`](Attempt::base). A cohort is *"attempts sharing (target, base)"*
//! (§4.10 item 6), so members that do not share a base are not a cohort and
//! yog will not present them as one.
//!
//! **Rejection is the absence of a delivery** (§4.10 item 6). Nothing here
//! rejects, marks or scores: a candidate that is never delivered changed no
//! target ref, and its two cleanup steps are *separate* balls calls —
//! [`release`] (the worktree goes, the source ref stays addressable) and
//! [`discard`] (both go). Which one a retirement spends is [`retention`]'s
//! answer, and that policy is world config: deleting the entry deletes a
//! default, not code.
//!
//! **Rework is source-owned and needs nothing here** (§4.10 item 5). A stale
//! candidate is reworked by messaging the agent bound to it
//! ([`Message`](crate::boundary::Action::Message)) to incorporate the current
//! target in its own attempt worktree and redeliver; balls' delivery refuses a
//! stale source before it merges, gates or moves anything (upstream bl-a1a4),
//! and yog never reconciles on an agent's behalf. The absence of a reconcile
//! path in this module is the implementation of that rule.

use std::io;
use std::path::Path;

use crate::start::Prepared;

use balls::attempt::{Attempt, Target};
use balls::delivery_repo::Project;
use balls::layout::Xdg;

pub mod cohort;
pub mod delivery;
pub mod retention;
pub mod spread;
#[cfg(test)]
mod tests;

pub use cohort::{Member, members};
pub use delivery::{Delivery, deliver, delivered_commit};
pub use spread::{Candidate, open, spread};

/// The fan's **two gestures**, as the control boundary carries them (VISION
/// §4.10, DESIGN §3.8/§8.5). One boundary [`Action`](crate::boundary::Action)
/// variant holds this enum rather than two holding its arms — the fold the
/// monitor's and the fleet's families already take, and for the same reasons:
/// one subject (a delivery obligation's candidates), one pair of ends, one
/// trail. Every layer under the boundary was already cut this way and calls
/// them "the fan's two" — `boundary::fan` holds both executors,
/// `boundary::codec::fan` both spellings, `boundary::line::fan` both readers —
/// so the carrier now says what those tables were already saying, and
/// [`action`](crate::boundary::action) is one row wider instead of two.
///
/// Every gesture still carries its whole parameter set, and each still spells
/// as its own slash verb, envelope `op` and help page: the fold is in the
/// carrier, never in the surface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verb {
    /// **Fan one delivery obligation into N isolated candidates** (VISION
    /// §4.10, bl-8746): pin the target once, ask balls for N attempts off that
    /// exact commit, and hand back the same
    /// [`Prepare`](crate::boundary::Action::Prepare) reply once per candidate,
    /// each rebound to its own attempt worktree. Every element is then fired by
    /// the ordinary [`Prompt`](crate::boundary::Action::Prompt) gesture — so
    /// per-variant overrides are the caller's, the §3.5 ceiling gates each
    /// birth exactly as it gates a single start, and the trail carries N
    /// ordinary fire rows rather than one row for N.
    ///
    /// **This is not the fan's group** ([`cohort`]): nothing here names a
    /// cohort, and none is recorded. It is the one act that *must* be one
    /// gesture — N attempts off one pinned target tip cannot be N separate
    /// gestures without losing the shared base that makes them siblings.
    /// `n <= 1` materializes nothing at all and answers with the ordinary claim
    /// binding, which is why there is no separate single-start path.
    Spread {
        prepared: Prepared,
        /// The project and the ball whose `work/<id>` ref is the target — one
        /// value, because a target is both or neither ([`Obligation`]).
        obligation: Obligation,
        n: usize,
    },
    /// **Retire one candidate** (VISION §4.10 items 4 and 6): release its
    /// worktree, and delete its source ref only when this project's retention
    /// policy says the keep has expired ([`retention`]). Two separate balls
    /// calls, never one — a rejected candidate stays inspectable by default,
    /// and a rejection changes no target ref at all, here or anywhere: there is
    /// no reject verb, because rejection is the *absence* of a delivery.
    Retire {
        obligation: Obligation,
        /// balls' opaque attempt handle, as the cohort read it back.
        handle: String,
    },
    /// **Deliver candidate** — never Adopt (VISION §5 V3 item 2): accept one
    /// candidate by the ordinary recursive source-to-target delivery (§4.10
    /// items 5–6, [`delivery::deliver`]). It advances the obligation's own
    /// target — `work/<id>` for a ball, so the ball's later close is the same
    /// operation one level up — and it neither closes that ball nor changes
    /// what its close delivers. There is no reject sibling to this arm: a
    /// rejection is the *absence* of a delivery, and the losers stay
    /// addressable until [`Retire`](Self::Retire) is spent on them.
    Deliver {
        obligation: Obligation,
        /// balls' opaque attempt handle, as the cohort read it back.
        handle: String,
        /// The delivery subject's text (first line only, upstream's rule);
        /// balls tags it `[<handle>]`, which is the acceptance fact
        /// [`delivered_commit`] later derives.
        summary: String,
    },
}

/// The delivery obligation a fan spreads over (§4.10 item 1) — a project repo
/// and, when there is one, the ball whose `work/<id>` ref is the target.
///
/// One value, because the two fields are one fact and every act on a candidate
/// needs both: `open`, `resume`, `release` and `discard` all re-derive the
/// target from it, and a pair that could drift apart would be a candidate
/// delivered onto the wrong ref. `ball` of `None` is the bare project-repo
/// obligation (§4.10 item 8): the target is the integration branch the project
/// itself names, never a literal here.
#[derive(Debug, Clone, PartialEq, Eq)]
/// **`project` is the wire name, not a path** (REMOTE §8, bl-f5f6): an
/// obligation is a boundary datum — it rides in [`Action::Fan`] and
/// [`Action::Retire`] — so it addresses its repo the way every other gesture
/// does. The `repo` every function here takes beside it is that name resolved,
/// once, at the dispatch chokepoint; nothing under this module resolves.
pub struct Obligation {
    pub project: String,
    pub ball: Option<String>,
}

impl Obligation {
    /// This obligation's delivery target in `repo`, asked of balls. Opaque by
    /// construction — yog cannot build one, only ask for one, which is what
    /// makes "callers never spell a ref name" mechanical.
    fn target(&self, repo: &Path) -> io::Result<Target> {
        Project::at(repo).target(self.ball.as_deref())
    }
}

/// Re-materialize one candidate by handle — balls' own crash retry
/// ([`Attempt::resume`]), and the only route from a handle back to a live
/// attempt. An unknown handle is refused upstream rather than quietly minted.
///
/// It is also the only route to the two cleanup calls, which is why a
/// [`discard`] re-materializes the worktree it is about to remove: balls hangs
/// cleanup off a live `Attempt` value and exposes no handle-only door. The act
/// is idempotent either way (create-if-absent, then remove).
fn resume(obligation: &Obligation, repo: &Path, xdg: &Xdg, handle: &str) -> io::Result<Attempt> {
    let target = obligation.target(repo)?;
    Attempt::resume(repo, xdg, &target, handle)
}

/// **Release** one candidate: the worktree goes, the source ref stays. A
/// rejected attempt changed no target ref and remains fully addressable — its
/// diff still reads, its ref still enumerates — which is what "losers stay
/// inspectable" means mechanically (§4.10 item 6).
pub fn release(obligation: &Obligation, repo: &Path, xdg: &Xdg, handle: &str) -> io::Result<()> {
    resume(obligation, repo, xdg, handle)?.release()
}

/// **Discard** one candidate: the worktree *and* the source ref. The attempt is
/// gone and a later [`resume`] of its handle is refused. Spent only when
/// [`retention`] says the retention has expired — balls never sweeps, and yog
/// never discards on an opinion.
pub fn discard(obligation: &Obligation, repo: &Path, xdg: &Xdg, handle: &str) -> io::Result<()> {
    resume(obligation, repo, xdg, handle)?.discard()
}
