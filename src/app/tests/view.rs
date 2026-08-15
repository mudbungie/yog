//! Tests for the model's read surface (`app/view.rs`): the roster rollups, the
//! focused-agent lookup, and the §11 accessories that crossed to the boundary
//! with bl-296f — the altitude-0 chrome and the selection's own live detail,
//! asked here through `test_support::chrome` exactly as the seat asks them.
//!
//! Split from `tests/focus.rs` alongside the production split — those are
//! reads; `tests/focus.rs` keeps the focus and seen-acknowledgement gestures.

use super::harness::Rig;
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
    model.focus_workspace(&crate::naming::leaf(&h.ws));
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
    model.focus_workspace(&crate::naming::leaf(&h.ws));
    let rows = model.conversations(10);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].root_id, "c-1");
    assert_eq!(rows[0].attention, 1);
    assert_eq!(rows[0].members, 1);
    assert_eq!(rows[0].depth, 0);
    assert_eq!(rows[0].direct, 0, "a lone root dispatched nobody");
    // The unfold is the SEAT's fold over the boundary's answer (bl-44e9), and
    // naming a childless row in the set reveals nothing — the flat list is the
    // all-collapsed case, so the two agree wherever there is no descent.
    let expanded = std::collections::HashSet::from(["c-1".to_owned()]);
    assert_eq!(model.visible_conversations(10, &expanded), rows);
    // No focus → the empty list (the general empty path, not a special case).
    model.focus_workspace("no-such-workspace");
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
    model.focus_workspace(&crate::naming::leaf(&h.ws));
    let deps = model.boundary_deps(&Cli::new("/no/lernie"), &Cli::new("/no/bl"));

    // What the chokepoint answers, and what a seat then paints from it. Since
    // bl-44e9 the frame has no derivation of its own to compare against — it
    // reads this reply like every other seat — so what parity means here is
    // that the *frameless* spelling and the chokepoint's are one value, and
    // that the seat's fold of it is the all-collapsed list.
    let Ok(Reply::Conversations(answered)) = model.answer(
        &deps,
        &Query::Conversations {
            workspace: crate::naming::leaf(&(h.ws.clone())),
        },
        500,
    ) else {
        panic!("conversations answers conversations");
    };

    // The truly frameless spelling: the published snapshot + a fresh read of
    // the durable ui.json — what the deposit consumer does.
    let snap = crate::state::latest_snapshot(&model.snapshot_cell());
    let ui = UiState::open(model.ui_json_path());
    assert_eq!(
        answer::conversations(&snap, &ui, &h.ws, 500),
        answered,
        "the snapshot derivation run without a frame"
    );
    let painted = model.conversations(500);
    assert_eq!(painted.len(), 1, "the fixture's one conversation");
    assert_eq!(
        painted,
        crate::nav::convs::visible(&answered, &std::collections::HashSet::new()),
        "a seat holding no fold paints the answer's root subset"
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
fn the_live_mark_seats_the_conversation_root_first() {
    let h = Harness::new();
    let (_c, mut model) = h.model();
    let seats = |m: &Rig, ws: &std::path::Path, id: &str| {
        crate::test_support::chrome::detail(m, ws, id, 0)
            .map(|view| view.seats.iter().map(|s| s.doing).collect::<Vec<_>>())
            .unwrap_or_default()
    };
    assert_eq!(
        seats(&model, &h.ws, "c-1"),
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
        seats(&model, &h.ws, "c-1"),
        vec![
            crate::nav::convs::Doing::Thinking,
            crate::nav::convs::Doing::Tools
        ]
    );
    // An id the tree does not carry roots no conversation, and an unfetched
    // workspace has no tree to ask — no seats either way, which is the mark at
    // rest rather than a case it branches on.
    assert!(seats(&model, &h.ws, "ghost").is_empty());
    assert!(seats(&model, std::path::Path::new("/no/such"), "c-1").is_empty());
}

#[test]
fn the_in_flight_strip_follows_the_conversation() {
    // The clock the elapsed segment is stamped against — the answer takes it,
    // so the derivation itself stays clock-free.
    let now = 1_800_000_000;
    let h = Harness::new();
    let (_c, mut model) = h.model();
    let strip = |m: &Rig, ws: &std::path::Path, id: &str, now| {
        crate::test_support::chrome::detail(m, ws, id, now).and_then(|view| view.strip)
    };
    // A settled conversation: answered, and still no strip — §7.2's promise
    // that an idle window pays nothing.
    assert!(
        strip(&model, &h.ws, "c-1", now).is_none(),
        "at rest, no strip"
    );
    // The same conversation with a model call open. The state needs a held
    // flock no fixture can take, so the row is injected and published exactly
    // as a real derivation would have handed it over.
    let mut streaming = agent("c-1", AgentState::InFlight);
    streaming.goal_name = Some("stench-pug".into());
    streaming.stream.text = Some("hi".into());
    streaming.call_start_unix = Some(now - 42);
    model.deriver.trees.get_mut(&h.ws).unwrap().agents = vec![streaming];
    model.publish();
    let open = strip(&model, &h.ws, "c-1", now).expect("a call is open");
    assert_eq!(open.class, crate::nav::convs::Flight::Inference);
    // No `stench-pug` segment: the streaming member IS the conversation, whose
    // §11 heading names it two lines above the strip (bl-3f70).
    assert_eq!(open.facts, "2 chars streamed · 42s");
    // The elapsed is recomputed off the same snapshot fact at whatever clock
    // the answer is derived on — a later ask reads a longer call with nothing
    // republished.
    assert_eq!(
        strip(&model, &h.ws, "c-1", now + 18)
            .expect("still open")
            .facts,
        "2 chars streamed · 1m"
    );
    // An id the tree does not carry roots no conversation, and an unfetched
    // workspace has no tree to ask.
    assert!(strip(&model, &h.ws, "ghost", now).is_none());
    assert!(strip(&model, std::path::Path::new("/no/such"), "c-1", now).is_none());
}
