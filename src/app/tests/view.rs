//! Tests for the model's read surface (`app/view.rs`): the roster rollups,
//! the tab bar, the conversation rows and the focused-agent lookup.
//!
//! Split from `tests/focus.rs` alongside the production split — those are
//! reads; `tests/focus.rs` keeps the focus and seen-acknowledgement gestures.

use super::{Harness, agent};
use crate::git_tree::AgentState;

#[test]
fn strip_total_counts_attention_bearing_agents() {
    let h = Harness::new();
    let (_c, model) = h.model();
    assert_eq!(model.strip_total(), 1, "the one unseen stop");
}

#[test]
fn focused_agent_resolves_the_selected_row_else_none() {
    let h = Harness::new();
    let (_c, mut model) = h.model();
    // No agent selected → None; a focused workspace alone is not enough.
    model.focus_workspace(&h.ws);
    assert!(model.focused_agent().is_none());
    // Selecting the agent resolves its snapshot row (the inspector target).
    model.focus_agent(&h.ws, "c-1");
    assert_eq!(
        model.focused_agent().map(|a| a.agent_id.as_str()),
        Some("c-1")
    );
    // A selected id absent from the tree resolves to None.
    model.focus_agent(&h.ws, "ghost");
    assert!(model.focused_agent().is_none());
    // A focused workspace with no snapshot (unfetched) resolves to None even
    // with an agent selected — the tree-absent arm.
    model.focus_agent(std::path::Path::new("/no/such/ws"), "x");
    assert!(model.focused_agent().is_none());
}

#[test]
fn the_tab_bar_and_conversation_list_carry_the_derived_facts() {
    let h = Harness::new();
    let (_c, mut model) = h.model();
    // The ad-hoc workspace is Foreign → an overflow entry with its attention.
    let bar = model.tab_bar();
    assert!(bar.tabs.is_empty(), "no named workspaces, no wall tabs");
    assert_eq!(bar.overflow.len(), 1);
    let entry = &bar.overflow[0];
    assert_eq!(entry.name, "ws");
    assert_eq!(entry.attention, 1);
    assert_eq!(entry.kind, crate::nav::tabs::Kind::Foreign);
    // The focused workspace's conversation list: one row per root agent (§11),
    // its unseen stop counted as attention.
    model.focus_workspace(&h.ws);
    let rows = model.conversations(10);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].root_id, "c-1");
    assert_eq!(rows[0].attention, 1);
    assert_eq!(rows[0].members, 1);
    assert_eq!(rows[0].depth, 0);
    assert_eq!(rows[0].direct, 0, "a lone root dispatched nobody");
    // The unfold is plumbed through the frame's own accessor (bl-fa82), and
    // naming a childless row in the set reveals nothing — the flat list is the
    // all-collapsed case, so the two agree wherever there is no descent.
    let expanded = std::collections::HashSet::from(["c-1".to_owned()]);
    assert_eq!(model.visible_conversations(10, &expanded), rows);
    // No focus → the empty list (the general empty path, not a special case).
    model.focus_workspace(std::path::Path::new("/no/such"));
    assert!(model.conversations(10).is_empty(), "unfetched ws: no rows");
    assert!(model.visible_conversations(10, &expanded).is_empty());
}

/// §8.5 parity (VISION §4.8): the boundary answer IS the frame's own
/// derivation run without a frame — same snapshot, same `ui.json`, same rows.
#[test]
fn the_boundary_answer_is_the_frames_derivation_without_a_frame() {
    use crate::boundary::reply::Reply;
    use crate::boundary::{Query, answer};
    use crate::cli_outbound::Cli;
    use crate::ui_state::UiState;

    let h = Harness::new();
    let (_c, mut model) = h.model();
    model.focus_workspace(&h.ws);
    let deps = model.boundary_deps(&Cli::new("/no/lernie"), &Cli::new("/no/bl"));

    // The frame's rows, and the same query answered through the chokepoint.
    let frame_rows = model.conversations(500);
    assert_eq!(frame_rows.len(), 1, "the fixture's one conversation");
    let Ok(Reply::Conversations(via_answer)) = model.answer(
        &deps,
        &Query::Conversations {
            workspace: h.ws.clone(),
        },
        500,
    ) else {
        panic!("conversations answers conversations");
    };
    assert_eq!(via_answer, frame_rows, "one derivation, two callers");

    // The truly frameless spelling: the published snapshot + a fresh read of
    // the durable ui.json — what the deposit consumer does.
    let snap = crate::state::latest_snapshot(&model.snapshot_cell());
    let ui = UiState::open(model.ui_json_path());
    assert_eq!(
        answer::conversations(&snap, &ui, &h.ws, 500),
        frame_rows,
        "the snapshot derivation run without a frame"
    );

    // The workspace rollups agree the same way.
    let Ok(Reply::Workspaces(rows)) = model.answer(&deps, &Query::Workspaces, 500) else {
        panic!("workspaces answers workspaces");
    };
    assert_eq!(rows.len(), 1);
    assert_eq!(
        (rows[0].attention, rows[0].agents, rows[0].running),
        model.workspace_stats(&h.ws)
    );
}

