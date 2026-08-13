//! What **one agent** is doing right now (DESIGN §5.1 #28b, §11) — the finest
//! live-activity fact yog derives, and the whole vocabulary of the §11 live
//! mark: one circle per agent, hue = its [`Doing`].
//!
//! Five states, total over the snapshot, and every one of them a **query** over
//! facts the tick already carries — the agent state (§5.1 #9: the executor lock
//! plus the open `response.json` fd), the latest step's tool records (#10), and
//! the kind of the last content delta in that same response file (#28b). No
//! flag is stored and none could be: yog observes neither a start nor an end,
//! only the disk at this tick.
//!
//! **This refines [`super::flight`], it does not compete with it.** The §5.1
//! #28 class is a fact about a *conversation's subtree*; this is a fact about
//! *one agent*, and the three model-call states here are the one thing #28
//! calls `Inference`. So [`super::flight::flight`] is written as a **fold over
//! this** — one authority for "is a model call streaming", one for "is a tool
//! running" — and the operator's `inference > tools > subagents` priority is
//! read off it rather than decided twice.
//!
//! **Idle is not "stopped".** The mark's green says *nothing is happening on
//! this seat right now*, which is equally true of a quiescent agent awaiting a
//! message, a killed one, and a seat with no agent in it at all. Whether a
//! branch ended well is a different question with its own carriers (the §3.5
//! state badge, the §6 marks); asking this one to answer it too would put two
//! facts on one circle.

use crate::git_tree::{Agent, AgentState, Delta, ToolCallState};

/// What one agent is doing, right now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Doing {
    /// A model call is open and **nothing has come back yet** — the request
    /// went out, the stream carries no content delta (§5.1 #28b). This is the
    /// wait on the API, the one segment of a call that used to look identical
    /// to a call already answering.
    Waiting,
    /// The model is thinking: the last content delta was a `thinking_delta`,
    /// which displays nothing, so without this the tail looks stalled.
    Thinking,
    /// The model is answering: the last content delta was a `text_delta`.
    Inference,
    /// A tool is executing under this agent — `input.json` landed, no
    /// `output.json` yet (§5.1 #10), under a driver still there to have
    /// started it.
    Tools,
    /// Nothing is in flight on this agent.
    Idle,
}

impl Doing {
    /// Is this one of the three states of an open model call? That union is
    /// exactly §5.1 #28's `Inference` class, which is why the fold below can
    /// ask it as one question.
    pub fn is_model_call(self) -> bool {
        matches!(self, Self::Waiting | Self::Thinking | Self::Inference)
    }
}

/// What `agent` is doing (§5.1 #28b).
///
/// The order is #28's, not a new one: an open model call outranks a tool,
/// because a model call is the more immediate thing and the priority is the
/// answer to "both at once" rather than a defect to design around. Under an
/// open call, the last delta splits it three ways — nothing back, thinking,
/// answering — and `None` meaning *waiting* is the general path with an empty
/// stream, not a case.
///
/// **A tool counts only under a live driver.** `output.json` never lands for a
/// tool whose driver died mid-call, so an unguarded reading would light that
/// seat forever; requiring the driver dissolves the stale record with an
/// invariant instead of an expiry rule.
pub fn doing(agent: &Agent) -> Doing {
    if agent.state == AgentState::InFlight {
        return match agent.stream.last_delta {
            None => Doing::Waiting,
            Some(Delta::Thinking) => Doing::Thinking,
            Some(Delta::Text) => Doing::Inference,
        };
    }
    if super::running(agent.state)
        && agent
            .tool_calls
            .iter()
            .any(|call| call.state == ToolCallState::InFlight)
    {
        return Doing::Tools;
    }
    Doing::Idle
}

/// One seat of the §11 live mark: an agent, named as every other seat names it
/// (§3.3), and what it is doing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Seat {
    pub name: String,
    pub doing: Doing,
}

/// The mark's seats for the conversation rooted at `root_id`: the **eye first**
/// — the agent the operator is talking to — then its subagents in §2.3 descent
/// order. An id that roots nothing yields no seats at all, which is what an
/// operator with no conversation open is looking at: the mark at rest.
///
/// The list is **not capped here.** How many circles the mark has is the mark's
/// own fact (`theme::icon::NODE_SEATS`), and a view-model that pre-truncated to
/// it would leave the seat that says *how many were dropped* with nothing to
/// count.
pub fn seats(agents: &[Agent], root_id: &str) -> Vec<Seat> {
    super::members(agents, root_id)
        .iter()
        .filter_map(|row| agents.get(row.index))
        .map(|agent| Seat {
            name: super::member_title(agent),
            doing: doing(agent),
        })
        .collect()
}
