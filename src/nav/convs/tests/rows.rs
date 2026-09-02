//! Table tests for the §11 row projection ([`super::super::row`]): the
//! one-row-per-root build, the §11 sort, the aggregated badge, the capped
//! preview, the ball stamp and the age labels. The structural questions —
//! members, root_of, liveness — are its parent's.

use super::*;

#[test]
fn one_row_per_root_with_subtree_member_counts() {
    // Two conversations: `r1-0` with one child, `r2-0` alone. Every id here is
    // descent-grammar shaped
    // (§2.3): two hyphen-free tokens per segment, so `r1-0-c-1` is `r1-0`'s
    // child and nothing shorter would be (`git_tree::descent`).
    let agents = [
        agent("r1-0", AgentState::Quiescent, 10),
        agent("r1-0-c-1", AgentState::Quiescent, 20),
        agent("r2-0", AgentState::Quiescent, 30),
    ];
    let rows = build(&agents, "/ws", &unseen, 100, &plain, &[]);
    assert_eq!(rows.len(), 2);
    // Recency: r2-0 (30) leads r1-0 (20).
    assert_eq!(rows[0].root_id, "r2-0");
    assert!(!rows[0].has_children());
    assert_eq!(rows[1].root_id, "r1-0");
    assert_eq!(rows[1].members, 2);
    assert!(rows[1].has_children());
    assert_eq!(rows[1].age_secs, 80, "age from the subtree's latest action");
    // bl-b7d9: the stamp is that same latest action, undistanced — the
    // conversation's own child (20), never the root's own 10 and never the
    // clock the age was measured against.
    assert_eq!(rows[1].last_active_unix, 20);
    assert_eq!(rows[0].last_active_unix, 30);
}

#[test]
fn sort_is_last_action_descending_and_nothing_else() {
    // bl-cad5: the §11 rank tiers are gone. `dead` bears attention (an unseen
    // rest, §6 rule 2) and `running` holds a driver, but both acted long ago,
    // so both sink below the idle rows that acted recently. The acked tip on
    // the idle pair keeps them out of rule 2 — the point is that it no longer
    // *matters* which rows carry attention, only when each last moved.
    let mut idle_new = agent("idle-new", AgentState::Quiescent, 90);
    idle_new.tip_oid = "seen-tip".into();
    let mut idle_old = agent("idle-old", AgentState::Quiescent, 10);
    idle_old.tip_oid = "seen-tip".into();
    let agents = [
        idle_new,
        idle_old,
        agent("running", AgentState::Live, 5),
        agent("dead", AgentState::Stopped, 1),
    ];
    let seen = |k, _: &str, _: &str, o: &str| k == SeenKind::Stopped && o == "seen-tip";
    let rows = build(&agents, "/ws", &seen, 100, &plain, &[]);
    let ids: Vec<&str> = rows.iter().map(|r| r.root_id.as_str()).collect();
    assert_eq!(ids, ["idle-new", "idle-old", "running", "dead"]);
    // Attention and liveness survive as badges — they just stopped reordering.
    assert_eq!(rows[0].attention, 0);
    assert_eq!(rows[3].attention, 1, "the flag still renders, it just sank");
    assert_eq!(rows[2].state, AgentState::Live);
}

#[test]
fn recency_is_the_last_action_of_any_kind_not_the_last_commit() {
    // The other direction (bl-cad5): a conversation whose *tip* is ancient but
    // whose live tail / freshly delivered message just landed leads one with a
    // newer tip and nothing since. `last_action_unix` is the folded fact
    // (git_tree::enumerate); the row reads it for both the sort and the age.
    let mut streaming = agent("streaming", AgentState::InFlight, 10);
    streaming.last_action_unix = 95;
    let fresher_tip = agent("idle", AgentState::Quiescent, 40);
    let rows = build(&[fresher_tip, streaming], "/ws", &unseen, 100, &plain, &[]);
    let ids: Vec<&str> = rows.iter().map(|r| r.root_id.as_str()).collect();
    assert_eq!(ids, ["streaming", "idle"], "last action, not last commit");
    assert_eq!(rows[0].age_secs, 5, "the age label reads the same one fact");
    assert_eq!(rows[1].age_secs, 60);
}

#[test]
fn equal_last_action_breaks_on_root_id_ascending() {
    // The deterministic tail (I9) is unchanged: same instant, id order.
    let agents = [
        agent("b-0", AgentState::Quiescent, 7),
        agent("a-0", AgentState::Quiescent, 7),
        agent("c-0", AgentState::Quiescent, 7),
    ];
    let rows = build(&agents, "/ws", &unseen, 10, &plain, &[]);
    let ids: Vec<&str> = rows.iter().map(|r| r.root_id.as_str()).collect();
    assert_eq!(ids, ["a-0", "b-0", "c-0"]);
}

#[test]
fn subtree_state_aggregates_in_flight_over_live_over_root() {
    // A quiescent root with a streaming child reads InFlight (and pulses).
    let streaming = [
        agent("r-0", AgentState::Quiescent, 1),
        agent("r-0-c-1", AgentState::InFlight, 2),
    ];
    let row = &build(&streaming, "/ws", &unseen, 10, &plain, &[])[0];
    assert_eq!(row.state, AgentState::InFlight);
    assert_eq!(row.flight, Some(crate::nav::convs::Flight::Inference));
    // A live child (no stream) reads Live — and pulses as a subagent, not as
    // inference: the badge aggregates the state, the class says what the work is.
    let live = [
        agent("r-0", AgentState::Quiescent, 1),
        agent("r-0-c-1", AgentState::Live, 2),
    ];
    let row = &build(&live, "/ws", &unseen, 10, &plain, &[])[0];
    assert_eq!(row.state, AgentState::Live);
    assert_eq!(row.flight, Some(crate::nav::convs::Flight::Subagents));
    // All settled: the root's own state decides, its uncertainty carried.
    let mut root = agent("r-0", AgentState::Stopped, 1);
    root.state_uncertain = true;
    let settled = [root, agent("r-0-c-1", AgentState::Quiescent, 2)];
    let row = &build(&settled, "/ws", &unseen, 10, &plain, &[])[0];
    assert_eq!(row.state, AgentState::Stopped);
    assert!(row.uncertain);
}

