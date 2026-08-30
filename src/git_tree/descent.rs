//! Hyphenated-descent tree ordering for the agent view (§2.3, §7.1).
//!
//! Agent ids encode the full descent from the root — hierarchy lives in the
//! name, not the filesystem. **litany's grammar is the authority and it is
//! narrow** (ARCH §2.3): an id is a chain of `-`-joined **segments**, and a
//! segment is `<ts>-<short>`, exactly two hyphen-free tokens. So a root is
//! two tokens (`20260427T160000Z-pre0`), a child four, a grandchild six, and
//! an id's parent is that id less its last two tokens ([`parent_id`], the
//! mirror of litany's own `prompt::inbox::parent_of`).
//!
//! This derives the render tree purely from the id set: an agent's parent is
//! its **derived** parent id when that id is present in the set, and every
//! other agent is a root row. That second clause is a *query against the
//! registry*, not string arithmetic — the same intersection litany's own
//! sweep applies (ARCH §8: "a branch whose derived address holds no ref …
//! is nobody's child … treated exactly as a root"). A pre-order walk yields
//! each agent paired with its nesting depth, children directly under their
//! parent. Nothing is stored — the tree is a query over the ids (PRINCIPLES
//! "Single source of truth").
//!
//! Two shapes this deliberately renders at depth 0, where a looser
//! longest-hyphen-prefix rule nested them (bl-c03e): an id **outside** the
//! grammar (`<root>-c0ffee`, one token too few — litany would never mint it),
//! and a descendant whose intermediate ancestor ref is **absent**. Both are
//! ids litany itself refuses to call anyone's child, and yog agreeing is what
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
mod tests;
