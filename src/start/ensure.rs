//! The workspace half of the start flow's executors (DESIGN §8.1, §8.6): the
//! idempotent `lernie new` ensure and the capability-control authoring that
//! runs **outside** the create skip so a
//! workspace made a moment ago and one made last week both converge to a
//! `config/default` naming the control shim.
//!
//! Split from [`super::exec`] at the seam the tests already used: the `bl`-facing
//! executors (create / claim / cross-check) are that file's concern, the
//! workspace's existence and its policy are this one's.

use super::exec::{CONTROL, Deps, MKDIR, NEW, REPO_MARK, StartError, verb_ok};
use crate::actions::verbs::{self, log_step_failure};
use crate::cli_outbound::Cli;
use crate::control;
use crate::opslog::Origin;
use crate::world::Layout;
use std::path::Path;

/// Ensure the bound workspace exists (§8.1, §3.1): skip when `<workspace>/repo.git`
/// is present (resume is the same path as opening). Otherwise `mkdir -p` the
/// parent chain — a failure logs a `["yog-step","mkdir"]` row (Z5) before it
/// returns — and `lernie new <workspace>` piped + opslog'd.
///
/// **Birth judges only what exists at birth (bl-00ee).** bl-c3a9 gated the
/// world's birth template here, against brazen's provider table; §16.2's wall
/// made that table the *workspace's*, born empty with the workspace and filled
/// by the operator's per-workspace sign-in afterwards — so the gate read a fact
/// that cannot exist yet and refused every workspace whose template names a row
/// brazen does not ship. The judgement did not move surface: the same
/// `is_unknown_row` faults the provider field in the §9.5 config pane, where
/// the workspace's own wall is a fact, and a row that is still dead at the fire
/// surfaces as the §8.3 auth-shaped step failure with Login one click away.
pub fn execute_ensure_workspace(
    deps: &Deps,
    ts: &str,
    workspace: &Path,
    layout: &Layout,
    origin: Origin,
) -> Result<bool, StartError> {
    let (lernie, state_root) = (&deps.lernie, deps.state_root.as_path());
    let created = create_workspace(lernie, state_root, ts, workspace, origin)?;
    // The capability control is authored **after** the create and **outside**
    // its skip (§8.6): a workspace made a moment ago and one made last week
    // both converge to a `config/default` naming the control shim, so every
    // agent forked from here on is adjudicated. Converged, not branched — the
    // steady state reads one file out of git and spawns nothing.
    let shim = crate::world::tools::control_path(&layout.tools);
    let authored = control::author::ensure_controlled(
        lernie,
        workspace,
        &shim,
        &deps.yog_binary,
        state_root,
        ts,
        origin,
    )?;
    if let Some(entry) = authored.filter(|e| e.exit != 0) {
        log_step_failure(state_root, ts, workspace, CONTROL, &entry.stderr, origin)?;
        return Err(StartError::Control(entry.stderr));
    }
    Ok(created)
}

/// The create half of [`execute_ensure_workspace`]: the parent chain and
/// `lernie new`. Skipped whole for a workspace that already exists — resume is
/// the same path as opening.
fn create_workspace(
    lernie: &Cli,
    state_root: &Path,
    ts: &str,
    workspace: &Path,
    origin: Origin,
) -> Result<bool, StartError> {
    if workspace.join(REPO_MARK).exists() {
        return Ok(false);
    }
    let parent = workspace.parent().unwrap_or(workspace);
    if let Err(e) = std::fs::create_dir_all(parent) {
        log_step_failure(state_root, ts, parent, MKDIR, &e.to_string(), origin)?;
        return Err(StartError::Io(e));
    }
    let ws_s = workspace.to_string_lossy();
    verb_ok(
        verbs::run_logged(lernie, state_root, ts, parent, &[NEW, &ws_s], origin)?,
        NEW,
    )?;
    Ok(true)
}
