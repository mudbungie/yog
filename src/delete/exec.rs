//! The effectful half of the §3.6 unmaking: run the [`plan`](super::plan)'s steps
//! in order, each leaving its durable record.
//!
//! Every step is one already-established mechanism — the logged short-piped `bl
//! unclaim` (§8.2), the write-through `ui.json` mutation (§4.1), and one
//! non-spawn `["yog-step","delete-workspace"]` ops row (§4.2). A refused unclaim
//! surfaces from its own ops row and **aborts before the removal** (§3.6): the
//! wall stays up with some claims released, which is a state the §3.5 join
//! already renders, never an error class.

use super::Step;
use crate::actions::verbs::{self, Outcome, log_step_done, log_step_failure};
use crate::cli_outbound::Cli;
use crate::opslog::Origin;
use crate::ui_state::UiState;
use std::io;
use std::path::Path;

/// The step name the removal logs under (§4.2's `["yog-step",…]` convention).
pub const DELETE_STEP: &str = "delete-workspace";

/// Why an unmaking did not happen. Every variant is a *refusal or an abort with a
/// durable record*: the two gates refuse before anything runs, and the verb/io
/// failures already left their `ops.jsonl` line before riding back.
#[derive(Debug, thiserror::Error)]
pub enum DeleteError {
    /// The §3.6 gate: the workspace has live conversations, named here so the
    /// operator can stop them first. Nothing was attempted.
    #[error("refused — live conversations: {}", .0.join(", "))]
    Live(Vec<String>),
    /// The typed name did not match the workspace's (§3.6 confirmation
    /// doctrine). Nothing was attempted.
    #[error("type the workspace's name to confirm")]
    NotArmed,
    /// The workspace is not one of yog's own named workspaces (§3.6 scope): yog
    /// may not delete what it did not place, and a replay is read-only.
    #[error("not a yog-named workspace")]
    Unnamed,
    /// A `bl unclaim` was refused (balls' own semantics, e.g. a store race) —
    /// the plan aborts before the removal, its ops row already written.
    #[error("`bl unclaim {id}` failed (exit {}): {}", .outcome.exit, .outcome.stderr)]
    ReleaseFailed { id: String, outcome: Outcome },
    #[error(transparent)]
    Io(#[from] io::Error),
}

/// Run the plan (§3.6). Releases first, removal last; the first failure aborts,
/// leaving a lawful, re-runnable intermediate state.
pub fn execute(
    steps: &[Step],
    bl: &Cli,
    ui: &mut UiState,
    state_root: &Path,
    ts: &str,
) -> Result<(), DeleteError> {
    for step in steps {
        match step {
            Step::Release { project, id, name } => {
                let outcome = verbs::unclaim(bl, state_root, ts, project, id, name)?;
                if !outcome.ok() {
                    return Err(DeleteError::ReleaseFailed {
                        id: id.clone(),
                        outcome,
                    });
                }
            }
            Step::Prune { key } => ui.prune_workspace(key),
            Step::Remove { workspace, wall } => remove(workspace, wall, state_root, ts)?,
        }
    }
    Ok(())
}

/// Remove the workspace directory whole and log the non-spawn step (§3.6 step 3,
/// §4.2). The ops row's `cwd` is the names root the write is made against — the
/// truthful directory, since the subject itself is gone by the time it lands.
fn remove(workspace: &Path, wall: &Path, state_root: &Path, ts: &str) -> Result<(), DeleteError> {
    let cwd = workspace.parent().unwrap_or(workspace);
    match remove_both(workspace, wall) {
        Ok(()) => {
            log_step_done(state_root, ts, cwd, DELETE_STEP, Origin::World)?;
            Ok(())
        }
        Err(e) => {
            log_step_failure(
                state_root,
                ts,
                cwd,
                DELETE_STEP,
                &e.to_string(),
                Origin::World,
            )?;
            Err(DeleteError::Io(e))
        }
    }
}

/// The wall, then the workspace (§3.6, §16.2 as amended). A wall that was never
/// materialized — a sphere whose providers were never configured — is **not** a
/// failure: absence is the same end state the removal is for, so `NotFound`
/// passes through and the workspace removal decides the step.
fn remove_both(workspace: &Path, wall: &Path) -> io::Result<()> {
    match std::fs::remove_dir_all(wall) {
        Err(e) if e.kind() != io::ErrorKind::NotFound => return Err(e),
        _ => {}
    }
    std::fs::remove_dir_all(workspace)
}

#[cfg(test)]
mod tests;
