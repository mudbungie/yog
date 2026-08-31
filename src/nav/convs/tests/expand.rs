//! Tables for the §11 unfold ([`super::super::expand`], bl-fa82): the visible
//! rows over the descent forest, the two walks the keyboard rides, and the
//! ancestor chain a jump reveals. The all-collapsed pin — that an empty set is
//! exactly the list this seat painted before — lives beside the row projection
//! it must not disturb, in [`super::rows`].

use super::*;
use crate::nav::convs::expand::{ancestors, forest_rows, parent_of, step, visible};
use std::collections::HashSet;

/// The expanded set, spelled as the shell hands it over.
fn open(ids: &[&str]) -> HashSet<String> {
    ids.iter().map(|s| (*s).to_string()).collect()
}

/// A three-generation fixture: `r-0` with two children, the second of which has
/// a child of its own; `s-0` alone and more recent, so it leads the list.
fn family() -> [Agent; 5] {
    [
        agent("r-0", AgentState::Quiescent, 10),
        agent("r-0-a-1", AgentState::Quiescent, 11),
        agent("r-0-b-1", AgentState::Quiescent, 12),
        agent("r-0-b-1-x-2", AgentState::Quiescent, 13),
        agent("s-0", AgentState::Quiescent, 90),
    ]
}

fn ids(rows: &[crate::nav::convs::ConvRow]) -> Vec<&str> {
    rows.iter().map(|r| r.root_id.as_str()).collect()
}

fn rows_with(agents: &[Agent], expanded: &HashSet<String>) -> Vec<crate::nav::convs::ConvRow> {
    visible(&forest(agents), expanded)
}

/// The boundary's own answer — the whole forest, no fold (REMOTE §9.7).
fn forest(agents: &[Agent]) -> Vec<crate::nav::convs::ConvRow> {
    forest_rows(agents, "/ws", &unseen, 100, &plain, &[])
}

/// REMOTE §9.7's altitude ruling (bl-44e9): the answer is the whole forest, and
/// every fold is a *selection* out of it — so the fold of the answer and the
/// answer of the fold are the same rows, for every set a seat could hold.
#[test]
fn the_answer_is_the_whole_forest_and_a_fold_selects_out_of_it() {
    let agents = family();
    let all = forest(&agents);
    assert_eq!(
        ids(&all),
        ["s-0", "r-0", "r-0-a-1", "r-0-b-1", "r-0-b-1-x-2"],
        "every member, in paint order, with no set consulted"
    );
    assert_eq!(
        ids(&visible(&all, &HashSet::new())),
        ["s-0", "r-0"],
        "no fold selects the root subset"
    );
    // The rollups are the answer's, not the fold's: a row carries the same
    // numbers whether or not the seat has it open.
    let open_all = open(&["r-0", "r-0-b-1"]);
    assert_eq!(
        visible(&all, &open_all),
        all,
        "everything open is everything"
    );
    for set in [HashSet::new(), open(&["r-0"]), open_all] {
        for row in visible(&all, &set) {
            let answered = all.iter().find(|r| r.root_id == row.root_id);
            assert_eq!(answered, Some(&row), "a fold changes no row it keeps");
        }
    }
}

#[test]
fn nothing_expanded_is_one_row_per_root() {
    let agents = family();
    let rows = rows_with(&agents, &HashSet::new());
    assert_eq!(ids(&rows), ["s-0", "r-0"], "recency order, roots only");
    assert_eq!(rows[1].members, 4, "a collapsed row speaks for its subtree");
    assert_eq!(rows[1].direct, 2);
    assert_eq!(rows[1].total(), 3, "total is every agent under it");
    assert!(rows.iter().all(|r| r.depth == 0));
}

#[test]
fn expanding_a_root_reveals_exactly_its_direct_children_in_descent_order() {
    let agents = family();
    let rows = rows_with(&agents, &open(&["r-0"]));
    assert_eq!(
        ids(&rows),
        ["s-0", "r-0", "r-0-a-1", "r-0-b-1"],
        "the grandchild stays hidden under its collapsed parent"
    );
    assert_eq!(rows[2].depth, 1);
    assert_eq!(rows[3].depth, 1);
    // The collapsed child's own badge covers its hidden descent.
    assert_eq!(rows[3].members, 2);
    assert_eq!(rows[3].direct, 1);
    assert_eq!(rows[2].members, 1, "a leaf is its own subtree");
    assert_eq!(rows[2].direct, 0);
    assert!(!rows[2].has_children());
}

#[test]
fn expansion_recurses_and_direct_and_total_part_ways() {
    let agents = family();
    let rows = rows_with(&agents, &open(&["r-0", "r-0-b-1"]));
    assert_eq!(
        ids(&rows),
        ["s-0", "r-0", "r-0-a-1", "r-0-b-1", "r-0-b-1-x-2"]
    );
    assert_eq!(rows[4].depth, 2, "a generation deeper indents again");
    // The root dispatched two agents itself and has three beneath it — the two
    // numbers the subagent field paints, distinct exactly where a grandchild is.
    assert_eq!((rows[1].direct, rows[1].total()), (2, 3));
}

#[test]
fn expanding_a_row_never_moves_its_conversation_in_the_list() {
    // The sort key is the *root's* subtree fold (bl-cad5); unfolding changes
    // which rows are painted, never the order the conversations arrive in.
    let agents = family();
    let rows = rows_with(&agents, &open(&["r-0", "r-0-b-1"]));
    assert_eq!(
        rows[0].root_id, "s-0",
        "the recent conversation still leads"
    );
    // An id nobody carries expands nothing; a leaf named in the set is a no-op.
    let noise = rows_with(&agents, &open(&["ghost", "s-0", "r-0-a-1"]));
    assert_eq!(ids(&noise), ["s-0", "r-0"]);
}

