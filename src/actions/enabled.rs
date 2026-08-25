//! **Whether a verb is offered for the current selection** (§8.2, §3.5) — the
//! enablement predicates over an agent or a joined ball, each refusing exactly
//! what the underlying `lernie`/`bl` verb would.
//!
//! Split from the composer's own rules at §12's budget on the seam the module
//! doc already draws: [`super`] asks whether there is anything to *fire* (a
//! goal, a title, a lawful work directory), this asks whether *this selection*
//! permits the verb. Pure and egui-free like the rest, so a refusal is covered
//! here rather than in coverage-excluded shell glue.

use crate::git_tree::{Agent, AgentState};
use crate::projects::join::JoinState;

/// True iff `selected_branch` names an agent (by id, §2.3) in `agents`
/// that is **live** — [`AgentState::Live`] or [`AgentState::InFlight`],
/// the two states where a driver holds the executor lock (§2.11). Stop
/// targets a live executor (§2.9), and it is wanted precisely during tool
/// execution (a `Live` agent between model calls), not only mid-model-call
/// — so both live states are stoppable. Returns `false` for `None`, for an
/// id not present, and for a `Quiescent` or `Stopped` agent (no executor
/// to signal).
pub fn stop_enabled(selected_branch: Option<&str>, agents: &[Agent]) -> bool {
    let Some(name) = selected_branch else {
        return false;
    };
    agents
        .iter()
        .any(|a| a.agent_id == name && matches!(a.state, AgentState::Live | AgentState::InFlight))
}

/// Message is the resume gesture (§8.2, ARCH §2.9: no resume verb — the
/// deposit restarts a driver). Unlike [`stop_enabled`] it is *not* gated on
/// agent state: a Quiescent or Stopped agent is precisely what you message to
/// continue it. Enabled iff the target is a conversation the world carries
/// (`present` — the §11 seat's own fact since bl-1eb0, so no caller re-walks
/// the agent set to ask) and the composer text is non-blank.
pub fn message_enabled(present: bool, content: &str) -> bool {
    present && !content.trim().is_empty()
}

/// Nudge fires inference on the selected conversation from the state it is
/// already in (§8.2, bl-9bef) — `lernie advance`, no text at all. So it is
/// [`message_enabled`] without the content half, and [`stop_enabled`]'s
/// **complement** on state: a driver already holds the lease of a Live or
/// InFlight agent, and lernie's own hop would take the clean no-op branch
/// (ARCH §2.11 Writer/driver totality). Offering it there would be a control
/// that fires and does nothing, which QUALITY H4 calls theater — so the two
/// verbs partition the states between them, Stop for the running ones and this
/// for the rest.
///
/// **One state at rest is exempt, for that same reason** (bl-fb87): a
/// conversation whose latest turn was cut off at the output limit
/// ([`Agent::truncated`], §4.4). Its transcript tail is an assistant turn with
/// no `tool_use`, and linked lernie's `advance` derives `Warrant::NothingDue`
/// from exactly that — it releases the lease and exits without creating a
/// step. So the control would fire and do nothing, which is the theater the
/// partition above exists to prevent; the recovery is [`message_enabled`],
/// which needs no gate because a deposit lands user-side and warrants a call.
/// The §7.3 step wound says so in words at the same moment
/// ([`crate::steps_view::OUTPUT_LIMIT`]).
///
/// Enabled iff an agent is selected, present in `agents`, Quiescent or
/// Stopped, and not truncated. `false` for no selection and for an id absent
/// from the set.
pub fn nudge_enabled(selected: Option<&str>, agents: &[Agent]) -> bool {
    selected.is_some_and(|name| {
        agents.iter().any(|a| {
            a.agent_id == name
                && !a.truncated
                && matches!(a.state, AgentState::Quiescent | AgentState::Stopped)
        })
    })
}

/// Stop offers `--stop-children` iff `agent_id` has a descendant in the id set
/// (§8.2) — another agent whose id extends `<agent_id>-…` (the hyphenated
/// descent, §2.3). Pure over the ids; no lone agent offers it.
pub fn stop_children_offered(agent_id: &str, agents: &[Agent]) -> bool {
    let prefix = format!("{agent_id}-");
    agents
        .iter()
        .any(|a| a.agent_id != agent_id && a.agent_id.starts_with(&prefix))
}

/// Close is offered iff the focused ball is bound to a local workspace — exactly
/// [`JoinState::Bound`] (§8.2, §3.5): the ball's claimant names a workspace here,
/// so there is a loop to deliver from.
pub fn close_enabled(state: JoinState) -> bool {
    matches!(state, JoinState::Bound)
}

/// Unclaim is offered iff the focused ball is bound to a local workspace —
/// [`JoinState::Bound`] (§8.2, §3.5). Release is `bl unclaim <id> --as <name>`.
pub fn unclaim_enabled(state: JoinState) -> bool {
    matches!(state, JoinState::Bound)
}

/// Assign is offered iff the ball is **ready and unclaimed** —
/// [`JoinState::ReadyStartable`] (§8.2, §3.5: "ready ball → ▶ Start or Assign to
/// an existing workspace"). Assign binds an unbound ball, so a bound / blocked /
/// claimed-elsewhere / delivered ball refuses — exactly what `bl claim` refuses.
pub fn assign_enabled(state: JoinState) -> bool {
    matches!(state, JoinState::ReadyStartable)
}