#[test]
fn deciding_agents_uncertainty_is_carried() {
    let mut child = agent("r-0-c-1", AgentState::InFlight, 2);
    child.state_uncertain = true;
    let agents = [agent("r-0", AgentState::Quiescent, 1), child];
    assert!(build(&agents, "/ws", &unseen, 10, &plain, &[])[0].uncertain);
}

#[test]
fn preview_is_the_roots_first_line_capped_with_streaming_fallback() {
    let mut a = agent("r-0", AgentState::Quiescent, 1);
    a.preview = Some("first line\nsecond line".into());
    assert_eq!(
        build(&[a], "/ws", &unseen, 10, &plain, &[])[0].preview,
        "first line"
    );
    let mut b = agent("r-0", AgentState::InFlight, 1);
    b.stream.text = Some("streamed so far".into());
    assert_eq!(
        build(&[b], "/ws", &unseen, 10, &plain, &[])[0].preview,
        "streamed so far",
        "streaming text backfills a missing request preview"
    );
    let mut c = agent("r-0", AgentState::Quiescent, 1);
    c.preview = Some("x".repeat(200));
    assert_eq!(
        build(&[c], "/ws", &unseen, 10, &plain, &[])[0]
            .preview
            .chars()
            .count(),
        80,
        "capped at the preview cap"
    );
    let d = agent("r-0", AgentState::Quiescent, 1);
    assert_eq!(build(&[d], "/ws", &unseen, 10, &plain, &[])[0].preview, "");
}

#[test]
fn the_ball_badge_is_the_roots_goal_stamp_coloured_by_the_join() {
    use crate::projects::join::JoinState;
    // The resolver is the §3.5 join: the stamped id `b1` is Bound with a title.
    let resolve = |id: &str| ConvBall {
        id: id.to_owned(),
        state: Some(JoinState::Bound),
        title: Some("Do the thing".to_owned()),
        badge: None,
    };
    // A root carrying the goal stamp; its quiescent child stamps nothing (only the
    // root's goal.md is a yog ball attribution).
    let mut root = agent("r-0", AgentState::Quiescent, 2);
    root.goal_ball = Some("b1".to_owned());
    let child = agent("r-0-c-1", AgentState::Quiescent, 1);
    let rows = build(&[root, child], "/ws", &unseen, 10, &resolve, &[]);
    let ball = rows[0].ball.as_ref().expect("the root's stamped ball");
    assert_eq!(ball.id, "b1");
    assert_eq!(ball.state, Some(JoinState::Bound));
    assert_eq!(ball.title.as_deref(), Some("Do the thing"));
    // A conversation with no stamp carries no ball.
    let bare = agent("bare-0", AgentState::Quiescent, 1);
    assert!(
        build(&[bare], "/ws", &unseen, 10, &plain, &[])[0]
            .ball
            .is_none()
    );
}

#[test]
fn age_clamps_clock_skew_to_zero() {
    let a = agent("r-0", AgentState::Quiescent, 100);
    assert_eq!(build(&[a], "/ws", &unseen, 50, &plain, &[])[0].age_secs, 0);
}

#[test]
fn age_label_buckets() {
    assert_eq!(age_label(-5), "0s");
    assert_eq!(age_label(42), "42s");
    assert_eq!(age_label(420), "7m");
    assert_eq!(age_label(7200), "2h");
    assert_eq!(age_label(200_000), "2d");
}

/// **The roster's passive sighting of a provider refusal** (bl-b43b). The
/// badge set is frozen at four (§5.1 #9), so a conversation refused at its
/// first model call comes to rest `stopped` exactly as one the operator
/// stopped — and a list where the two read identically is a list that cannot
/// be scanned.
#[test]
fn a_refused_conversation_paints_bad_and_a_stopped_one_does_not() {
    let stopped = agent("r1-0", AgentState::Stopped, 10);
    let mut refused = agent("r2-0", AgentState::Stopped, 20);
    refused.failure = Some(r#"{"type":"error","status":401,"message":"Unauthorized"}"#.to_owned());
    let rows = build(&[stopped, refused], "/ws", &unseen, 100, &plain, &[]);

    let tone = |id: &str| {
        rows.iter()
            .find(|r| r.root_id == id)
            .map(|r| r.tone)
            .expect(id)
    };
    assert_eq!(tone("r2-0"), Tone::Bad);
    assert_eq!(tone("r1-0"), Tone::Plain);
    assert_eq!(
        rows.iter().map(|r| r.state).collect::<Vec<_>>(),
        vec![AgentState::Stopped, AgentState::Stopped],
        "the badge is the same word for both, which is the whole defect"
    );
    // …and the hue's own words ride with it (bl-9b88): a red row that cannot
    // say what is wrong with it is a row the operator must open to learn the
    // one thing every red row in the workspace says.
    let says = |id: &str| {
        rows.iter()
            .find(|r| r.root_id == id)
            .and_then(|r| r.failure.clone())
    };
    assert_eq!(says("r2-0").as_deref(), Some("Unauthorized"));
    assert_eq!(says("r1-0"), None);
}
