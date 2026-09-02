//! The alignment monitor's two boundary executors (VISION §4.9, rung V6):
//! arming and flagging. Split out of [`dispatch`](super::dispatch) for the same
//! reason every other family is — the chokepoint is a table, and a table stops
//! being one when arms carry bodies.
//!
//! **Arming is a config write and nothing else.** It adds (or deletes) one
//! `cadence.yaml` entry and seeds the policy file the entry names. There is no
//! monitor to start: the sentry ([`crate::monitor::sentry`]) is always running
//! and finds nothing to do until an entry exists, so arming cannot half-apply
//! and disarming cannot leave a thread behind.
//!
//! **Flagging is one ops row and nothing else.** It is the signal-out verb — a
//! typed call, so an alignment responder can say "look at this" without yog
//! parsing a verdict out of its prose (bl-7aef) — and it deliberately touches
//! neither the conversation nor its driver.

use std::path::Path;

use crate::monitor::{Verb, arming, flag as flagging};
use crate::opslog::{self, OpEntry, Origin};

use super::dispatch::Deps;
use super::reply::Reply;

/// The step name the arm/disarm gesture logs its own completion under, so the
/// trail records the config write like every other yog step.
const ARM_STEP: &str = "arm-monitor";
const DISARM_STEP: &str = "disarm-monitor";

/// The family's one door (§8.5): the chokepoint hands the whole [`Verb`] here
/// and this decides nothing but which of the two bodies below runs it.
///
/// `workspace` and `agent` both arrive **already resolved** by the chokepoint's
/// two addressings (REMOTE §8; bl-49bc for the conversation), which is why the
/// verb's own name fields are not read here: the flag lands on the agent id the
/// resolution produced, never on the display name a peer may have spelled.
pub(super) fn dispatch(
    deps: &Deps,
    ts: &str,
    workspace: &Path,
    agent: &str,
    verb: &Verb,
) -> Result<Reply, String> {
    match verb {
        Verb::Arm { model, .. } => arm(deps, ts, workspace, Some(model)),
        Verb::Disarm { .. } => arm(deps, ts, workspace, None),
        Verb::Flag { reason, .. } => flag(deps, ts, workspace, agent, reason),
    }
}

/// Arm or disarm one workspace. `model` present is the arm — the cheap model
/// the check is pinned to — and absent is the disarm.
fn arm(deps: &Deps, ts: &str, workspace: &Path, model: Option<&str>) -> Result<Reply, String> {
    let path = deps.state_root.join(crate::app::cadence::CADENCE_YAML);
    let before = std::fs::read_to_string(&path).unwrap_or_default();
    let key = crate::nav::ws_key(workspace);
    let after = match model {
        Some(model) => arming::arm(&before, &key, model).ok_or_else(|| {
            format!(
                "{}: monitor: carries an inline value; edit it by hand and try again",
                crate::app::cadence::CADENCE_YAML
            )
        })?,
        None => arming::disarm(&before, &key),
    };
    std::fs::create_dir_all(&deps.state_root).map_err(|e| e.to_string())?;
    std::fs::write(&path, after).map_err(|e| e.to_string())?;
    if model.is_some() {
        seed_policy(&deps.state_root)?;
    }
    let step = if model.is_some() {
        ARM_STEP
    } else {
        DISARM_STEP
    };
    let done = OpEntry::step_done(
        ts.to_owned(),
        step,
        crate::nav::ws_key(workspace),
        Origin::World,
    );
    opslog::append(&deps.state_root, &done).map_err(|e| e.to_string())?;
    Ok(Reply::Armed {
        armed: model.is_some(),
    })
}

/// Write the policy file if it is not already there. Never overwritten: the
/// prompt is the operator's tuning surface, and re-arming must not silently
/// discard what they wrote into it.
fn seed_policy(state_root: &Path) -> Result<(), String> {
    let path = state_root.join(arming::PROMPT_FILE);
    if path.exists() {
        return Ok(());
    }
    std::fs::write(&path, arming::TEMPLATE).map_err(|e| e.to_string())
}

/// Raise an attention item on one conversation: one row, nothing else.
fn flag(
    deps: &Deps,
    ts: &str,
    workspace: &Path,
    agent: &str,
    reason: &str,
) -> Result<Reply, String> {
    let entry = flagging::raised(ts.to_owned(), workspace, agent, reason);
    opslog::append(&deps.state_root, &entry).map_err(|e| e.to_string())?;
    Ok(Reply::Flagged)
}

#[cfg(test)]
mod tests;
