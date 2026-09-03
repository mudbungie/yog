//! The composite "start a conversation" verb (DESIGN §3.4, §8.1, §15 M6 Z3):
//! **a pure planner + a step executor**, the one flow that turns Enter-in-a-box
//! into a running litany loop.
//!
//! §3.4's two orthogonal axes, one composer. *Where* a prompt goes — the target
//! `workspace` path: the focused one, or, in a world with zero workspaces,
//! `<names-root>/home` (§3.1's default name). **The where axis is a path, and
//! only a path** (bl-d942): workspace names are the operator's now, chosen before
//! the flow starts, so there is no name to resolve here and no minted/existing
//! branch to take — bootstrap is the empty case of the general path, an absent
//! directory [`Step::EnsureWorkspace`] founds. *What* it carries — the [`Payload`]
//! rung: **bare** (an empty composer), **path** (a work directory), or **ball**
//! (a picked/created ball).
//!
//! [`plan`] is the pure planner: given the [`StartInputs`] it returns the ordered
//! [`Step`] sequence — the amended
//! §8.1 order **seed → `litany new` → `bl` mutations → prompt**, so every
//! substrate step precedes every `bl` mutation and a failed substrate can never
//! mint an orphaned claim. Every step is idempotent-or-convergent, so re-running
//! `plan` after a partial failure yields the shorter remainder: a bound ball
//! drops [`Step::Claim`], an existing workspace's [`Step::EnsureWorkspace`] is a
//! no-op skip. A **new** ball defers its id to a single [`Step::Create`]; the
//! executor re-plans the freshly-minted (Ready, unclaimed) ball — the
//! new→existing transition *is* the convergence, not a special case.
//!
//! The request the planner reads and the step sequence it returns — [`Payload`]
//! and its [`BallSpec`], [`StartInputs`], [`Step`] — are [`model`]: the inert
//! shapes, split from the planner at §12's pre-split band, so what a start *is*
//! and how it is *derived* are two files.
//!
//! The effectful half — the piped `bl`/`litany` executors and the claim
//! cross-check — lives in [`exec`]; the detached `litany prompt` and the
//! conversation mint it fires with in [`prompt`]; the goal composition in
//! [`goal`]. The **goal reaches the model unmutated**
//! (§3.3, bl-6920): [`prompt::execute_prompt`] mints afresh and
//! passes the name via `--name` as it fires — litany states the stored fact in
//! its assembled context; nothing is prepended to the payload. The name is the
//! *conversation's* (bl-df65); the workspace's rides `YOG_NAME` and never the
//! goal text. **The pre-submit name prediction and the composer view-model are
//! the seat's** (bl-7cc8): `Prepared` carries no predicted name, so `/prepare`
//! answers nothing a seat could preview, and a derivation with no carrier is
//! not this crate's.

use crate::projects::join::JoinState;

mod ensure;
mod exec;
mod goal;
mod identity;
pub mod instructions;
mod lineage;
mod model;
mod prompt;
mod run;
#[cfg(test)]
mod tests;

pub use ensure::execute_ensure_workspace;
pub use exec::{
    ClaimResolved, DETACHED_EXIT, Deps, Prepared, StartError, cross_check_claim, execute_claim,
    execute_create, on_mint,
};
pub use goal::parse_ball_stamp;
pub use identity::{parse_identity_stamp, strip_identity_stamp};
pub use model::{BallSpec, Payload, StartInputs, Step};
pub use prompt::{Fire, execute_prompt};
pub use run::{prepare, resolve_worktree};

/// The pure planner (§8.1): the amended-order step sequence to reach a running
/// loop. Substrate first (seed, `litany new`), then the ball rung's `bl`
/// mutations (create for a new ball — the id defers the rest to a re-plan; else
/// claim when unclaimed), then the deferred prompt. Re-run after any step and it
/// converges to the shorter remainder.
pub fn plan(inputs: &StartInputs) -> Vec<Step> {
    let workspace = inputs.workspace.clone();
    let mut steps = vec![
        Step::EnsureSeeded,
        Step::EnsureWorkspace {
            workspace: workspace.clone(),
        },
    ];
    match &inputs.payload {
        Payload::Ball {
            project,
            ball: BallSpec::New { title, body },
        } => {
            // The id is unknown until `bl create` mints it: emit create alone and
            // re-plan the minted ball (the new→existing convergence, §8.1).
            let _ = project;
            steps.push(Step::Create {
                project: inputs.repo.clone().unwrap_or_default(),
                title: title.clone(),
                body: body.clone(),
            });
            return steps;
        }
        Payload::Ball {
            project,
            ball: BallSpec::Existing { id, join, .. },
        } if claim_needed(*join) => {
            let _ = project;
            steps.push(Step::Claim {
                project: inputs.repo.clone().unwrap_or_default(),
                id: id.clone(),
                name: crate::naming::leaf(&inputs.workspace),
            });
        }
        _ => {}
    }
    // The planner is pure, so the Prompt step previews the composer with the
    // ball's *canonical* worktree formula (§3.3); the executor re-composes with
    // the claim's cross-checked worktree (canonical or `<id>-<claimant>`) once it
    // has run [`Step::Claim`] — the executor's return is authoritative.
    let worktree = goal::canonical_worktree(inputs);
    let prepared = goal::compose_prepared(inputs, worktree.as_deref());
    steps.push(Step::Prompt {
        workspace: prepared.workspace,
        binding: prepared.binding,
        goal: prepared.goal,
    });
    steps
}

/// Whether the start flow must claim (§3.5, §8.1): a ready, unclaimed ball. A
/// ball already bound to its workspace ([`JoinState::Bound`]) drops the claim —
/// resume, not a second mint; re-claiming would trip bl's benign double-claim.
pub(crate) fn claim_needed(join: JoinState) -> bool {
    matches!(join, JoinState::ReadyStartable)
}

/// Whether the roster offers a ▶ Start affordance for this join state (§3.5,
/// §11): a ready ball. A [`JoinState::Bound`] ball already has a running
/// workspace (re-prompt is the composer's job, §3.4).
pub fn is_start_eligible(state: JoinState) -> bool {
    matches!(state, JoinState::ReadyStartable)
}

/// Whether the roster offers a ▶ Continue (resume) affordance for this join state
/// (§8.1 resume, addendum): a [`JoinState::Bound`] ball. [`plan`] is total over
/// every join state — it will happily plan for a claimed-elsewhere ball — so this
/// predicate is the **only** guard against resuming a ball this yog does not own;
/// it must stay covered, never a shell-glue check. Routes through the same planner
/// (prompt-only, since [`claim_needed`] is false for a bound ball).
pub fn is_resume_eligible(state: JoinState) -> bool {
    matches!(state, JoinState::Bound)
}