#[test]
fn the_toggle_round_trips_the_list_through_the_shared_disclosure_set() {
    // A seat's disclosure toggle is one flip of membership in the set this
    // module reads (it never owns one). Two flips of the same id restore the
    // rows byte for byte, which is what makes the disclosed list a *state*
    // rather than a first render.
    let agents = family();
    let collapsed = rows_with(&agents, &HashSet::new());
    let mut set = HashSet::new();
    let toggle = |set: &mut HashSet<String>, id: &str| {
        if !set.remove(id) {
            set.insert(id.to_owned());
        }
    };
    toggle(&mut set, "r-0");
    let open_rows = rows_with(&agents, &set);
    assert_eq!(ids(&open_rows), ["s-0", "r-0", "r-0-a-1", "r-0-b-1"]);
    toggle(&mut set, "r-0");
    assert!(set.is_empty(), "the second flip takes the id back out");
    assert_eq!(rows_with(&agents, &set), collapsed);
}

#[test]
fn the_strict_descent_rule_decides_membership_not_a_prefix() {
    // Mirrors the git_tree::descent fixtures: an id outside the two-token
    // grammar and one whose parent ref is absent are both ROOT rows here, never
    // re-attached to some shorter prefix that happens to match.
    let agents = [
        agent("r-0", AgentState::Quiescent, 10),
        agent("r-0-orphanmaker", AgentState::Quiescent, 20),
        agent("r-0-gone-1-kid-2", AgentState::Quiescent, 30),
    ];
    let rows = rows_with(&agents, &open(&["r-0"]));
    assert_eq!(
        ids(&rows),
        ["r-0-gone-1-kid-2", "r-0-orphanmaker", "r-0"],
        "three roots by recency; expanding r-0 reveals nothing"
    );
    assert!(rows.iter().all(|r| r.depth == 0 && r.direct == 0));
}

#[test]
fn an_empty_workspace_has_no_visible_rows() {
    assert!(rows_with(&[], &open(&["r-0"])).is_empty());
}

#[test]
fn the_walk_steps_the_visible_rows_and_wraps() {
    let agents = family();
    let rows = rows_with(&agents, &open(&["r-0"]));
    // ↓ from the collapsed `s-0` lands on the next row at the same level; ↓
    // from `r-0` (now expanded) enters its first child — the same call either
    // way, which is why the walk needs no rule about expansion.
    assert_eq!(step(&rows, Some("s-0"), 1).as_deref(), Some("r-0"));
    assert_eq!(step(&rows, Some("r-0"), 1).as_deref(), Some("r-0-a-1"));
    assert_eq!(step(&rows, Some("r-0-a-1"), -1).as_deref(), Some("r-0"));
    assert_eq!(
        step(&rows, Some("r-0-b-1"), 1).as_deref(),
        Some("s-0"),
        "wraps"
    );
    assert_eq!(step(&rows, Some("s-0"), -1).as_deref(), Some("r-0-b-1"));
    // An unknown or absent selection starts before the front.
    assert_eq!(step(&rows, None, 1).as_deref(), Some("s-0"));
    assert_eq!(step(&rows, Some("ghost"), -1).as_deref(), Some("r-0-b-1"));
    assert_eq!(step(&[], Some("s-0"), 1), None);
}

#[test]
fn the_walk_skips_a_collapsed_subtree_whole() {
    let agents = family();
    let collapsed = rows_with(&agents, &HashSet::new());
    assert_eq!(
        step(&collapsed, Some("s-0"), 1).as_deref(),
        Some("r-0"),
        "with nothing expanded the walk is the root list"
    );
    // The grandchild is not steppable while its parent is folded away, even
    // though its own parent's parent is open.
    let partway = rows_with(&agents, &open(&["r-0"]));
    assert_eq!(
        step(&partway, Some("r-0-b-1-x-2"), 1).as_deref(),
        Some("s-0")
    );
}

#[test]
fn parent_of_pages_up_one_level_off_the_painted_depths() {
    let agents = family();
    let rows = rows_with(&agents, &open(&["r-0", "r-0-b-1"]));
    assert_eq!(parent_of(&rows, "r-0-b-1-x-2").as_deref(), Some("r-0-b-1"));
    assert_eq!(parent_of(&rows, "r-0-b-1").as_deref(), Some("r-0"));
    assert_eq!(parent_of(&rows, "r-0-a-1").as_deref(), Some("r-0"));
    assert_eq!(parent_of(&rows, "r-0"), None, "a root has nothing above it");
    assert_eq!(parent_of(&rows, "s-0"), None);
    assert_eq!(parent_of(&rows, "ghost"), None);
}

#[test]
fn ancestors_are_the_chain_a_jump_reveals() {
    let agents = family();
    assert_eq!(
        ancestors(&agents, "r-0-b-1-x-2"),
        ["r-0", "r-0-b-1"],
        "outermost first — expanding all of them makes the target visible"
    );
    assert_eq!(ancestors(&agents, "r-0-a-1"), ["r-0"]);
    assert!(
        ancestors(&agents, "r-0").is_empty(),
        "a root is already visible"
    );
    assert!(ancestors(&agents, "ghost").is_empty());
    // The reveal is exactly enough: expanding the chain paints the target.
    let revealed = rows_with(&agents, &open(&["r-0", "r-0-b-1"]));
    assert!(revealed.iter().any(|r| r.root_id == "r-0-b-1-x-2"));
}
