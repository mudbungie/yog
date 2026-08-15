//! Workspace deletion — the §3.6 unmaking (DESIGN §3.6, the §8.1 planner idiom,
//! the §8.2 verb row).
//!
//! Deletion is the raise's inverse at the raise's altitude: raising is `mkdir -p`
//! the names root + `lernie new` under the operator's chosen name (§3.1, §3.4),
//! so unmaking is the release of the
//! sphere's live claims followed by removal of the workspace directory — a write
//! to yog's **own** names root, never inside a workspace (I2 stands untouched).
//! Because the dir's existence *is* the registration (§3.1), the removal is the
//! de-registration; there is no registry to update.
//!
//! Two pure halves live here and one effectful one in [`exec`]:
//!
//! - [`Confirmation`] — what the §3.6 dialog states and what the plan is built
//!   from: the conversations that die (by display name), the bound balls it
//!   releases (by id), and the **gate** — the live conversations, the §10 "?"
//!   uncertainty counting as live (fail closed). One derived object drives the
//!   dialog, the refusal and the plan, so the operator confirms exactly what
//!   runs.
//! - [`plan`] — the ordered [`Step`] sequence. Releases first, removal last, so
//!   a crash anywhere leaves a live workspace with some claims released
//!   (benign, re-runnable), never a removed workspace with steps unfinished.
//!   Convergent on re-run: the released balls have left the join, so a re-plan
//!   is the shorter remainder.
//!
//! **Arming is by naming the object** (§3.6's confirmation doctrine, normative
//! for every future destructive verb): [`Confirmation::armed`] is true only when
//! the operator has typed the workspace's own name. Obscurity is not the safety
//! mechanism; the typed name is — and the verb takes **no keyboard binding,
//! ever**, so no reflex can reach it.

use crate::nav::convs::Conversation;
use std::path::{Path, PathBuf};

pub mod agent;
mod exec;
pub use exec::{DELETE_STEP, DeleteError, execute};

/// One live bound ball the unmaking releases (§3.6 step 1) — the ball's id and
/// the §5.1 #1 **name** of the project whose store holds it.
///
/// A name rather than the `bl` cwd since bl-b4b5, for the reason the join row
/// it is copied off carries one (REMOTE §8.1): a confirmation is what a *seat*
/// states, and a seat holding an answer must be able to build the same object
/// the engine gates on. [`plan`] resolves it back to the clone the verb runs
/// in, at the one seam that owns the round trip.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Claim {
    pub project: String,
    pub id: String,
}

/// What the §3.6 dialog states and the plan runs — derived, never stored: the
/// workspace's name, every conversation that dies (by display name), the live
/// ones (the gate), and the claims the unmaking releases.
///
/// **It no longer carries the workspace's path** (bl-b4b5). The path was the
/// address the gesture had already named, spelled a second time as an
/// engine-side directory — §8.1's own test — and it made the object
/// unbuildable by a seat that reads over a wire. [`plan`] takes the path from
/// the caller that resolved the address, which is where it already was.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Confirmation {
    pub name: String,
    pub conversations: Vec<String>,
    /// The conversations that hold (or may hold) a driver — the §3.6 gate. Empty
    /// ⇒ the verb may proceed; non-empty ⇒ it refuses, naming them so the
    /// operator stops them first. Stop keeps its own semantics; verbs stay
    /// orthogonal (no kill is folded into the delete).
    pub live: Vec<String>,
    pub claims: Vec<Claim>,
}

impl Confirmation {
    /// The bound ball ids the dialog names (§3.6: "what it releases, by id").
    pub fn ball_ids(&self) -> Vec<String> {
        self.claims.iter().map(|c| c.id.clone()).collect()
    }

    /// The gate (§3.6): refused while any of the workspace's agents probes
    /// Live/InFlight — or is unobservable (§10). An `rm` under a flock-holding
    /// driver is a race with a running process.
    pub fn refused(&self) -> bool {
        !self.live.is_empty()
    }

    /// Armed iff the gate is clear **and** the operator typed the workspace's own
    /// name (§3.6). What counts as "typed the name" is
    /// [`names::normalize`](crate::names::normalize) — the same reading that
    /// validates a name at creation (§3.1), so a name the operator could raise is
    /// exactly a name they can retype to take down: surrounding whitespace
    /// forgiven, nothing else.
    pub fn armed(&self, typed: &str) -> bool {
        !self.refused() && crate::names::normalize(typed) == self.name
    }
}

