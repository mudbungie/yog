//! User actions issued through `cli_outbound` (ARCH §3.4 / §3.5).
//!
//! This root holds every **enablement predicate** (pure, no egui): whether an
//! action is offered for the current selection. The **short, piped, logged**
//! verbs — message/stop/scan and `bl` close/unclaim/create/update — live in
//! [`verbs`], which appends each outcome to `ops.jsonl` (§8.2, §15 Y16); the
//! **detached** `lernie prompt` (the new-root launch, §8.1) is now the one
//! [`start::execute_prompt`](crate::start::execute_prompt) path (Y17 moved it
//! onto `spawn_detached`, unifying the composer's new-prompt with the start
//! flow's final prompt — one detached, logged launch, no piped-and-drained
//! variant whose `Stream` drop would SIGTERM the loop on yog's exit).
//!
//! Predicate discipline (§8.2): **Stop** needs a Live/InFlight executor to
//! signal ([`stop_enabled`]); **Nudge** is its complement — the states with no
//! driver holding the lease ([`nudge_enabled`]); **Message** is the resume
//! gesture and works on *any* selected agent ([`message_enabled`], ARCH §2.9 —
//! no resume verb);
//! **Close** needs the ball bound to a local workspace ([`close_enabled`] =
//! `JoinState::Bound`); **Unclaim**/release and **Move** need the same
//! ([`unclaim_enabled`], [`move_enabled`]); **Assign** needs a ready, unclaimed
//! ball ([`assign_enabled`] = `JoinState::ReadyStartable`) — each predicate
//! refuses exactly what the underlying `bl` verb would (§8.2/§3.5); **Scan** is
//! unconditional (offered for any focused workspace — no predicate). All are
//! functions of their inputs and carry no egui dependency, reusable by any future
//! frontend. One reads the filesystem — [`work_dir_refusal`], which asks whether a
//! typed work directory is there (bl-6191); it is a *question about an input*, not
//! state, and it is asked here rather than in the coverage-excluded form for the
//! same reason every other refusal is. Per §3.5 the UI holds no persistent state —
//! `ActionsState` is in-memory only and discarded on exit.

pub mod drafts;
pub mod verbs;

pub use drafts::{DraftKey, Drafts};

use crate::git_tree::{Agent, AgentState};
use crate::projects::join::JoinState;
use std::io;

/// Ephemeral action-surface state. Held in memory by the running
/// frontend and discarded on exit (ARCH §3.5: frontends hold no
/// persistent state).
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ActionsState {
    /// In-progress composer text, **keyed by the target it was typed for**
    /// ([`Drafts`], bl-a69a): the box is one widget whose verb follows the
    /// selection (§11), so its buffer is one draft *per* target — a new-root
    /// prompt in a workspace, or a message to an agent. Read and written back
    /// each frame through [`DraftKey::composer`]; a send clears its own key.
    /// Each draft is disabled while empty (or whitespace-only).
    pub drafts: Drafts,
    /// The **work directory** a new conversation is born with (§3.4 path rung,
    /// STORIES S2) — the §11 birth-config block's editable box (bl-7927), not the
    /// composer's. RAM (§3.5), and **pre-filled** at boot with the bare rung's own
    /// resolution (the operator's home dir), so it always states where the next
    /// start runs instead of meaning it by being blank. Set ⇒ a new prompt fires
    /// `Payload::Path` with this directory as the target preamble + driver cwd;
    /// emptied by hand ⇒ the bare rung, which resolves to that same home.
    ///
    /// `Default` leaves it empty because `ActionsState` cannot read the env; the
    /// seed is folded once in `ShellState::new`, the one boundary that can.
    pub path_dir: String,
    /// User-selected agent id (§2.3 — the id is the address; `lernie
    /// stop`/`message` take it, not the `agents/*` ref). Gates Stop (which
    /// also needs the agent live) and Message.
    pub selected_branch: Option<String>,
    /// Whether Stop should cascade to the selected agent's descendants
    /// (§8.2 `--stop-children`; the §11 composer's children checkbox). Offered
    /// only when the agent actually has descendants ([`stop_children_offered`]).
    pub stop_children: bool,
}

/// The §11 birth-config work-directory field's verdict (bl-6191): the refusal
/// sentence when `typed` names something that is not an existing directory, or
/// `None` when the next start may run there.
///
/// **Empty is lawful, not a refusal** — an emptied box is the bare rung, which
/// resolves to the same home the pre-filled default states ([`ActionsState::path_dir`],
/// `shell::fire`), so it is the general path with no input rather than a case.
///
/// The question itself is the **spawn boundary's**
/// ([`crate::cli_outbound::work_dir_fault`]), asked here before Enter fires
/// anything instead of after a fork has already misattributed it: `std::process`
/// reports a bad `current_dir` as ENOENT against the *program*, so the operator
/// who typed a bad directory was told their binary was missing. One reading, one
/// sentence, both layers.
pub fn work_dir_refusal(typed: &str) -> Option<String> {
    let dir = typed.trim();
    if dir.is_empty() {
        return None;
    }
    crate::cli_outbound::work_dir_fault(std::path::Path::new(dir)).map(|e| e.to_string())
}

