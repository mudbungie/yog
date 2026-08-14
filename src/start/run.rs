//! The [`prepare`] orchestrator (DESIGN §8.1, §15 M6 Z3): it **runs
//! [`plan`](super::plan)'s output step-by-step** — the order lives only in the
//! planner, never duplicated here. There is nothing to resolve first (bl-d942:
//! the target is a path and the name is its leaf), so [`prepare`] *is* the
//! iteration: the substrate + `bl` mutations run in place, a new ball's
//! [`Create`](super::Step::Create) re-plans the minted ball (the new→existing
//! convergence), and the detached [`Prompt`](super::Step::Prompt) is **deferred**
//! to the composer — its fire-time params are composed purely from the same
//! source ([`goal::compose_prepared`](super::goal)).

use super::ensure::execute_ensure_workspace;
use super::exec::{execute_claim, execute_create};
use super::goal::compose_prepared;
use super::{BallSpec, Deps, Payload, Prepared, StartError, StartInputs, Step};
use crate::binding::work_worktree_path;
use crate::projects::join::JoinState;
use crate::world::{layout_under, seed, tools};
use std::path::{Path, PathBuf};

/// The resolved ball worktree for the composer (§3.3, addendum): the claim's
/// cross-checked worktree when a claim ran (canonical or `<id>-<claimant>`, from
/// [`ClaimResolved`](super::ClaimResolved)), else — the resume path, no claim —
/// the variant that exists on disk (`<id>-<claimant>` only when it alone is
/// present), else the canonical formula. `None` for non-existing-ball rungs
/// (bare/path/new name no worktree), so the composer's driver cwd falls back to
/// `~` (§3.4). `pub`: the story fixtures assert the resolution directly.
pub fn resolve_worktree(
    payload: &Payload,
    repo: Option<&Path>,
    balls_state_root: &Path,
    name: &str,
    claimed: Option<PathBuf>,
) -> Option<PathBuf> {
    let (
        Payload::Ball {
            ball: BallSpec::Existing { id, .. },
            ..
        },
        Some(repo),
    ) = (payload, repo)
    else {
        return None;
    };
    Some(claimed.unwrap_or_else(|| existing_worktree(balls_state_root, repo, id, name)))
}

/// The on-disk worktree of an already-claimed ball for the resume path (§8.1): the
/// `<id>-<claimant>` variant when only it exists, else the canonical `<id>` — a
/// pure disk read (`Path::exists`), never a mutation (I7). The resume path has no
/// `bl claim` stdout to cross-check, so disk is the ground truth here.
fn existing_worktree(balls_state_root: &Path, project: &Path, id: &str, name: &str) -> PathBuf {
    let canonical = work_worktree_path(balls_state_root, project, id, None);
    let suffixed = work_worktree_path(balls_state_root, project, id, Some(name));
    if !canonical.exists() && suffixed.exists() {
        suffixed
    } else {
        canonical
    }
}

/// Run every mutating step [`plan`](super::plan) emits (§8.1: "the executor runs
/// it step-by-step"), returning the composer's [`Prepared`]. The planned `Prompt`
/// is deferred (a no-op) — it fires later, on confirm
/// ([`super::execute_prompt`]) — and is composed after the loop with the
/// **claim's cross-checked worktree** (addendum: never the canonical guess), so
/// the preamble + driver cwd name the path bl actually minted. A new ball's plan
/// ends at `Create`, which re-enters here; every other plan reaches the
/// after-loop return.
pub fn prepare(deps: &Deps, inputs: &StartInputs, ts: &str) -> Result<Prepared, StartError> {
    let name = crate::naming::leaf(&inputs.workspace);
    // The §7.3 attribution for every row this flow writes (bl-48f8): the rung's
    // own, read once from the payload. The substrate steps below name no ball
    // and no conversation, so nothing downstream could recover it from the argv.
    let origin = inputs.payload.origin();
    let mut claimed: Option<PathBuf> = None;
    for step in super::plan(inputs) {
        match step {
            Step::EnsureSeeded => {
                let layout = layout_under(&inputs.yog_data_root);
                // The world's agent tools first (§16.7 W9): a pure-disk converge
                // of `<world>/tools/bl`, so the shim an agent's bash finds is
                // present before any driver exists to run one. Idempotent, no
                // spawn — it precedes the lernie seed for cost, not order.
                tools::ensure_shim(&layout.tools, tools::BL, &deps.bl)?;
                seed::ensure_seeded(&deps.lernie, &deps.state_root, ts, &layout, origin)?;
                // The capability control's own shim (§8.6): the executable
                // lernie's tool-control seam consults, converged here beside
                // the agent tools because the next step authors its absolute
                // path into the workspace's policy.
                tools::ensure_control(&layout.tools)?;
            }
            Step::EnsureWorkspace { workspace } => {
                // The birth template is judged inside the ensure, on the fresh
                // branch only (bl-c3a9) — the layout is the same pure fold the
                // seed step made, so the file gated and the file `lernie new`
                // commits are one path. The pinned template already grants the
                // worker role the whole tool pool (§8.1, bl-7fc8), so nothing
                // runs after: the first `config/default` is lernie's/operator's
                // one home, read and edited through §9.3/§9.4 like every later
                // config.
                let layout = layout_under(&inputs.yog_data_root);
                execute_ensure_workspace(deps, ts, &workspace, &layout, origin)?;
            }
            Step::Create {
                project,
                title,
                body,
            } => {
                let id = execute_create(
                    &deps.bl,
                    &deps.state_root,
                    ts,
                    &project,
                    &title,
                    &body,
                    &name,
                )?;
                return prepare(deps, &with_minted(inputs, id, title, body), ts);
            }
            Step::Claim { project, id, name } => {
                claimed = Some(
                    execute_claim(
                        &deps.bl,
                        &deps.state_root,
                        ts,
                        &project,
                        &id,
                        &name,
                        &inputs.balls_state_root,
                    )?
                    .worktree,
                );
            }
            Step::Prompt { .. } => {}
        }
    }
    let worktree = resolve_worktree(
        &inputs.payload,
        inputs.repo.as_deref(),
        &inputs.balls_state_root,
        &name,
        claimed,
    );
    Ok(compose_prepared(inputs, worktree.as_deref()))
}

/// Re-plan a new ball as its freshly-minted existing self (§8.1): the Ready,
/// unclaimed ball whose id `bl create` just returned — the convergence.
fn with_minted(inputs: &StartInputs, id: String, title: String, body: String) -> StartInputs {
    StartInputs {
        payload: Payload::Ball {
            // The payload keeps the project **name** it came in with (REMOTE
            // §8); only the ball changes. `inputs.repo` is that name already
            // located, and stays as it is.
            project: inputs.payload.project().unwrap_or_default(),
            ball: BallSpec::Existing {
                id,
                title,
                body,
                join: JoinState::ReadyStartable,
            },
        },
        ..inputs.clone()
    }
}
