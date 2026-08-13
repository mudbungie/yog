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

/// One live bound ball the unmaking releases (§3.6 step 1) — the ball's id and the
/// project whose store holds it (the `bl` cwd, §8.2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Claim {
    pub project: PathBuf,
    pub id: String,
}

/// What the §3.6 dialog states and the plan runs — derived, never stored: the
/// workspace's name and path, every conversation that dies (by display name),
/// the live ones (the gate), and the claims the unmaking releases.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Confirmation {
    pub name: String,
    pub workspace: PathBuf,
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

/// Derive the §3.6 confirmation for a workspace from its conversations
/// ([`crate::nav::convs::liveness`]) and the claims the §3.5 join binds to it.
pub fn confirmation(
    name: &str,
    workspace: &Path,
    convs: &[Conversation],
    claims: Vec<Claim>,
) -> Confirmation {
    Confirmation {
        name: name.to_owned(),
        workspace: workspace.to_path_buf(),
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
pub fn plan(confirm: &Confirmation, world_root: &Path) -> Vec<Step> {
    let mut steps: Vec<Step> = confirm
        .claims
        .iter()
        .map(|c| Step::Release {
            project: c.project.clone(),
            id: c.id.clone(),
            name: confirm.name.clone(),
        })
        .collect();
    steps.push(Step::Prune {
        key: crate::nav::ws_key(&confirm.workspace),
    });
    steps.push(Step::Remove {
        workspace: confirm.workspace.clone(),
        wall: crate::world::wall::root_under(world_root, &confirm.name),
    });
    steps
}

#[cfg(test)]
mod tests;
