//! Unit tests for the `refs/litany/*` mark projection (marks).

use super::*;
use crate::git_tree::AgentState;

/// An unmarked agent — every `refs/litany/*` oid absent.
fn agent() -> Agent {
    Agent {
        branch_name: "agents/a-b".into(),
        agent_id: "a-b".into(),
        tip_oid: "0".repeat(40),
        tip_short_oid: "00000000".into(),
        tip_timestamp_unix: 0,
        last_action_unix: 0,
        messages: 0,
        steps: vec![],
        preview: None,
        stream: crate::git_tree::Stream::default(),
        tool_calls: vec![],
        state: AgentState::Quiescent,
        state_uncertain: false,
        truncated: false,
        pending: vec![],
        conflicted_oid: None,
        budget_oid: None,
        abandoned_oid: None,
        held: None,
        notify_oid: None,
        goal_ball: None,
        name: None,
        goal_name: None,
        call_start_unix: None,
    }
}

#[test]
fn an_unmarked_agent_wears_nothing() {
    assert!(agent().marks().is_empty());
}

#[test]
fn each_oid_projects_to_its_own_mark() {
    for (set, want) in [
        (
            (|a: &mut Agent| a.notify_oid = Some("n".into())) as fn(&mut Agent),
            AgentMark::Notified,
        ),
        (
            |a| a.budget_oid = Some("b".into()),
            AgentMark::BudgetExhausted,
        ),
        (
            |a| a.conflicted_oid = Some("c".into()),
            AgentMark::Conflicted,
        ),
        (|a| a.abandoned_oid = Some("x".into()), AgentMark::Abandoned),
    ] {
        let mut ag = agent();
        set(&mut ag);
        assert_eq!(ag.marks(), vec![want]);
    }
}

#[test]
fn every_mark_at_once_comes_back_in_section_6_order() {
    let mut ag = agent();
    ag.abandoned_oid = Some("x".into());
    ag.conflicted_oid = Some("c".into());
    ag.budget_oid = Some("b".into());
    ag.notify_oid = Some("n".into());
    assert_eq!(
        ag.marks(),
        vec![
            AgentMark::Notified,
            AgentMark::BudgetExhausted,
            AgentMark::Conflicted,
            AgentMark::Abandoned,
        ]
    );
}

#[test]
fn a_mark_survives_its_acknowledgement() {
    // §6's promise, as a test: acknowledging moves a `ui.json` watermark and
    // never touches the ref, so the mark the seats render is unmoved by an
    // ack. `marks()` reads only the oids — there is no `seen` input to move.
    let mut ag = agent();
    ag.notify_oid = Some("n".into());
    let before = ag.marks();
    let seen = |k, _w: &str, _a: &str, o: &str| k == crate::ui_state::SeenKind::Notify && o == "n";
    assert!(!crate::attention::attention(&ag, "ws", &seen).notify);
    assert_eq!(ag.marks(), before);
    assert_eq!(before, vec![AgentMark::Notified]);
}

#[test]
fn parse_oids_strips_prefix_and_keeps_oid() {
    let out = b"refs/litany/conflicted/a-b 1111111111111111111111111111111111111111\n\
                refs/litany/conflicted/c-d 2222222222222222222222222222222222222222\n";
    let oids = parse_oids(out, CONFLICTED_PREFIX);
    assert_eq!(oids.len(), 2);
    assert_eq!(
        oids.get("a-b").map(String::as_str),
        Some("1".repeat(40).as_str())
    );
    assert_eq!(
        oids.get("c-d").map(String::as_str),
        Some("2".repeat(40).as_str())
    );
}

#[test]
fn parse_oids_ignores_nonmatching_and_malformed_lines() {
    // A ref outside the namespace, and a line with no oid field, both
    // contribute nothing.
    let out = b"refs/heads/agents/a-b 3333333333333333333333333333333333333333\n\
                refs/litany/budget-exhausted/x-y 4444444444444444444444444444444444444444\n\
                nospaceline\n";
    let oids = parse_oids(out, BUDGET_PREFIX);
    assert_eq!(oids.len(), 1);
    assert_eq!(
        oids.get("x-y").map(String::as_str),
        Some("4".repeat(40).as_str())
    );
}

#[test]
fn parse_oids_drops_empty_id() {
    // The bare prefix with no id after it strips to "" and is dropped.
    let out = b"refs/litany/notify/ 5555555555555555555555555555555555555555\n";
    assert!(parse_oids(out, NOTIFY_PREFIX).is_empty());
}

#[test]
fn parse_oids_empty_input_is_empty_map() {
    assert!(parse_oids(b"", CONFLICTED_PREFIX).is_empty());
}

#[test]
fn getters_return_oid_or_none() {
    let marks = Marks {
        conflicted: HashMap::from([("a-b".to_string(), "aa".to_string())]),
        budget: HashMap::from([("c-d".to_string(), "bb".to_string())]),
        abandoned: HashMap::from([("e-f".to_string(), "cc".to_string())]),
        notify: HashMap::from([("g-h".to_string(), "dd".to_string())]),
        held: HashMap::new(),
    };
    assert_eq!(marks.conflicted_oid("a-b").as_deref(), Some("aa"));
    assert_eq!(marks.conflicted_oid("c-d"), None);
    assert_eq!(marks.budget_oid("c-d").as_deref(), Some("bb"));
    assert_eq!(marks.abandoned_oid("e-f").as_deref(), Some("cc"));
    assert_eq!(marks.notify_oid("g-h").as_deref(), Some("dd"));
    assert_eq!(marks.notify_oid("a-b"), None);
}
