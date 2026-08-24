//! The workspace half of the start flow's executors (DESIGN §8.1, §8.6, §3.7):
//! the idempotent `lernie new` ensure and the **policy convergence** that runs
//! **outside** the create skip, so a workspace made a moment ago and one made
//! last week both converge to a config lineage naming the control shim and
//! composing frozen project instructions — **the lineage the drone will fork
//! off** (§8.7), which is `config/default` unless the ball's tags named another.
//!
//! Split from [`super::exec`] at the seam the tests already used: the `bl`-facing
//! executors (create / claim / cross-check) are that file's concern, the
//! workspace's existence and its policy are this one's.
//!
//! **Two files, one drive** (§3.7 item 4). §8.6's `tool_control:` block and
//! §3.7's `instructions/**` glob are two control files of one yog policy, and
//! each owns its own fixed point ([`control::author::workflow_drift`],
//! [`manifest::drift`]) and knows nothing of the other. This module collects
//! whichever drifted and converges them in a **single** `lernie config` pass —
//! one checkout, one commit, one ops row. With neither drifted nothing is
//! staged and nothing spawns, which is the steady state of every start after
//! the first.

use super::exec::{CONTROL, Deps, MKDIR, NEW, REPO_MARK, StartError, verb_ok};
use super::instructions::manifest;
use crate::actions::verbs::{self, log_step_failure};
use crate::cli_outbound::Cli;
use crate::config_edit::branch::edit::{
    DraftFile, EditOrigin, EditPlan, drive, next_nonce, stage_files,
};
use crate::control;
use crate::opslog::{OpEntry, Origin};
use crate::world::Layout;
use crate::xdg::stage_root_under;
use std::io;
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
    config: &str,
    layout: &Layout,
    origin: Origin,
) -> Result<bool, StartError> {
    let (lernie, state_root) = (&deps.lernie, deps.state_root.as_path());
    let created = create_workspace(lernie, state_root, ts, workspace, origin)?;
    // yog's policy is authored **after** the create and **outside** its skip
    // (§8.6, §3.7): a workspace made a moment ago and one made last week both
    // converge to a config lineage naming the control shim and composing
    // frozen instructions, so every agent forked from here on is adjudicated
    // and instructed. On `config`, not on `default` (§8.7): the branch this
    // start will fork its drone off is the only one whose policy governs it. Converged, not branched — the steady state reads two
    // files out of git and spawns nothing.
    let shim = crate::world::tools::control_path(&layout.tools);
    let drafts: Vec<DraftFile> = [
        control::author::workflow_drift(workspace, config, &shim),
        manifest::drift(workspace, config),
    ]
    .into_iter()
    .flatten()
    .collect();
    let authored = converge(deps, workspace, config, &drafts, state_root, ts, origin)?;
    if let Some(entry) = authored.filter(|e| e.exit != 0) {
        log_step_failure(state_root, ts, workspace, CONTROL, &entry.stderr, origin)?;
        return Err(StartError::Control(entry.stderr));
    }
    Ok(created)
}

/// Stage `drafts` and drive the one `lernie config <ws> <config>` pass that
/// commits them (§9.3 — the only lawful writer of `config/*`, so yog never
/// writes inside a workspace itself). `None` when nothing drifted: no staging
/// dir, no spawn, no ops row.
///
/// Attributed to the surface that asked for the start (bl-48f8): being born
/// uncontrolled or uninstructed is that start's failure, and it banners where
/// the start was offered.
fn converge(
    deps: &Deps,
    workspace: &Path,
    config: &str,
    drafts: &[DraftFile],
    state_root: &Path,
    ts: &str,
    origin: Origin,
) -> io::Result<Option<OpEntry>> {
    if drafts.is_empty() {
        return Ok(None);
    }
    let staging = stage_files(&stage_root_under(state_root), &next_nonce(), drafts)?;
    let plan = EditPlan::compose(
        &deps.yog_binary,
        workspace,
        config,
        &EditOrigin::Advance,
        &staging,
    );
    Ok(Some(drive(
        &deps.lernie,
        workspace,
        &plan,
        ts,
        state_root,
        origin,
    )))
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
