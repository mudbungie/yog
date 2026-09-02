//! The descent tree as a **query over the id set** (§2.3, §7.1): the pre-order
//! walk's depths and sibling order, the two shapes that deliberately render at
//! depth 0 (an id outside litany's grammar, a descendant whose intermediate
//! ancestor ref is absent), and the direct-children rule the §11 rail shares.

use super::*;
use crate::git_tree::AgentState;

fn agent(id: &str) -> Agent {
    Agent {
        branch_name: format!("agents/{id}"),
        agent_id: id.to_string(),
        tip_oid: "0".repeat(40),
        tip_short_oid: "00000000".into(),
        tip_timestamp_unix: 0,
        last_action_unix: 0,
        messages: 0,
        steps: vec![],
        preview: None,
        stream: crate::git_tree::Stream::default(),
        tool_calls: vec![],
        state: AgentState::Stopped,
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
        name: None,
        goal_name: None,
        call_start_unix: None,
    }
}

/// Collect `(depth, id)` in render order.
fn order(ids: &[&str]) -> Vec<(usize, String)> {
    let agents: Vec<Agent> = ids.iter().map(|id| agent(id)).collect();
    descent_order(&agents)
        .into_iter()
        .map(|r| (r.depth, agents[r.index].agent_id.clone()))
        .collect()
}

#[test]
fn empty_set_yields_no_rows() {
    assert!(order(&[]).is_empty());
}

#[test]
fn two_roots_with_internal_hyphens_are_both_depth_zero() {
    // Root ids carry a hyphen (timestamp-suffix), yet neither is a
    // prefix of the other, so both render at depth 0.
    let out = order(&["20260427T160000Z-aaaa", "20260427T160001Z-bbbb"]);
    assert_eq!(
        out,
        vec![
            (0, "20260427T160000Z-aaaa".into()),
            (0, "20260427T160001Z-bbbb".into()),
        ]
    );
}

#[test]
fn child_nests_under_parent() {
    // One descent segment below the root: four tokens, parent present.
    let out = order(&["root-x", "root-x-c1-y"]);
    assert_eq!(out, vec![(0, "root-x".into()), (1, "root-x-c1-y".into())]);
}

#[test]
fn multi_level_descent_increments_depth() {
    let out = order(&["a-b", "a-b-c-d", "a-b-c-d-e-f"]);
    assert_eq!(
        out,
        vec![
            (0, "a-b".into()),
            (1, "a-b-c-d".into()),
            (2, "a-b-c-d-e-f".into()),
        ]
    );
}

#[test]
fn siblings_render_id_sorted_under_their_parent() {
    let out = order(&["p-0", "p-0-z-1", "p-0-a-1"]);
    assert_eq!(
        out,
        vec![
            (0, "p-0".into()),
            (1, "p-0-a-1".into()),
            (1, "p-0-z-1".into()),
        ]
    );
}

#[test]
fn an_id_outside_the_two_token_grammar_is_a_root() {
    // `<root>-c0ffee` is one token short of a descent segment (§2.3), so
    // its derived parent is the bare `a`, a ref nobody holds. litany calls
    // that nobody's child; so does yog. (bl-c03e — world C laid exactly
    // this shape, and litany's sweep died on it.)
    let out = order(&["a-b", "a-b-c0ffee"]);
    assert_eq!(out, vec![(0, "a-b".into()), (0, "a-b-c0ffee".into())]);
}

#[test]
fn a_descendant_whose_parent_ref_is_absent_is_a_root() {
    // The grandchild's derived parent `a-b-c-d` is not in the set, so it
    // is nobody's child — it does NOT re-attach to the present `a-b`.
    let out = order(&["a-b", "a-b-c-d-e-f"]);
    assert_eq!(out, vec![(0, "a-b".into()), (0, "a-b-c-d-e-f".into())]);
}

#[test]
fn prefix_without_hyphen_boundary_is_not_an_ancestor() {
    // `a-bb-c-d` derives the parent `a-bb`, not the present `a-b`: the
    // derivation is over whole tokens, so a shared byte prefix is nothing.
    let out = order(&["a-b", "a-bb-c-d"]);
    assert_eq!(out, vec![(0, "a-b".into()), (0, "a-bb-c-d".into())]);
}

#[test]
fn parent_id_strips_exactly_one_two_token_segment() {
    assert_eq!(parent_id("20260427T160000Z-pre0"), None);
    assert_eq!(
        parent_id("20260427T160000Z-pre0-20260427T160100Z-c0ffeeba").as_deref(),
        Some("20260427T160000Z-pre0")
    );
    assert_eq!(parent_id("r-aa-c-bb-g-cc").as_deref(), Some("r-aa-c-bb"));
    // Degenerate short ids still obey the two-token rule.
    assert_eq!(parent_id("a-b"), None);
    assert_eq!(parent_id("solo"), None);
    assert_eq!(parent_id(""), None);
}

/// The rail's cards hang off this one membership rule (VISION V1): direct
/// children only, id-sorted, and a grandchild is nobody's direct child.
#[test]
fn children_of_names_the_direct_descent_id_children_id_sorted() {
    let agents: Vec<Agent> = ["r-aa", "r-aa-c-zz", "r-aa-c-bb", "r-aa-c-bb-g-cc", "s-dd"]
        .iter()
        .map(|id| agent(id))
        .collect();
    let names = |id: &str| -> Vec<String> {
        children_of(&agents, id)
            .into_iter()
            .filter_map(|i| agents.get(i).map(|a| a.agent_id.clone()))
            .collect()
    };
    assert_eq!(names("r-aa"), ["r-aa-c-bb", "r-aa-c-zz"]);
    assert_eq!(names("r-aa-c-bb"), ["r-aa-c-bb-g-cc"]);
    assert!(names("s-dd").is_empty());
}
