//! `attention` test harness: the shared agent builder and `Seen` closures,
//! split into [`signals`] (the per-agent predicate) and [`roster`] (rollups,
//! sort, jump).

use super::*;
use crate::git_tree::{Agent, AgentState};

mod roster;
mod signals;

/// A default agent: `Live`, no marks, no mail, a per-id tip oid (so a
/// rest-watermark test can target this branch's tip). Running is the only state
/// that stirs nothing on its own — since bl-2194 an agent **at rest** at an
/// unseen tip is rule 2 firing, so a calm baseline is either running or an
/// at-rest agent whose tip is acked ([`tips_acked`]).
fn agent(id: &str) -> Agent {
    Agent {
        branch_name: format!("agents/{id}"),
        agent_id: id.to_string(),
        tip_oid: format!("tip-{id}"),
        tip_short_oid: "tip".into(),
        tip_timestamp_unix: 0,
        last_action_unix: 0,
        messages: 0,
        steps: vec![],
        preview: None,
        stream: crate::git_tree::Stream::default(),
        tool_calls: vec![],
        state: AgentState::Live,
        state_uncertain: false,
        truncated: false,
        refused: false,
        pending: vec![],
        conflicted_oid: None,
        budget_oid: None,
        abandoned_oid: None,
        notify_oid: None,
        held: None,
        goal_ball: None,
        name: None,
        goal_name: None,
        call_start_unix: None,
    }
}

/// A `Seen` closure that acknowledges exactly one oid for one kind — every
/// other query is unseen.
fn acked(kind: SeenKind, oid: &'static str) -> impl Fn(SeenKind, &str, &str, &str) -> bool {
    move |k, _w, _a, o| k == kind && o == oid
}

/// The `Seen` closure that acknowledges nothing.
fn nothing(_: SeenKind, _: &str, _: &str, _: &str) -> bool {
    false
}

/// A `Seen` closure that acknowledges the rest watermark of every listed agent
/// (its `tip-<id>` oid) — the post-bl-2194 *idle* agent: at rest at a tip you
/// have already seen, which is the only way rest stays quiet.
fn tips_acked(ids: &'static [&'static str]) -> impl Fn(SeenKind, &str, &str, &str) -> bool {
    move |k, _w, _a, o| k == SeenKind::Stopped && ids.iter().any(|id| o == format!("tip-{id}"))
}