/// **The same object off answers alone** (REMOTE §9.7, bl-b4b5) — what the §3.6
/// dialog derives at a seat that holds no world: the workspace's own name, the
/// landed `Query::Conversations` forest, and the landed `Query::WorkspaceBalls`
/// listing.
///
/// It is [`confirmation`] with its two inputs read from the other end, and it
/// answers the *same type*: one `armed`, one `refused`, one `ball_ids`,
/// whichever side asks. The engine's own re-derivation at fire is unmoved and
/// stays authoritative (§9.8, bl-1747) — this is the painted affordance, which
/// may land an ask period late and may never be the thing that decides.
pub fn confirmation_of_rows(
    name: &str,
    rows: &[crate::nav::convs::ConvRow],
    balls: &[crate::nav::BoundBall],
) -> Confirmation {
    confirmation(
        name,
        &crate::nav::convs::liveness_of_rows(rows),
        balls
            .iter()
            .filter(|b| b.state == crate::projects::join::JoinState::Bound)
            .map(|b| Claim {
                project: b.project.clone(),
                id: b.id.clone(),
            })
            .collect(),
    )
}

/// Derive the §3.6 confirmation for a workspace from its conversations
/// ([`crate::nav::convs::liveness`]) and the claims the §3.5 join binds to it.
pub fn confirmation(name: &str, convs: &[Conversation], claims: Vec<Claim>) -> Confirmation {
    Confirmation {
        name: name.to_owned(),
        conversations: convs.iter().map(|c| c.name.clone()).collect(),
        live: convs
            .iter()
            .filter(|c| c.live)
            .map(|c| c.name.clone())
            .collect(),
        claims,
    }
}

/// One step of the unmaking plan (§3.6), in the §8.1 planner idiom: pure data,
/// order load-bearing, each step individually convergent on re-run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Step {
    /// `bl unclaim <id> --as <name>` in the project — one per live bound ball,
    /// each a logged short-piped verb (§8.2). Released, not stranded: leaving N
    /// claims on a dead name is just N manual releases later.
    Release {
        project: PathBuf,
        id: String,
        name: String,
    },
    /// Prune the workspace's `ui.json` keys — `seen[ws]`, its `pinned` entry and
    /// its `collapsed` override (§4.1, §3.6 step 2), keyed by the workspace path.
    Prune { key: String },
    /// Remove the workspace directory **and its wall** — the sphere's own
    /// brazen config, sign-ins and model cache (§16.2 as amended) — logged as
    /// the one non-spawn step `["yog-step","delete-workspace"]` (§4.2's
    /// sentinel convention). One step, because §3.6's "everything inside the
    /// wall" is now literally that: leaving the wall behind would strand a
    /// dead sphere's credentials for the next workspace to reuse the name.
    Remove { workspace: PathBuf, wall: PathBuf },
}

/// The §3.6 plan for a confirmed unmaking: every release, then the prune, then the
/// removal. The gate is **not** re-checked here — [`plan`] is pure, and the one
/// place a delete is armed is [`Confirmation::armed`] at the entry point.
/// `workspace` is the path the caller already resolved the gesture's address
/// to, and `projects` the §5.1 #1 naming set each claim's project resolves
/// against — the same enumeration the name was minted over, so a claim always
/// resolves and one that somehow does not releases nothing rather than running
/// `bl` in a directory nobody named.
pub fn plan(
    confirm: &Confirmation,
    workspace: &Path,
    world_root: &Path,
    projects: &[PathBuf],
) -> Vec<Step> {
    let mut steps: Vec<Step> = confirm
        .claims
        .iter()
        .filter_map(|c| {
            Some(Step::Release {
                project: crate::naming::resolve(projects, &c.project).ok()?,
                id: c.id.clone(),
                name: confirm.name.clone(),
            })
        })
        .collect();
    steps.push(Step::Prune {
        key: crate::nav::ws_key(workspace),
    });
    steps.push(Step::Remove {
        workspace: workspace.to_path_buf(),
        wall: crate::world::wall::root_under(world_root, &confirm.name),
    });
    steps
}

#[cfg(test)]
mod tests;
