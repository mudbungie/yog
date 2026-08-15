//! Table tests for the conversation-list view-model (§11, §15 Z9): the
//! structural layer — which agents form a conversation, how it resolves from a
//! member, and whether it is live. The row projection's tables are in [`rows`].

use super::*;
use crate::ui_state::SeenKind;

/// The per-agent §5.1 #28b fact the mark paints, one budget below this one.
mod doing;

/// The §11 unfold — visible rows, the walks, the reveal — one budget below.
mod expand;

/// The §11 live-activity classes and their priority, one budget below this one.
mod flight;

/// The §3.3 display ladder — its own file, one budget below this one.
mod name;

/// The §11 row projection's own tables, one budget below this one.
mod rows;

/// The **selection's** fold out of the answered forest (REMOTE §9.7), one
/// budget below this one.
mod select;

/// The §11 bottom in-flight strip's own tables, one budget below this one.
mod strip;

/// The §3.3 ladder as a seat holds it (REMOTE §9.4), one budget below.
mod titles;

/// An agent holding one tool call on its latest step, `name` being what
/// `input.json` named it (`None` = a record yog could read no name from) and
/// `start` its landing stamp (`None` = a record yog could not stat).
fn named_tool(
    id: &str,
    agent_state: AgentState,
    tool: crate::git_tree::ToolCallState,
    name: Option<&str>,
    start: Option<i64>,
) -> Agent {
    let mut a = agent(id, agent_state, 1);
    a.tool_calls = vec![crate::git_tree::ToolCall {
        tool_id: "toolu_1".into(),
        name: name.map(str::to_owned),
        start_unix: start,
        state: tool,
    }];
    a
}

fn agent(id: &str, state: AgentState, ts: i64) -> Agent {
    Agent {
        branch_name: format!("agents/{id}"),
        agent_id: id.to_string(),
        tip_oid: "a".repeat(40),
        tip_short_oid: "aaaaaaaa".into(),
        tip_timestamp_unix: ts,
        last_action_unix: ts,
        messages: 0,
        steps: vec![],
        preview: None,
        stream: crate::git_tree::Stream::default(),
        tool_calls: vec![],
        state,
        state_uncertain: false,
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

/// The all-unseen closure: every watermark reads unacknowledged.
fn unseen(_: SeenKind, _: &str, _: &str, _: &str) -> bool {
    false
}

/// A ball resolver that knows nothing: any stamped id renders from source 1 only
/// (no join facts). Never actually invoked by the goal_ball-`None` fixtures.
fn plain(id: &str) -> ConvBall {
    ConvBall {
        id: id.to_owned(),
        state: None,
        title: None,
        badge: None,
    }
}

#[test]
fn members_returns_the_subtree_in_descent_order() {
    let agents = [
        agent("r1-0", AgentState::Quiescent, 1),
        agent("r1-0-b-1", AgentState::Quiescent, 2),
        agent("r1-0-a-1", AgentState::Quiescent, 3),
        agent("r2-0", AgentState::Quiescent, 4),
    ];
    let rows = members(&agents, "r1-0");
    let ids: Vec<&str> = rows
        .iter()
        .filter_map(|r| agents.get(r.index))
        .map(|a| a.agent_id.as_str())
        .collect();
    assert_eq!(
        ids,
        ["r1-0", "r1-0-a-1", "r1-0-b-1"],
        "root first, children id-sorted"
    );
    assert_eq!(rows[0].depth, 0);
    assert_eq!(rows[1].depth, 1);
    assert!(
        members(&agents, "r1-0-a-1").is_empty(),
        "a child is not a root"
    );
    assert!(members(&agents, "ghost").is_empty());
}

#[test]
fn root_of_resolves_any_member_to_its_conversation_root() {
    let agents = [
        agent("r1-0", AgentState::Quiescent, 1),
        agent("r1-0-a-1", AgentState::Quiescent, 2),
        agent("r2-0", AgentState::Quiescent, 3),
    ];
    assert_eq!(root_of(&agents, "r1-0").as_deref(), Some("r1-0"));
    assert_eq!(root_of(&agents, "r1-0-a-1").as_deref(), Some("r1-0"));
    assert_eq!(root_of(&agents, "r2-0").as_deref(), Some("r2-0"));
    assert_eq!(root_of(&agents, "ghost"), None);
    assert_eq!(root_of(&[], "r1-0"), None);
}

#[test]
fn liveness_flags_a_conversation_whose_any_member_holds_a_driver() {
    // §3.6's gate is per-agent: a live *child* under a settled root still means
    // a driver is running inside the wall.
    let agents = [
        agent("r1-0", AgentState::Quiescent, 10),
        agent("r1-0-c-1", AgentState::InFlight, 20),
        agent("r2-0", AgentState::Live, 30),
        agent("r3-0", AgentState::Stopped, 40),
    ];
    let convs = liveness(&agents);
    assert_eq!(
        convs,
        [
            Conversation {
                name: "r1-0".to_owned(),
                live: true
            },
            Conversation {
                name: "r2-0".to_owned(),
                live: true
            },
            Conversation {
                name: "r3-0".to_owned(),
                live: false
            },
        ]
    );
}

#[test]
fn an_unobservable_probe_counts_as_live() {
    // §10's "?" is not a definite reading, so the gate fails closed — a quiescent
    // agent yog could not observe still blocks the unmaking.
    let mut agents = [agent("r1-0", AgentState::Quiescent, 10)];
    assert!(!liveness(&agents)[0].live);
    agents[0].state_uncertain = true;
    assert!(liveness(&agents)[0].live);
}

#[test]
fn a_workspace_with_no_agents_has_no_conversations() {
    assert!(liveness(&[]).is_empty());
}
