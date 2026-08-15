//! The §3.6 unmaking's two executors (§8.5), split from
//! [`dispatch`](super) at §12's line budget.
//!
//! The seam is what these two do that no other arm does: they **gate**. Every
//! other action in the chokepoint's table routes to an executor; these
//! re-derive the §3.6 confirmation *at fire time* — off the published
//! snapshot, never off the dialog that offered the verb — and refuse
//! fail-closed. Whichever frontend fires, the gate is this one.

use crate::delete;
use crate::ui_state::UiState;

use super::super::answer;
use super::super::reply::Reply;
use super::Deps;

/// The §3.6 unmaking, gated at fire time exactly as the dialog gates it:
/// yog's own named workspace, nothing live, the typed name armed — else the
/// refusal, with nothing attempted (fail-closed, whichever frontend fires).
pub(super) fn unmake(
    deps: &Deps,
    ui: &mut UiState,
    ts: &str,
    workspace: &std::path::Path,
    typed: &str,
) -> Result<Reply, String> {
    let confirm = answer::confirmation_of(&deps.snapshot, workspace)
        .ok_or_else(|| delete::DeleteError::Unnamed.to_string())?;
    if confirm.refused() {
        return Err(delete::DeleteError::Live(confirm.live.clone()).to_string());
    }
    if !confirm.armed(typed) {
        return Err(delete::DeleteError::NotArmed.to_string());
    }
    delete::execute(
        &delete::plan(
            &confirm,
            workspace,
            &crate::world::layout_under(&deps.yog_data_root).root,
            &deps.snapshot.projects,
        ),
        &deps.bl,
        ui,
        &deps.state_root,
        ts,
    )
    .map(|()| Reply::Deleted)
    .map_err(|e| e.to_string())
}

/// The §3.6 class one conversation deep (bl-f17a): gate liveness fail-closed
/// off the snapshot ("?" counts as live), then spawn the lernie verb — the
/// only lawful remover of agent state (I2). `--children` rides exactly when
/// `typed` re-states the conversation's name; the bare form otherwise, so a
/// subtree nobody confirmed is declined by the substrate's own
/// `HasDescendants` — the census is computed by the verb at the moment it
/// acts, never a stale dialog's. A clean removal prunes the dead subtree's
/// `ui.json` watermarks (§4.1) — the same not-mere-hygiene as the workspace
/// prune: a re-used id must not inherit a dead conversation's acknowledgements.
pub(super) fn delete_agent(
    deps: &Deps,
    ui: &mut UiState,
    ts: &str,
    workspace: &std::path::Path,
    agent: &str,
    typed: &str,
) -> Result<Reply, String> {
    let confirm = answer::agent_confirmation_of(&deps.snapshot, workspace, agent)
        .ok_or_else(|| delete::DeleteError::Unnamed.to_string())?;
    if confirm.refused() {
        return Err(delete::agent::live_refusal(&confirm.live));
    }
    let children = confirm.subtree_armed(typed);
    let outcome = delete::agent::spawn(
        &deps.lernie,
        &deps.state_root,
        ts,
        workspace,
        agent,
        children,
    )
    .map_err(|e| e.to_string())?;
    if outcome.ok() {
        ui.prune_agent(&crate::nav::ws_key(workspace), agent);
    }
    Ok(Reply::Outcome(outcome))
}
