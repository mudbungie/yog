//! The ball rung's `bl claim` and the worktree cross-check that follows it
//! (DESIGN §3.3, §8.1, §5.1 #5).
//!
//! Split off [`super`] at §12's pre-split band on the seam the flow itself
//! draws: every other executor there is finished when the verb exits zero, and
//! this one is the single step whose *answer* — the path bl printed — still has
//! to be judged. A mismatch is a workspace-convention
//! [`Drift`](super::StartError::Drift), logged as a `["yog-step","cross-check"]`
//! row (Z5) before it returns, so no error class is invisible to §7.3.

use super::{AS, StartError, verb_ok};
use crate::actions::verbs::{self, log_step_failure};
use crate::binding::work_worktree_path;
use crate::cli_outbound::Cli;
use crate::opslog::Origin;
use std::path::{Path, PathBuf};

const CLAIM: &str = "claim";
/// The `["yog-step",<name>]` step name for this module's non-spawn abort (§4.2).
const DRIFT: &str = "cross-check";

/// The worktree a claim resolved to and which formula variant matched (§3.3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaimResolved {
    pub worktree: PathBuf,
    pub suffixed: bool,
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
    // The judgement is pure and its **row is this caller's** (bl-e59e): the
    // drift line is attributed to the seat that asked for the start, and only a
    // caller holding the spawn handle knows who that was. Separating the two is
    // this file's own seam said one level down — what is judged here, and what
    // that judgement costs the trail there.
    cross_check_claim(&out.stdout, balls_state_root, project, id, name).inspect_err(|err| {
        let _ = log_step_failure(
            state_root,
            ts,
            project,
            DRIFT,
            &err.to_string(),
            Origin::Balls,
            bl.client(),
        );
    })
}

/// Cross-check `bl claim`'s stdout against the bl-delivery worktree formula
/// (§3.3, §5.1 #5): the canonical `<id>` leaf or the `<id>-<claimant>` variant
/// matches; anything else is a workspace-convention
/// [`Drift`](StartError::Drift).
///
/// **Pure since bl-e59e** — the `["yog-step","cross-check"]` row (Z5) is
/// [`execute_claim`]'s, which is the only caller that knows which seat asked.
/// No error class is invisible to §7.3 by that move: the row is still written
/// on every path that reaches this judgement through the flow.
pub fn cross_check_claim(
    stdout: &str,
    balls_state_root: &Path,
    project: &Path,
    id: &str,
    name: &str,
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
    Err(err)
}
