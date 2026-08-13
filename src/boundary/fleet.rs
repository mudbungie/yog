//! The armed loop's one boundary executor (VISION §4.3, rung V4 item 2):
//! arming. Split out of [`dispatch`](super::dispatch) for the same reason every
//! other family is — the chokepoint is a table, and a table stops being one
//! when arms carry bodies.
//!
//! **Arming is a config write and nothing else.** It adds (or deletes) one
//! `cadence.yaml` `fleet:` entry. There is no loop to start: the pilot
//! ([`crate::fleet::pilot`]) is always running and finds nothing to do until an
//! entry exists, so arming cannot half-apply and disarming cannot leave a
//! thread behind — the same shape the monitor's arm takes, one file over.
//!
//! **It seeds nothing and it spawns nothing.** The monitor's arm seeds a policy
//! file because its mechanism is a prompt; this one's policy is two numbers in
//! the entry it just wrote. And the first spawn is the *loop's*, on its next
//! tick, through the ordinary start door with the ordinary ceiling in front of
//! it — an arm that also spawned would be two instructions under one word.

use std::path::Path;

use crate::fleet::{Verb, arming};
use crate::opslog::{self, OpEntry, Origin};

use super::dispatch::Deps;
use super::reply::Reply;

/// The step names the arm/disarm gesture logs its own completion under, so the
/// trail records the config write like every other yog step.
const ARM_STEP: &str = "arm-fleet";
const DISARM_STEP: &str = "disarm-fleet";

/// The family's one door (§8.5): the chokepoint hands the whole [`Verb`] here
/// and this decides nothing but which way the one body runs.
pub(super) fn dispatch(deps: &Deps, ts: &str, verb: &Verb) -> Result<Reply, String> {
    match verb {
        Verb::Arm {
            workspace,
            project,
            cap,
        } => arm(deps, ts, workspace, Some((project.as_path(), *cap))),
        Verb::Disarm { workspace } => arm(deps, ts, workspace, None),
    }
}

/// Arm or disarm one workspace. The pair present is the arm — the project the
/// loop takes work from and the cap it may hold — and absent is the disarm.
fn arm(
    deps: &Deps,
    ts: &str,
    workspace: &Path,
    policy: Option<(&Path, usize)>,
) -> Result<Reply, String> {
    let path = deps.state_root.join(crate::app::cadence::CADENCE_YAML);
    let before = std::fs::read_to_string(&path).unwrap_or_default();
    let key = crate::nav::ws_key(workspace);
    let after = match policy {
        Some((project, cap)) => arming::arm(&before, &key, &crate::nav::ws_key(project), cap)
            .ok_or_else(|| {
                format!(
                    "{}: fleet: carries an inline value; edit it by hand and try again",
                    crate::app::cadence::CADENCE_YAML
                )
            })?,
        None => arming::disarm(&before, &key),
    };
    std::fs::create_dir_all(&deps.state_root).map_err(|e| e.to_string())?;
    std::fs::write(&path, after).map_err(|e| e.to_string())?;
    let step = if policy.is_some() {
        ARM_STEP
    } else {
        DISARM_STEP
    };
    let done = OpEntry::step_done(ts.to_owned(), step, key, Origin::Balls);
    opslog::append(&deps.state_root, &done).map_err(|e| e.to_string())?;
    Ok(Reply::Armed {
        armed: policy.is_some(),
    })
}

#[cfg(test)]
mod tests;
