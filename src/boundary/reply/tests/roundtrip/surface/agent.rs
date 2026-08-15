//! The conversation-addressed answers that are **one object rather than a
//! listing**: the §11 seat's own view of its selection (REMOTE §9.4, bl-1eb0)
//! and the policy that selection is frozen on (bl-13f9). Their own file beside
//! the rest of the family, on the seam their spellings already take — a flat
//! envelope of scalars, never a `rows` array.
//!
//! Two fixtures each, because every optional key on both payloads is
//! absent-not-null: an agent wearing every §6 mark with a class in flight, a
//! full live mark and a strip, and one at rest wearing none (the arm where
//! `marks`, `flight`, `seats` and `strip` are keys the encoder declines to
//! write at all), and a governing config still standing at
//! a lineage's tip beside the ordinary frozen one that has been left behind.

use super::super::super::super::Reply;
use crate::boundary::answer::agent::AgentView;
use crate::config_edit::branch::GoverningConfig;
use crate::git_tree::{AgentMark, AgentState};
use crate::nav::convs::{Doing, Flight, FlightStrip, Seat};

fn seat(name: &str, doing: Doing) -> Seat {
    Seat {
        name: name.to_owned(),
        doing,
    }
}

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
            // Every arm of the §5.1 #28b doing table, so a transposed token
            // cannot pass here either.
            seats: vec![
                seat("pennant", Doing::Waiting),
                seat("kid-a", Doing::Thinking),
                seat("kid-b", Doing::Inference),
                seat("kid-c", Doing::Tools),
                seat("kid-d", Doing::Idle),
            ],
            strip: Some(FlightStrip {
                class: Flight::Tools,
                facts: "Bash · 5s".to_owned(),
            }),
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
            seats: vec![],
            strip: None,
        }),
        Reply::Governing(GoverningConfig {
            oid: "b".repeat(40),
            short_oid: "bbbbbbbb".to_owned(),
            branch_name_if_tip_of_one: Some("default".to_owned()),
            files: vec!["workflow.yaml".to_owned(), "souls/base.md".to_owned()],
        }),
        // The ordinary frozen case: the lineage has advanced past the commit,
        // so it names no branch and the key is one the encoder writes as null.
        Reply::Governing(GoverningConfig {
            oid: "c".repeat(40),
            short_oid: "cccccccc".to_owned(),
            branch_name_if_tip_of_one: None,
            files: vec![],
        }),
    ]
}