#[test]
fn the_live_mark_seats_the_focused_conversation_root_first() {
    let h = Harness::new();
    let (_c, mut model) = h.model();
    // Nothing selected is no seats — which the mark paints as itself at rest,
    // never as a case it branches on.
    assert!(model.mark_seats().is_empty(), "nothing selected, no seats");
    model.focus_agent(&h.ws, "c-1");
    assert_eq!(
        model
            .mark_seats()
            .iter()
            .map(|s| s.doing)
            .collect::<Vec<_>>(),
        vec![crate::nav::convs::Doing::Idle],
        "a settled conversation seats its root, idle"
    );
    // The same conversation mid-call, with a child running a tool: the eye is
    // the root's own business, the node its child's.
    let mut streaming = agent("c-1", AgentState::InFlight);
    streaming.stream.last_delta = Some(crate::git_tree::Delta::Thinking);
    let mut kid = agent("c-1-20260427T120100Z-bbbb", AgentState::Live);
    kid.tool_calls = vec![crate::git_tree::ToolCall {
        tool_id: "toolu_1".into(),
        name: Some("Bash".into()),
        start_unix: None,
        state: crate::git_tree::ToolCallState::InFlight,
    }];
    model.deriver.trees.get_mut(&h.ws).unwrap().agents = vec![streaming, kid];
    model.publish();
    assert_eq!(
        model
            .mark_seats()
            .iter()
            .map(|s| s.doing)
            .collect::<Vec<_>>(),
        vec![
            crate::nav::convs::Doing::Thinking,
            crate::nav::convs::Doing::Tools
        ]
    );
    // A selected id the tree does not carry roots no conversation, and an
    // unfetched workspace has no tree to ask.
    model.focus_agent(&h.ws, "ghost");
    assert!(model.mark_seats().is_empty());
    model.focus_agent(std::path::Path::new("/no/such"), "c-1");
    assert!(model.mark_seats().is_empty());
}

#[test]
fn the_in_flight_strip_follows_the_focused_conversation() {
    // The frame's wall clock, minted by the shell and handed in (§5.1 #28's
    // elapsed ticks off it, so the model itself stays clock-free).
    let now = 1_800_000_000;
    let h = Harness::new();
    let (_c, mut model) = h.model();
    assert!(
        model.flight_strip(now).is_none(),
        "nothing selected, no strip"
    );
    // A settled conversation: selected, snapshotted, and still no strip —
    // §7.2's promise that an idle window pays nothing.
    model.focus_agent(&h.ws, "c-1");
    assert!(model.flight_strip(now).is_none(), "at rest, no strip");
    // The same conversation with a model call open. The state needs a held
    // flock no fixture can take, so the row is injected and published exactly
    // as a real derivation would have handed it over.
    let mut streaming = agent("c-1", AgentState::InFlight);
    streaming.goal_name = Some("stench-pug".into());
    streaming.stream.text = Some("hi".into());
    streaming.call_start_unix = Some(now - 42);
    model.deriver.trees.get_mut(&h.ws).unwrap().agents = vec![streaming];
    model.publish();
    let strip = model.flight_strip(now).unwrap();
    assert_eq!(strip.class, crate::nav::convs::Flight::Inference);
    assert_eq!(strip.facts, "stench-pug · 2 chars streamed · 42s");
    // The elapsed is recomputed per frame off the same snapshot fact — a later
    // clock reads a longer call with nothing republished.
    assert_eq!(
        model.flight_strip(now + 18).unwrap().facts,
        "stench-pug · 2 chars streamed · 1m"
    );
    // A selected id the tree does not carry roots no conversation, and an
    // unfetched workspace has no tree to ask.
    model.focus_agent(&h.ws, "ghost");
    assert!(model.flight_strip(now).is_none());
    model.focus_agent(std::path::Path::new("/no/such"), "c-1");
    assert!(model.flight_strip(now).is_none());
}
