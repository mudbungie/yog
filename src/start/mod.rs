//! The composite "start a conversation" verb (DESIGN §3.4, §8.1, §15 M6 Z3):
//! **a pure planner + a step executor**, the one flow that turns Enter-in-a-box
//! into a running lernie loop.
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
//! §8.1 order **seed → `lernie new` → `bl` mutations → prompt**, so every
//! substrate step precedes every `bl` mutation and a failed substrate can never
//! mint an orphaned claim. Every step is idempotent-or-convergent, so re-running
//! `plan` after a partial failure yields the shorter remainder: a bound ball
//! drops [`Step::Claim`], an existing workspace's [`Step::EnsureWorkspace`] is a
//! no-op skip. A **new** ball defers its id to a single [`Step::Create`]; the
//! executor re-plans the freshly-minted (Ready, unclaimed) ball — the
//! new→existing transition *is* the convergence, not a special case.
//!
//! The effectful half — the piped `bl`/`lernie` executors and the claim
//! cross-check — lives in [`exec`]; the detached `lernie prompt` and the
//! conversation mint it fires with in [`prompt`]; the goal composition and the
//! pre-mint preview in [`goal`]. The **goal reaches the model unmutated**
//! (§3.3, bl-6920): [`goal::preview`] renders the greyed
//! name prediction pre-submit, and [`prompt::execute_prompt`] mints afresh and
//! passes the name via `--name` as it fires — lernie states the stored fact in
//! its assembled context; nothing is prepended to the payload. The name is the
//! *conversation's* (bl-df65); the workspace's rides `YOG_NAME` and never the
//! goal text.

use crate::projects::join::JoinState;
use std::path::PathBuf;

mod ensure;
mod exec;
mod goal;
mod identity;
mod prompt;
mod run;
#[cfg(test)]
mod tests;

pub use ensure::execute_ensure_workspace;
pub use exec::{
    ClaimResolved, DETACHED_EXIT, Deps, Prepared, StartError, cross_check_claim, execute_claim,
    execute_create, on_mint,
};
pub(crate) use goal::leaf_name;
pub use goal::{Composer, parse_ball_stamp, preview};
pub use identity::{identity_preview, parse_identity_stamp, strip_identity_stamp};
pub use prompt::execute_prompt;
pub use run::{prepare, resolve_worktree};

/// The ball a start targets (§3.4 ball rung): an **existing** ball (id + join
/// state known, from the roster) or a **new** ball whose id `bl create` mints.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BallSpec {
    Existing {
        id: String,
        title: String,
        body: String,
        join: JoinState,
    },
    New {
        title: String,
        body: String,
    },
}

/// The **what** axis (§3.4): the payload rung, each the one below plus inputs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Payload {
    /// bare — the empty composer; driver cwd `~`.
    Bare,
    /// path — a work directory; target preamble; driver cwd the directory.
    Path { dir: PathBuf },
    /// ball — a ball in `project`; `bl claim`/`create`; driver cwd the worktree.
    Ball { project: PathBuf, ball: BallSpec },
}

impl Payload {
    /// The §7.3 banner surface every ops row this rung writes is attributed to
    /// (bl-48f8) — **the rung is the origin**, and it is the one thing the argv
    /// cannot say: a ball-rung start and a bare-rung one write byte-identical
    /// `lernie new` / `lernie prompt` / `["yog-step","mkdir"]` lines, so a
    /// derivation at read time would hand the balls fold and the composer the
    /// same row and be wrong for one of them every time.
    ///
    /// A ball rung was offered on the roster's balls section — the ▶ Start /
    /// ▶ Continue / Create-&-Start rows — so that is where its whole flow
    /// banners, substrate steps included (§11, bl-6ad8: "banners where the start
    /// was offered … the surface the ▶ Start row itself is on"). The bare and
    /// path rungs are the composer's own Enter, the empty world's bootstrap box
    /// being the same box before a workspace exists.
    pub fn origin(&self) -> crate::opslog::Origin {
        match self {
            Self::Ball { .. } => crate::opslog::Origin::Balls,
            Self::Bare | Self::Path { .. } => crate::opslog::Origin::Conversation,
        }
    }
}

