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
use std::path::{Path, PathBuf};

use balls::attempt::{Attempt, Target};
use balls::delivery_repo::Project;
use balls::layout::Xdg;

use crate::start::Prepared;

pub mod cohort;
pub mod retention;
#[cfg(test)]
mod tests;

pub use cohort::{Member, members};

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

/// One materialized candidate: the three identities balls returns and yog
/// stores nowhere. The `handle` is opaque — yog binds an agent to it and reads
/// it back off the trail, never parsing meaning out of it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidate {
    /// balls' opaque attempt handle (`at-` + 8 hex), minted off the live set.
    pub handle: String,
    /// The private, single-writer worktree — the fire's `--cwd` binding (§3.3,
    /// bl-6654) and the one directory this candidate may write in.
    pub worktree: PathBuf,
    /// The exact commit this attempt forked from: `merge-base(target, source)`,
    /// derived by balls, never stored. Half of the cohort key.
    pub base: String,
}

impl Candidate {
    /// Read one live [`Attempt`] into the three identities yog carries.
    fn of(attempt: &Attempt) -> Candidate {
        Candidate {
            handle: attempt.handle().to_owned(),
            worktree: attempt.worktree().to_path_buf(),
            base: attempt.base().to_owned(),
        }
    }
}

/// Materialize `n` isolated candidate attempts over one delivery obligation.
///
/// A ball obligation targets the `work/<id>` ref `bl close` already delivers
/// into — so accepting a candidate advances the ball's own branch, and the
/// ball's later close is the same operation one level up (§4.10 item 1).
///
/// The target is resolved **once** and every attempt is opened against that one
/// value; the shared [`base`](Candidate::base) is then proved rather than
/// assumed (see the module note). A fan of `0` materializes nothing, which is
/// the same fold with no inputs.
pub fn open(
    obligation: &Obligation,
    repo: &Path,
    xdg: &Xdg,
    n: usize,
) -> io::Result<Vec<Candidate>> {
    let target = obligation.target(repo)?;
    let mut out = Vec::with_capacity(n);
    for _ in 0..n {
        out.push(Candidate::of(&Attempt::open(repo, xdg, &target)?));
    }
    one_base(out)
}

/// Every member of a fan shares one base, or it is not a fan (§4.10 item 6).
///
/// A divergent base means the target moved between two `Attempt::open` calls,
/// so the members fork from different commits and are not comparable. The
/// refusal is loud and leaves what was materialized addressable — balls never
/// sweeps attempts and neither does this: retiring them is [`retention`]'s
/// call, made by the operator, exactly as it is for a loser.
fn one_base(members: Vec<Candidate>) -> io::Result<Vec<Candidate>> {
    let mut bases: Vec<&str> = members.iter().map(|c| c.base.as_str()).collect();
    bases.sort_unstable();
    bases.dedup();
    if bases.len() > 1 {
        return Err(io::Error::other(format!(
            "the delivery target moved under the fan: its {} attempts report {} different base \
             commits ({}), so they are not siblings — retry the fan",
            members.len(),
            bases.len(),
            bases.join(", "),
        )));
    }
    Ok(members)
}

/// The fan **fire**: one prepared start spent once per candidate, each bound to
/// its own attempt worktree (§4.10 items 1–2).
///
/// `n <= 1` is the ordinary path untouched — the claim's `work/<id>` binding,
/// no attempt materialized, no candidate namespace entered. Above one, each
/// returned [`Prepared`] differs from the given one in exactly its
/// [`binding`](Prepared::binding), which is bl-6654's typed `--cwd` channel: the
/// agent's working-directory mark is seeded at creation, so every tool step of
/// every later turn runs inside that candidate's own worktree and no two
/// write-capable lineages share a mutable checkout (§4.10 item 3).
///
/// The per-variant overrides are the caller's: each returned value is fired by
/// the ordinary [`Prompt`](crate::boundary::Action::Prompt) gesture, so a fan
/// leaves N ordinary fire rows on the §4.2 trail — N committed execution facts,
/// which is what [`cohort`] reads the membership back out of.
pub fn spread(
    prepared: &Prepared,
    obligation: &Obligation,
    repo: &Path,
    xdg: &Xdg,
    n: usize,
) -> io::Result<Vec<Prepared>> {
    if n <= 1 {
        return Ok(vec![prepared.clone()]);
    }
    Ok(open(obligation, repo, xdg, n)?
        .into_iter()
        .map(|c| Prepared {
            binding: Some(c.worktree),
            ..prepared.clone()
        })
        .collect())
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
