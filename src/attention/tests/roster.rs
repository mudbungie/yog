//! The rollups (workspace count, strip total), the roster sort, and
//! jump-to-next-attention wrapping.

use super::{agent, nothing, tips_acked};
use crate::attention::*;
use crate::git_tree::{Agent, AgentState};

/// Wrap agents as roster keys all under one workspace path, each carrying its
/// (unacknowledged) attention verdict — the raw order, unsorted, so the step
/// tests exercise the walk over a known sequence.
fn roster(ags: &[Agent]) -> Vec<RosterKey> {
    ags.iter()
        .map(|a| RosterKey {
            ws: "/w".to_string(),
            agent_id: a.agent_id.clone(),
            attention: attention(a, "/w", &nothing).any(),
        })
        .collect()
}

/// A focus on agent `id` within `/w`.
fn focus(id: &str) -> (&str, &str) {
    ("/w", id)
}

/// Three entries: a1 (attention), a2 (calm), a3 (attention).
fn jump_fixture() -> [Agent; 3] {
    let mut a1 = agent("a1");
    a1.state = AgentState::Stopped;
    let mut a3 = agent("a3");
    a3.notify_oid = Some("n".into());
    [a1, agent("a2"), a3]
}

#[test]
fn workspace_count_and_strip_total() {
    let mut needs = agent("a");
    needs.notify_oid = Some("n".into());
    let mut needs2 = agent("c");
    needs2.state = AgentState::Stopped;
    let ws_a = [needs, agent("b")];
    let ws_b = [needs2];
    assert_eq!(workspace_count(&ws_a, "/w/a", &nothing), 1);
    assert_eq!(workspace_count(&ws_b, "/w/b", &nothing), 1);
    assert_eq!(
        strip_total(
            &[("/w/a", ws_a.as_slice()), ("/w/b", ws_b.as_slice())],
            &nothing
        ),
        2
    );
}

#[test]
fn roster_sorts_attention_then_running_then_idle_then_descent() {
    let running = agent("r-live"); // Live by default
    let mut attn = agent("r-attn");
    attn.state = AgentState::Stopped; // unseen rest -> attention
    // The two idle rows are at rest at tips already seen — since bl-2194 that is
    // what "idle" means (rest alone is rule 2 firing).
    let (mut idle, mut idle2) = (agent("r"), agent("r-idle"));
    idle.state = AgentState::Quiescent;
    idle2.state = AgentState::Quiescent;
    let agents = [idle, running, attn, idle2];
    let ids: Vec<&str> = sorted_roster(&agents, "/w", &tips_acked(&["r", "r-idle"]))
        .iter()
        .map(|&i| agents[i].agent_id.as_str())
        .collect();
    // attention; then running; then the two idle in descent order (root
    // before its child).
    assert_eq!(ids, vec!["r-attn", "r-live", "r", "r-idle"]);
}

#[test]
fn roster_order_is_path_order_across_and_sorted_within() {
    let mut attn = agent("z");
    attn.state = AgentState::Stopped;
    let ws_b = [attn, agent("y")]; // attention agent floats up
    let solo = [agent("s")];
    // Passed out of path order; roster_order re-sorts to /w/a then /w/b.
    let order = roster_order(
        &[("/w/b", ws_b.as_slice()), ("/w/a", solo.as_slice())],
        &nothing,
    );
    let flat: Vec<(&str, &str)> = order
        .iter()
        .map(|e| (e.ws.as_str(), e.agent_id.as_str()))
        .collect();
    assert_eq!(flat, vec![("/w/a", "s"), ("/w/b", "z"), ("/w/b", "y")]);
}

#[test]
fn jump_empty_roster_is_none() {
    assert!(next_attention(&[], None).is_none());
}

#[test]
fn jump_from_none_finds_first_attention() {
    let ags = jump_fixture();
    let hit = next_attention(&roster(&ags), None).unwrap();
    assert_eq!(hit.agent_id, "a1");
}

#[test]
fn jump_none_when_nothing_has_attention() {
    let ags = [agent("a1"), agent("a2")];
    assert!(next_attention(&roster(&ags), None).is_none());
}

#[test]
fn jump_wraps_past_the_end() {
    let ags = jump_fixture();
    // Focused on a3 (last, attention); next wraps to a1.
    let hit = next_attention(&roster(&ags), Some(focus("a3"))).unwrap();
    assert_eq!(hit.agent_id, "a1");
}

#[test]
fn jump_sole_attention_returns_itself_on_full_wrap() {
    let mut only = agent("a1");
    only.notify_oid = Some("n".into());
    let ags = [only, agent("a2"), agent("a3")];
    let hit = next_attention(&roster(&ags), Some(focus("a1"))).unwrap();
    assert_eq!(hit.agent_id, "a1");
}

#[test]
fn jump_unknown_focus_starts_from_front() {
    let ags = jump_fixture();
    let hit = next_attention(&roster(&ags), Some(focus("ghost"))).unwrap();
    assert_eq!(hit.agent_id, "a1");
}

#[test]
fn step_empty_roster_is_none() {
    assert!(step(&[], None, 1).is_none());
}

#[test]
fn step_from_none_lands_on_the_ends() {
    let ags = jump_fixture();
    let r = roster(&ags);
    // +1 from nothing → first entry; -1 from nothing → last entry. Unlike the
    // jump, calm entries are visited: a2 is not skipped.
    assert_eq!(step(&r, None, 1).unwrap().agent_id, "a1");
    assert_eq!(step(&r, None, -1).unwrap().agent_id, "a3");
}

#[test]
fn step_moves_one_entry_each_way_visiting_calm_entries() {
    let ags = jump_fixture();
    let r = roster(&ags);
    // Down from a1 → a2 (calm, not skipped); up from a2 → a1.
    assert_eq!(step(&r, Some(focus("a1")), 1).unwrap().agent_id, "a2");
    assert_eq!(step(&r, Some(focus("a2")), -1).unwrap().agent_id, "a1");
}

#[test]
fn step_wraps_both_directions() {
    let ags = jump_fixture();
    let r = roster(&ags);
    // Down past the last wraps to the first; up past the first wraps to last.
    assert_eq!(step(&r, Some(focus("a3")), 1).unwrap().agent_id, "a1");
    assert_eq!(step(&r, Some(focus("a1")), -1).unwrap().agent_id, "a3");
}

#[test]
fn step_unknown_focus_starts_from_the_end_matching_direction() {
    let ags = jump_fixture();
    let r = roster(&ags);
    assert_eq!(step(&r, Some(focus("ghost")), 1).unwrap().agent_id, "a1");
    assert_eq!(step(&r, Some(focus("ghost")), -1).unwrap().agent_id, "a3");
}
