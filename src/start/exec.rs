//! The effectful half of the start flow (DESIGN §3.3, §8.1, §15 M6 Z3): the
//! piped `bl`/`lernie` executors, the claim worktree
//! cross-check, and the shapes the [`prepare`](super::prepare) orchestrator
//! threads through them. The detached fire — and the §3.3 conversation mint it
//! stamps with — is [`prompt`](super::prompt); everything here is piped, gated
//! and re-runnable.
//!
//! Every non-spawn abort leaves a `["yog-step",<name>]` ops row before it returns
//! (§4.2, Z5's [`log_step_failure`]): the conversation mint's pool exhaustion
//! ([`on_mint`], applied at the fire), the workspace `mkdir`
//! ([`execute_ensure_workspace`]),
//! and the claim cross-check [`Drift`](StartError::Drift) — so no error class is
//! invisible to the §7.3 failed-action surface (the eprintln purge left none
//! behind).

use crate::actions::verbs::{self, Outcome, log_step_failure};
use crate::binding::work_worktree_path;
use crate::cli_outbound::Cli;
use crate::opslog::Origin;
use crate::world::seed;
use lernie::mint::MintError;
use std::io;
use std::path::{Path, PathBuf};

const CREATE: &str = "create";
const CLAIM: &str = "claim";
const AS: &str = "--as";
const BODY: &str = "--body";
pub(super) const NEW: &str = "new";
/// The workspace marker (§3.1); a workspace is a dir directly holding it.
pub(super) const REPO_MARK: &str = "repo.git";
/// The `["yog-step",<name>]` step names for the non-spawn aborts (§4.2).
const MINT: &str = "mint";
pub(super) const MKDIR: &str = "mkdir";
const DRIFT: &str = "cross-check";
/// The §8.6 authoring of the capability control onto `config/default`.
pub(super) const CONTROL: &str = "control";

pub use crate::opslog::DETACHED_EXIT;

/// The injected binaries + the ops-log target (§14). Owned — [`prepare`] borrows
/// each field as it threads them into the per-step executors.
pub struct Deps {
    pub bl: Cli,
    pub lernie: Cli,
    pub state_root: PathBuf,
    /// yog's own binary — the `$EDITOR` shim the §9.3 lineage write re-enters,
    /// which is how §8.6 authors the capability control onto `config/default`
    /// without yog ever writing inside a workspace itself.
    pub yog_binary: PathBuf,
}

/// The worktree a claim resolved to and which formula variant matched (§3.3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaimResolved {
    pub worktree: PathBuf,
    pub suffixed: bool,
}

/// The composer's fire-time parameters (§8.1): the resolved workspace `name` (for
/// `YOG_NAME` alone — it never enters the goal text, §3.3), the `workspace` it
/// was prepared in, the per-rung binding, and the editable `goal` prefill (the
/// conversation's identity line is minted and stamped at fire, not carried here).
///
/// **A prepare reply is also the next gesture** (§8.1), so it obeys the wire's
/// own rule (REMOTE §8, bl-f5f6): `workspace` is the **name**, re-resolved when
/// the deferred [`Prompt`](crate::boundary::Action::Prompt) lands. `binding` is
/// the one path left, and it is not an identity — it is lernie's `--cwd`, a
/// filesystem fact the engine minted and the engine consumes, carried back
/// verbatim by a seat that never reads it.
///
/// It carried a separate `name` — the §3.2 `--as`/`YOG_NAME` stamp — until
/// bl-f5f6. §3.1 makes a workspace's name its directory leaf and §3.2 makes
/// that same leaf the claim identity, so once `workspace` became the name the
/// two fields were one string twice, and the second went.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Prepared {
    /// The workspace's name (§3.1: its leaf) — the boundary address **and**
    /// the §3.2 `--as`/`YOG_NAME` stamp, which are one fact.
    pub workspace: String,
    /// The §3.3 **typed work target** (bl-6654): the directory the fire passes
    /// as lernie's `--cwd`, seeding the agent's working-directory mark at
    /// creation so *every* tool step runs there — not just the first process.
    /// `None` binds nothing and lets lernie's own default (the agent worktree)
    /// stand: the bare rung, and a ball not yet created.
    pub binding: Option<PathBuf>,
    pub goal: String,
    /// The §7.3 banner surface this start's ops rows carry (bl-48f8) — the
    /// rung's own ([`Payload::origin`](super::Payload::origin)), carried here so
    /// the deferred fire tags the same surface the prepare steps did. A start
    /// that failed at `bl claim` and one that failed at the detached prompt are
    /// the same gesture, and must not banner in two different places.
    pub origin: Origin,
}

