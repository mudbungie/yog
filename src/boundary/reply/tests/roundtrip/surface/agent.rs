//! The §11 conversation seat (REMOTE §9.4, bl-1eb0): its own file beside the
//! rest of the conversation-addressed family, on the seam its spelling already
//! takes ([`agent`](crate::boundary::reply)).
//!
//! Two fixtures, because every optional key on this payload is absent-not-null:
//! one wearing every §6 mark with a class in flight, and one at rest wearing
//! none — the arm where `marks` and `flight` are keys the encoder declines to
//! write at all.

use super::super::super::super::Reply;
use crate::boundary::answer::agent::AgentView;
use crate::git_tree::{AgentMark, AgentState};
use crate::nav::convs::Flight;

pub(super) fn agent() -> Vec<Reply> {
    vec![
        Reply::Agent(AgentView {
            agent_id: "r-0-c-1".to_owned(),
            root: "r-0".to_owned(),
            ancestors: vec!["r-0".to_owned()],
            name: "pennant".to_owned(),
            display_only: true,
            tip: "a".repeat(40),
            state: AgentState::InFlight,
            // Every arm of the mark table, so a transposed token cannot pass.
            marks: vec![
                AgentMark::Notified,
                AgentMark::BudgetExhausted,
                AgentMark::Conflicted,
                AgentMark::Held,
                AgentMark::Abandoned,
            ],
            held: Some(crate::control::hold::Held {
                tool_use_id: "toolu_1".to_owned(),
                tool: "Bash".to_owned(),
                reason: "unconfined".to_owned(),
            }),
            flight: Some(Flight::Tools),
            present: true,
            nudgeable: false,
            stoppable: true,
            stop_children: true,
        }),
        Reply::Agent(AgentView {
            agent_id: "r-0".to_owned(),
            root: "r-0".to_owned(),
            ancestors: vec![],
            name: "r-0".to_owned(),
            display_only: false,
            tip: String::new(),
            state: AgentState::Stopped,
            marks: vec![],
            held: None,
            flight: None,
            present: false,
            nudgeable: false,
            stoppable: false,
            stop_children: false,
        }),
    ]
}
