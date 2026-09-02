//! The agent-delete tables: the gate over the conversation's members, the
//! amended §3.6 arming rule (typed name iff subtree), the `DeleteReport`
//! parse, and the two spawns — the unlogged dry-run census and the logged
//! removal.

use super::*;

mod exec;

pub(super) const ROOT: &str = "r-aa";
pub(super) const CHILD: &str = "r-aa-c-bb";

/// One agent row, the conversation-list fixture shape (litany ARCH §2.3).
fn agent(id: &str, state: AgentState, ts: i64) -> Agent {
    Agent {
        branch_name: format!("agents/{id}"),
        agent_id: id.to_string(),
        name: None,
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
        truncated: false,
        failure: None,
        pending: vec![],
        conflicted_oid: None,
        budget_oid: None,
        abandoned_oid: None,
        notify_oid: None,
        held: None,
        goal_ball: None,
        goal_name: None,
        call_start_unix: None,
    }
}

#[test]
fn the_gate_counts_every_running_or_uncertain_member_and_only_members() {
    let kid = agent(CHILD, AgentState::InFlight, 2);
    let ghost = agent("z-zz", AgentState::Live, 3);
    let mut root = agent(ROOT, AgentState::Stopped, 1);
    root.state_uncertain = true; // the §10 "?" counts as live — fail closed
    let confirm = confirmation(ROOT, &[root, kid, ghost]);
    assert_eq!(
        confirm.live,
        [ROOT, CHILD],
        "the other conversation is not ours"
    );
    assert!(confirm.refused());
    assert!(!confirm.subtree_armed(ROOT), "never armed while refused");
}

#[test]
fn a_settled_conversation_passes_and_the_typed_name_arms_the_subtree() {
    let mut root = agent(ROOT, AgentState::Stopped, 1);
    root.goal_name = Some("fix the parser".into());
    let confirm = confirmation(ROOT, &[root, agent(CHILD, AgentState::Quiescent, 2)]);
    assert!(!confirm.refused());
    assert_eq!(
        confirm.name, "fix the parser",
        "the §3.3 ladder names the dialog"
    );
    assert!(
        confirm.subtree_armed("  fix the parser "),
        "whitespace forgiven"
    );
    assert!(!confirm.subtree_armed("fix"), "nothing else is");
}

#[test]
fn an_absent_root_is_the_general_path_with_empty_inputs() {
    // litany's delete of an absent agent is already its postcondition; yog's
    // gate mirrors that convergence rather than minting an error class.
    let confirm = confirmation("gone-id", &[]);
    assert_eq!(confirm.name, "gone-id", "rung three: its own id");
    assert!(!confirm.refused());
}

/// **The seat's own projection of the same gate** (REMOTE §9.7, bl-b4b5): the
/// §11 danger row and the dialog fold `Query::Conversations`' answered forest
/// rather than the engine's agent set, and the two must say the same thing —
/// the chokepoint's re-derivation at fire is what actually decides, so a
/// painted affordance that disagreed would offer a verb the engine refuses.
#[test]
fn the_seat_reads_the_same_gate_off_an_answered_forest() {
    let kid = agent(CHILD, AgentState::InFlight, 2);
    let ghost = agent("z-zz", AgentState::Live, 3);
    let mut root = agent(ROOT, AgentState::Stopped, 1);
    root.state_uncertain = true;
    root.goal_name = Some("fix the parser".into());
    let agents = [root, kid, ghost];
    let rows = crate::nav::convs::forest_rows(
        &agents,
        "/ws",
        &|_, _: &str, _: &str, _: &str| false,
        100,
        &|id: &str| crate::nav::convs::ConvBall {
            id: id.to_owned(),
            state: None,
            title: None,
            badge: None,
        },
        &[],
    );
    let seat = confirmation_of_rows(&rows, ROOT);
    assert_eq!(
        seat,
        confirmation(ROOT, &agents),
        "one gate, two projections"
    );
    assert!(seat.refused(), "the §10 uncertainty still counts as live");
    // A root the answer does not carry is the same empty-input arm the engine's
    // own miss takes: its id for a name, nothing live.
    let absent = confirmation_of_rows(&rows, "gone-id");
    assert_eq!(absent, confirmation("gone-id", &[]));
}

#[test]
fn the_refusal_names_the_live_members() {
    assert_eq!(
        live_refusal(&["r-aa".into(), "r-aa-c-bb".into()]),
        "refused \u{2014} live: r-aa, r-aa-c-bb \u{2014} stop them first"
    );
}

#[test]
fn the_report_parse_reads_both_moods_and_refuses_garbage() {
    let subtree = parse_report(
        "would delete r-aa; descendants: 2 (r-aa-c-bb, r-aa-d-ee); pending deposits: 3",
    )
    .unwrap();
    assert_eq!(subtree.descendants, [CHILD, "r-aa-d-ee"]);
    assert_eq!(subtree.pending_deposits, 3);
    let leaf = parse_report("deleted r-aa; descendants: 0; pending deposits: 0").unwrap();
    assert_eq!(
        leaf,
        Census {
            descendants: vec![],
            pending_deposits: 0
        }
    );
    for garbage in [
        "",
        "agent \"r-aa\" is being driven",
        "would delete r-aa; descendants: 1 (x; pending deposits: y",
    ] {
        assert!(parse_report(garbage).is_none(), "{garbage:?}");
    }
}
