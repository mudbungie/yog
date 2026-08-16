//! The fan's **materializing half** (VISION §4.10 items 1–3; DESIGN §3.8):
//! open N isolated candidates over one obligation, prove they share one base,
//! and rebind a prepared start once per candidate. Split from the family's
//! carrier ([`super`]) at §12's budget when the delivery arm arrived
//! (bl-c2bd) — what a fan *is* stays in `mod`; how one is spread lives here.

use std::io;
use std::path::{Path, PathBuf};

use balls::attempt::Attempt;
use balls::layout::Xdg;

use crate::start::Prepared;

use super::Obligation;

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
/// sweeps attempts and neither does this: retiring them is
/// [`retention`](super::retention)'s call, made by the operator, exactly as it
/// is for a loser.
pub(super) fn one_base(members: Vec<Candidate>) -> io::Result<Vec<Candidate>> {
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
/// which is what [`cohort`](super::cohort) reads the membership back out of.
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