/// The start request the shell hands [`plan`] / [`prepare`] / [`preview`]: the
/// two §3.4 axes plus the roots the worktree and seed paths derive from, and the
/// conversation mint's occupied set. `home` is the bare rung's driver cwd (`~`,
/// resolved from the env at the shell boundary).
///
/// **One input type, because there is nothing left to resolve** (bl-d942). It
/// carried a `Target` and the workspace mint's occupied-set claimants, and
/// [`prepare`] resolved those into a second, name-bearing struct; with workspace
/// names chosen by the operator (§3.1) the target *is* the workspace path and its
/// name *is* the leaf — a computed fact, so it is a query ([`goal::leaf_name`])
/// and not a field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StartInputs {
    /// The target workspace's absolute path (§3.4): the focused workspace (named
    /// **or foreign**), a resume ball's claimant workspace, or `<names-root>/
    /// <name>` for a raise — the operator's typed name, or `home` at bootstrap
    /// (§3.1). Its leaf is the `--as`/`YOG_NAME` stamp. Carrying the path, not a
    /// names-root-relative leaf, is what lets a foreign focus (which lives outside
    /// yog's flat names root) resolve to the right `lernie prompt <ws>`. An absent
    /// directory is founded by [`Step::EnsureWorkspace`] — that, and nothing else,
    /// is what "raising a workspace" is.
    pub workspace: PathBuf,
    pub payload: Payload,
    pub home: PathBuf,
    pub yog_data_root: PathBuf,
    pub balls_state_root: PathBuf,
    /// The occupied set for the **conversation** mint (§3.3): the stamped names of
    /// the target workspace's live roots, read back from the goals the §11
    /// conversation list already parses. Per-workspace and nothing wider —
    /// workspaces are isolation walls, so two spheres never need distinct names.
    /// Empty for a workspace that does not exist yet: the general path with no
    /// inputs, not a bootstrap case.
    pub conversation_names: Vec<String>,
}

/// One step of the start flow (§8.1). The sequence is a projection; [`prepare`]
/// runs the mutating steps in order and **defers** [`Prompt`](Step::Prompt) to
/// the composer (fired later, edited, by [`execute_prompt`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Step {
    /// **The world is founded** — always planned; both halves converge rather
    /// than branch. The executor seeds `<world>/tools/bl`, the agent-tool shim
    /// (§16.7 W9, rewritten only on drift), then runs `LERNIE_HOME=… lernie
    /// prime`, skipping a seeded home (§16.6 W3, the general path with the seed
    /// present). A marker step: the executor derives the world layout from
    /// `yog_data_root` (the single source), so the step carries no path of its own.
    EnsureSeeded,
    /// `mkdir -p` + `lernie new <workspace>` — always planned; the executor skips
    /// an existing dir (§8.1 convergence; bootstrap is this with an absent dir).
    EnsureWorkspace { workspace: PathBuf },
    /// `bl create <title> [--body B]` — the ball New rung; mints the id, after
    /// which the plan re-derives as an existing ball (§8.1).
    Create {
        project: PathBuf,
        title: String,
        body: String,
    },
    /// `bl claim <id> --as <name>` — the ball rung, unclaimed only; stamped with
    /// the target workspace name (§3.2). Dropped for a bound ball (resume).
    Claim {
        project: PathBuf,
        id: String,
        name: String,
    },
    /// `lernie prompt <workspace> <goal>` fired detached, `YOG_NAME=<name>` (the
    /// **workspace** name, §3.2), cwd per the §3.4 rung. `goal` is the editable
    /// payload prefill; the conversation's identity line is minted and stamped at
    /// fire, never carried here (§3.3).
    Prompt {
        name: String,
        workspace: PathBuf,
        /// The §3.3 typed work target the fire will pass as lernie's `--cwd`
        /// (bl-6654) — the plan's preview of it, off the ball's *canonical*
        /// worktree formula; the executor re-derives it from the claim.
        binding: Option<PathBuf>,
        goal: String,
    },
}

/// The pure planner (§8.1): the amended-order step sequence to reach a running
/// loop. Substrate first (seed, `lernie new`), then the ball rung's `bl`
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
            steps.push(Step::Create {
                project: project.clone(),
                title: title.clone(),
                body: body.clone(),
            });
            return steps;
        }
        Payload::Ball {
            project,
            ball: BallSpec::Existing { id, join, .. },
        } if claim_needed(*join) => {
            steps.push(Step::Claim {
                project: project.clone(),
                id: id.clone(),
                name: goal::leaf_name(&inputs.workspace),
            });
        }
        _ => {}
    }
    // The planner is pure, so the Prompt step previews the composer with the
    // ball's *canonical* worktree formula (§3.3); the executor re-composes with
    // the claim's cross-checked worktree (canonical or `<id>-<claimant>`) once it
    // has run [`Step::Claim`] — the executor's return is authoritative.
    let worktree = goal::canonical_worktree(&inputs.payload, &inputs.balls_state_root);
    let prepared = goal::compose_prepared(inputs, worktree.as_deref());
    steps.push(Step::Prompt {
        name: prepared.name,
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
