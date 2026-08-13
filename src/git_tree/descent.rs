//! Hyphenated-descent tree ordering for the agent view (§2.3, §7.1).
//!
//! Agent ids encode the full descent from the root — hierarchy lives in the
//! name, not the filesystem. **lernie's grammar is the authority and it is
//! narrow** (ARCH §2.3): an id is a chain of `-`-joined **segments**, and a
//! segment is `<ts>-<short>`, exactly two hyphen-free tokens. So a root is
//! two tokens (`20260427T160000Z-pre0`), a child four, a grandchild six, and
//! an id's parent is that id less its last two tokens ([`parent_id`], the
//! mirror of lernie's own `prompt::inbox::parent_of`).
//!
//! This derives the render tree purely from the id set: an agent's parent is
//! its **derived** parent id when that id is present in the set, and every
//! other agent is a root row. That second clause is a *query against the
//! registry*, not string arithmetic — the same intersection lernie's own
//! sweep applies (ARCH §8: "a branch whose derived address holds no ref …
//! is nobody's child … treated exactly as a root"). A pre-order walk yields
//! each agent paired with its nesting depth, children directly under their
//! parent. Nothing is stored — the tree is a query over the ids (PRINCIPLES
//! "Single source of truth").
//!
//! Two shapes this deliberately renders at depth 0, where a looser
//! longest-hyphen-prefix rule nested them (bl-c03e): an id **outside** the
//! grammar (`<root>-c0ffee`, one token too few — lernie would never mint it),
//! and a descendant whose intermediate ancestor ref is **absent**. Both are
//! ids lernie itself refuses to call anyone's child, and yog agreeing is what
//! keeps one grammar rather than two.

use std::collections::HashMap;

use super::Agent;

/// One rendered row: an agent's index in the input slice and its nesting depth
/// in the descent tree. Owned and `Copy` — the caller resolves the index back
/// into the slice it already holds, so no row borrows the agent set.
#[derive(Clone, Copy)]
pub struct DescentRow {
    pub depth: usize,
    pub index: usize,
}

/// Order `agents` as a descent tree: roots first (id-sorted), each agent's
/// children immediately beneath it (also id-sorted), depth = nesting level.
pub fn descent_order(agents: &[Agent]) -> Vec<DescentRow> {
    // Sibling order is by id, deterministically, at every level. Pair each
    // agent with its original index so no step ever indexes back into `agents`.
    let mut sorted: Vec<(usize, &Agent)> = agents.iter().enumerate().collect();
    sorted.sort_by(|(_, a), (_, b)| a.agent_id.cmp(&b.agent_id));

    // Children keyed by parent index; a leaf simply has no entry.
    let mut children: HashMap<usize, Vec<usize>> = HashMap::new();
    let mut roots: Vec<usize> = Vec::new();
    for &(i, agent) in &sorted {
        match parent_index(agents, agent.agent_id.as_str()) {
            Some(parent) => children.entry(parent).or_default().push(i),
            None => roots.push(i),
        }
    }

    let mut rows = Vec::with_capacity(agents.len());
    for &root in &roots {
        walk(root, 0, &children, &mut rows);
    }
    rows
}

fn walk(i: usize, depth: usize, children: &HashMap<usize, Vec<usize>>, rows: &mut Vec<DescentRow>) {
    rows.push(DescentRow { depth, index: i });
    if let Some(kids) = children.get(&i) {
        for &child in kids {
            walk(child, depth + 1, children, rows);
        }
    }
}

/// The id one descent segment up: `id` less its last two hyphen-free tokens
/// (§2.3). `None` for an id of two tokens or fewer — a root, which has no
/// parent — and the derivation is total over every other string, so an id
/// outside the grammar simply derives an address nobody holds.
fn parent_id(id: &str) -> Option<String> {
    id.rsplitn(3, '-').nth(2).map(str::to_owned)
}

/// Indices of `id`'s **direct** descent-id children, id-sorted like every
/// other sibling order here. The membership rule is [`parent_index`]'s, reused
/// rather than restated — the step spine's cards (VISION V1) hang off the
/// same provenance tree §11's descent rows are drawn from, so the two can
/// never disagree about who dispatched whom.
pub fn children_of(agents: &[Agent], id: &str) -> Vec<usize> {
    let mut found: Vec<(&str, usize)> = agents
        .iter()
        .enumerate()
        .filter(|(_, agent)| parent_id(&agent.agent_id).as_deref() == Some(id))
        .map(|(index, agent)| (agent.agent_id.as_str(), index))
        .collect();
    found.sort_unstable();
    found.into_iter().map(|(_, index)| index).collect()
}

/// Index of the agent this one descends from: its [`parent_id`], looked up in
/// the present set. `None` — a root row — for a root id, for an id outside the
/// grammar, and for a descendant whose parent ref is absent.
fn parent_index(agents: &[Agent], id: &str) -> Option<usize> {
    let parent = parent_id(id)?;
    agents.iter().position(|a| a.agent_id == parent)
}

#[cfg(test)]
mod tests {
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
        // its derived parent is the bare `a`, a ref nobody holds. lernie calls
        // that nobody's child; so does yog. (bl-c03e — world C laid exactly
        // this shape, and lernie's sweep died on it.)
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
}