/// A start-flow failure. Every variant is already a durable ops row before it
/// rides back (a ran-non-zero verb's [`Outcome`], a synthetic step-failure line
/// for the mint / mkdir / [`Drift`](StartError::Drift)).
#[derive(Debug, thiserror::Error)]
pub enum StartError {
    #[error("`{verb}` failed (exit {}): {}", .outcome.exit, .outcome.stderr)]
    VerbFailed {
        verb: &'static str,
        outcome: Outcome,
    },
    #[error("claim worktree drift: bl printed {stdout:?}, expected {canonical} or {suffixed}")]
    Drift {
        stdout: String,
        canonical: String,
        suffixed: String,
    },
    #[error(transparent)]
    Mint(#[from] MintError),
    #[error(transparent)]
    Seed(#[from] seed::SeedError),
    /// The `lernie config` drive that authors the capability control onto
    /// `config/default` ran non-zero (§8.6). A workspace whose policy could not
    /// be written would birth drones nothing adjudicates, so the start stops
    /// here rather than proceeding uncontrolled.
    #[error("the capability control could not be authored: {0}")]
    Control(String),
    #[error(transparent)]
    Io(#[from] io::Error),
}

/// Map a mint result to a name or a **logged abort** (§8.1): an exhausted pool
/// leaves a `["yog-step","mint"]` row (Z5) before it returns, so neither mint's
/// one non-spawn failure — the §3.1 workspace name here, the §3.3 conversation
/// name at fire — is ever a dropped error.
pub fn on_mint(
    result: Result<String, MintError>,
    state_root: &Path,
    ts: &str,
    cwd: &Path,
    origin: Origin,
) -> Result<String, StartError> {
    match result {
        Ok(name) => Ok(name),
        Err(e) => {
            log_step_failure(state_root, ts, cwd, MINT, &e.to_string(), origin)?;
            Err(StartError::Mint(e))
        }
    }
}

/// `bl create <title> --as <name> [--body B]` in the project (§8.2). Returns the
/// minted id (bl prints it on stdout). An empty body is elided.
pub fn execute_create(
    bl: &Cli,
    state_root: &Path,
    ts: &str,
    project: &Path,
    title: &str,
    body: &str,
    name: &str,
) -> Result<String, StartError> {
    // The ungated `run_logged` core (not `verbs::create`) — `prepare` already gated.
    let mut args = vec![CREATE, title, AS, name];
    if !body.is_empty() {
        args.extend([BODY, body]);
    }
    // A `bl create` is only ever the ball rung's (§8.1), so its origin is the
    // roster's balls section outright, not a threaded parameter.
    let out = verbs::run_logged(bl, state_root, ts, project, &args, Origin::Balls)?;
    verb_ok(out, CREATE).map(|o| o.stdout.trim().to_owned())
}

/// `bl claim <id> --as <name>` in the project (§8.1), piped + opslog'd, then the
/// stdout worktree path cross-checked against the bl-delivery formula.
pub fn execute_claim(
    bl: &Cli,
    state_root: &Path,
    ts: &str,
    project: &Path,
    id: &str,
    name: &str,
    balls_state_root: &Path,
) -> Result<ClaimResolved, StartError> {
    let out = verb_ok(
        verbs::run_logged(
            bl,
            state_root,
            ts,
            project,
            &[CLAIM, id, AS, name],
            Origin::Balls,
        )?,
        CLAIM,
    )?;
    cross_check_claim(
        &out.stdout,
        balls_state_root,
        project,
        id,
        name,
        state_root,
        ts,
    )
}

/// Cross-check `bl claim`'s stdout against the bl-delivery worktree formula
/// (§3.3, §5.1 #5): the canonical `<id>` leaf or the `<id>-<claimant>` variant
/// matches; anything else is a workspace-convention [`Drift`](StartError::Drift),
/// logged as a `["yog-step","cross-check"]` row (Z5) before it returns.
pub fn cross_check_claim(
    stdout: &str,
    balls_state_root: &Path,
    project: &Path,
    id: &str,
    name: &str,
    state_root: &Path,
    ts: &str,
) -> Result<ClaimResolved, StartError> {
    let got = PathBuf::from(stdout.trim());
    let canonical = work_worktree_path(balls_state_root, project, id, None);
    let suffixed = work_worktree_path(balls_state_root, project, id, Some(name));
    if got == canonical {
        return Ok(ClaimResolved {
            worktree: canonical,
            suffixed: false,
        });
    }
    if got == suffixed {
        return Ok(ClaimResolved {
            worktree: suffixed,
            suffixed: true,
        });
    }
    let err = StartError::Drift {
        stdout: got.display().to_string(),
        canonical: canonical.display().to_string(),
        suffixed: suffixed.display().to_string(),
    };
    log_step_failure(
        state_root,
        ts,
        project,
        DRIFT,
        &err.to_string(),
        Origin::Balls,
    )?;
    Err(err)
}

/// Return `out` iff the verb exited 0, else a [`StartError::VerbFailed`] carrying
/// its already-logged [`Outcome`] (§8.2).
pub(super) fn verb_ok(out: Outcome, verb: &'static str) -> Result<Outcome, StartError> {
    if out.ok() {
        Ok(out)
    } else {
        Err(StartError::VerbFailed { verb, outcome: out })
    }
}