/// True iff `goal` is a goal at all (§8.1): at least one non-whitespace
/// character. **The one definition of "blank", shared by every site a goal can
/// fire from** — the composer's Enter and the bootstrap box (through
/// [`new_prompt_enabled`], which adds the work-directory half), and the start
/// pane's Send / its §11 Enter binding, which have no directory to ask about
/// (the planner resolved their cwd).
///
/// It also decides whether a start **opens** a goal draft at all: a rung whose
/// prefill is blank composed nothing to edit, so there is nothing to draft
/// (§3.4's table — the bare rung's prefill is "none"). Before bl-9acf the raise
/// opened one anyway, and its Send fired the identity preamble followed by
/// nothing onto the wire — spend with no instruction behind it.
pub fn goal_present(goal: &str) -> bool {
    !goal.trim().is_empty()
}

/// True iff the composer's Enter may fire a new conversation (§11): there is
/// something to say ([`goal_present`]) **and** somewhere lawful to say it
/// ([`work_dir_refusal`] on the birth block's work directory). One predicate
/// rather than two because they arm one gesture; the field's own red flag is
/// what tells the operator *which* half refused.
pub fn new_prompt_enabled(input: &str, work_dir: &str) -> bool {
    goal_present(input) && work_dir_refusal(work_dir).is_none()
}

/// A new ball's Create & Start is offered iff its title is non-blank (§8.1): a
/// ball with no title has nothing to name the work. A distinct §3.5 rule from
/// [`new_prompt_enabled`] though the current bodies coincide (as [`close_enabled`]
/// / [`unclaim_enabled`] / [`move_enabled`] share `Bound`) — each names the rule
/// its `bl`/start path enforces. Covered here, not inlined in coverage-excluded
/// shell glue.
pub fn create_ball_enabled(title: &str) -> bool {
    !title.trim().is_empty()
}

/// The new-ball form's two placeholder hints (§8.1, bl-b2ed). The form is two
/// bare `TextEdit`s and a button; empty, nothing distinguished the title box
/// from the body box, nor said the second was multiline. The hints are the
/// affordance, so they live here beside the form's other rule
/// ([`create_ball_enabled`]) rather than as literals in coverage-excluded shell
/// glue. The body's hint states the goal the way the composer's does ("say what
/// you want done"), not a bare field name — a body that is not a definition of
/// done is what the start flow hands the agent as its preamble (§8.1).
pub fn new_ball_hints() -> NewBallHints {
    NewBallHints {
        title: "title".to_owned(),
        body: "body — what done looks like".to_owned(),
    }
}

/// The [`new_ball_hints`] pair, named so the form cannot swap them.
#[derive(Debug, PartialEq, Eq)]
pub struct NewBallHints {
    /// Placeholder for the single-line title box.
    pub title: String,
    /// Placeholder for the multiline body box.
    pub body: String,
}

/// A composer draft is RAM until it is cleanly *deposited* (§5.3, STORIES S1): a
/// message clears its draft iff the verb both launched (`Ok`) and exited 0
/// ([`Outcome::ok`](verbs::Outcome::ok)) — every failure keeps the text so the
/// operator can retry. Pinned here as a covered predicate, never in coverage-
/// excluded shell glue, so a regression that ate the draft on failure fails a
/// test instead of slipping through.
pub fn draft_clears(result: &io::Result<verbs::Outcome>) -> bool {
    result.as_ref().is_ok_and(verbs::Outcome::ok)
}

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
/// Enabled iff an agent is selected, present in `agents`, and Quiescent or
/// Stopped. `false` for no selection and for an id absent from the set.
pub fn nudge_enabled(selected: Option<&str>, agents: &[Agent]) -> bool {
    selected.is_some_and(|name| {
        agents.iter().any(|a| {
            a.agent_id == name && matches!(a.state, AgentState::Quiescent | AgentState::Stopped)
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

/// Move is offered iff the ball is bound to a local workspace —
/// [`JoinState::Bound`] (§8.2, §3.5). Move is unclaim-then-claim (release + assign
/// to another workspace): only a ball this yog owns can be re-homed, so an
/// unclaimed or claimed-elsewhere ball refuses (what `bl unclaim` would refuse).
pub fn move_enabled(state: JoinState) -> bool {
    matches!(state, JoinState::Bound)
}

#[cfg(test)]
mod tests;
