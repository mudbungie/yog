//! The §3.3 ladder as a seat holds it (REMOTE §9.4, bl-1eb0): the same answer
//! the agent-set lookup gives, addressed by id and buildable from a row
//! listing.

use super::*;
use crate::nav::convs::Titles;

/// The table is the ladder: a named agent reads its name, a nameless one its
/// payload line, and both agree with the slice-side lookup every engine seat
/// uses — one ladder, two lookups.
#[test]
fn a_title_is_the_same_answer_the_agent_lookup_gives() {
    let mut named = agent("aaaa-0001", AgentState::Stopped, 1);
    named.name = Some("pennant".to_owned());
    let mut bare = agent("bbbb-0002", AgentState::Stopped, 1);
    bare.preview = Some("fix the gate".to_owned());
    let agents = vec![named, bare];
    let titles = Titles::of(&agents);

    assert_eq!(titles.name("aaaa-0001"), "pennant");
    assert_eq!(titles.name("bbbb-0002"), "fix the gate");
    for a in &agents {
        assert_eq!(
            titles.name(&a.agent_id),
            display_name_of(&agents, &a.agent_id)
        );
    }
}

/// An id nobody here carries lands on the ladder's floor — the terminal
/// generation, never the whole ancestry chain — which is what a deposit from
/// `user`, or from a peer this workspace has deleted, paints.
#[test]
fn an_unknown_id_lands_on_the_floor() {
    let titles = Titles::of(&[agent("aaaa-0001", AgentState::Stopped, 1)]);
    assert_eq!(titles.name("user"), "user");
    assert_eq!(
        titles.name("20260801T225418Z-aaaa-20260801T225500Z-bbbb"),
        "20260801T225500Z-bbbb"
    );
    assert_eq!(Titles::default().name("solo"), "solo");
}

/// The seat's own construction: a conversations reply carries `root_id` and
/// `display`, and that pair **is** this table — so a face holding replies and
/// no agent set resolves exactly the same names.
#[test]
fn a_row_listing_builds_the_same_table() {
    let mut named = agent("aaaa-0001", AgentState::Stopped, 1);
    named.name = Some("pennant".to_owned());
    let agents = vec![named, agent("bbbb-0002", AgentState::Stopped, 1)];
    let rows = build(&agents, "/w", &unseen, 0, &plain, &[]);

    let from_rows = Titles::of_rows(&rows);
    let from_agents = Titles::of(&agents);
    for a in &agents {
        assert_eq!(from_rows.name(&a.agent_id), from_agents.name(&a.agent_id));
    }
}
