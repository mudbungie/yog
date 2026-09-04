//! User actions issued through `cli_outbound` (ARCH §3.4 / §3.5).
//!
//! This root holds the §8.2 **enablement predicates that cross** — whether a
//! verb is offered for the conversation the boundary is answering about, each
//! refusing exactly what the underlying `litany` verb would. The **short,
//! piped, logged** verbs — message/stop/scan and `bl`
//! close/unclaim/create/update — live in [`verbs`], which appends each outcome
//! to `ops.jsonl` (§8.2, §15 Y16); the **detached** `litany prompt` (the new-root
//! launch, §8.1) is the one
//! [`start::execute_prompt`](crate::start::execute_prompt) path (Y17 moved it
//! onto `spawn_detached`, unifying the composer's new-prompt with the start
//! flow's final prompt — one detached, logged launch, no piped-and-drained
//! variant whose `Stream` drop would SIGTERM the loop on yog's exit).
//!
//! Predicate discipline (§8.2): **Stop** needs a Live/InFlight executor to
//! signal ([`stop_enabled`]); **Nudge** is its complement — the states with no
//! driver holding the lease, less the one shape litany reads as nothing-due
//! ([`nudge_enabled`]); **Message** is the resume gesture and works on *any*
//! selected agent (ARCH §2.9 — no resume verb; its gate's text half is a
//! composer's, so the conjunction is the seat's, bl-7cc8); **Scan** is
//! unconditional (offered for any focused workspace — no predicate). All three
//! ride the §8.5 agent answer as `nudgeable`/`stoppable`/`stop_children`
//! (`boundary::answer::agent`), so a seat is **told** rather than asked to
//! re-derive.
//!
//! **What a seat derives for itself is not held here.** A gate a row already
//! carries the input for is the seat's (REMOTE §9.4: a gate that is *not*
//! derivable from a row goes on the row). Two families left on that rule:
//!
//! - the composer's own RAM (bl-7cc8, deleted with the face it served,
//!   bl-7942): the drafts keyed by target (§5.3), the selection they were typed
//!   against, and the predicate that decided when a send cleared one.
//! - the form and ball-row rules (bl-33e9): whether a typed goal or ball title
//!   is blank, whether a typed work directory exists, and whether a §3.5
//!   `JoinState` permits assign / release / close. The first two judge a
//!   *seat's form inputs* before a fire, and the executor at the far end
//!   already refuses exactly what they predicted — the spawn boundary names a
//!   bad work directory ([`crate::cli_outbound::work_dir_fault`]) and `bl`
//!   itself refuses a claim of a bound ball — so the refusal at fire is the one
//!   home. The third is a one-line fold over `BoundBall::state`, which
//!   `Query::WorkspaceBalls` already answers.
//!
//! If a seat ever needs one of these facts stated instead of derived, it files
//! against yog and the reply is designed then.

pub mod verbs;

use crate::git_tree::{Agent, AgentState};

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

/// Nudge fires inference on the selected conversation from the state it is
/// already in (§8.2, bl-9bef) — `litany advance`, no text at all. So it is
/// the message gate without the content half, and [`stop_enabled`]'s
/// **complement** on state: a driver already holds the lease of a Live or
/// InFlight agent, and litany's own hop would take the clean no-op branch
/// (ARCH §2.11 Writer/driver totality). Offering it there would be a control
/// that fires and does nothing, which QUALITY H4 calls theater — so the two
/// verbs partition the states between them, Stop for the running ones and this
/// for the rest.
///
/// **One state at rest is exempt, for that same reason** (bl-fb87): a
/// conversation whose latest turn was cut off at the output limit
/// ([`Agent::truncated`], §4.4). Its transcript tail is an assistant turn with
/// no `tool_use`, and linked litany's `advance` derives `Warrant::NothingDue`
/// from exactly that — it releases the lease and exits without creating a
/// step. So the control would fire and do nothing, which is the theater the
/// partition above exists to prevent; the recovery is a Message, which needs
/// no gate because a deposit lands user-side and warrants a call.
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

#[cfg(test)]
mod tests;
