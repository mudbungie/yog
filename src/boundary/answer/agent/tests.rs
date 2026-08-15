//! The §11 conversation seat's derivation (REMOTE §9.4, bl-1eb0). Every field
//! is a fold some other module tests; what is pinned here is which fold each
//! field is, and that absence is answered rather than refused.

use std::path::Path;

use super::*;
use crate::boundary::tests::{agent as agent_row, snapshot};
use crate::control::hold::Held;
use crate::git_tree::AgentState;

const ROOT: &str = "20260801T000000Z-r0";
const CHILD: &str = "20260801T000000Z-r0-20260801T000100Z-c0";

/// The seat resolves the **conversation**, not the row: a selected member reads
/// its root's id, its root's name and its root's flight — that is what the §11
/// centre header paints — while `state`, `tip` and the two §8.2 gates stay the
/// selection's own.
#[test]
fn a_member_reads_its_own_state_and_its_conversations_name() {
    let ws = Path::new("/w");
    let mut root = agent_row(ROOT, AgentState::Quiescent, 1);
    root.name = Some("pennant".to_owned());
    let mut child = agent_row(CHILD, AgentState::InFlight, 2);
    child.tip_oid = "b".repeat(40);
    let snap = snapshot(ws, "alba", vec![root, child], vec![]);

    let view = agent(&snap, ws, CHILD);
    assert_eq!(view.agent_id, CHILD);
    assert_eq!(view.root, ROOT);
    // The chain §11's visible-selection invariant unfolds, outermost first.
    assert_eq!(view.ancestors, vec![ROOT.to_owned()]);
    assert_eq!(view.name, "pennant");
    assert!(!view.display_only);
    assert_eq!(view.tip, "b".repeat(40));
    assert_eq!(view.state, AgentState::InFlight);
    assert_eq!(view.flight, Some(crate::nav::convs::Flight::Inference));
    // Stop is the selection's own liveness; the cascade is the looser prefix
    // test, so the child — which nothing descends from — offers none.
    assert!(view.stoppable);
    assert!(!view.stop_children);

    // The root of the same conversation: at rest itself, and the cascade is
    // offered because something below it is not.
    let at_root = agent(&snap, ws, ROOT);
    assert_eq!(at_root.state, AgentState::Quiescent);
    assert!(at_root.ancestors.is_empty(), "a root opens nothing");
    assert!(at_root.present && at_root.nudgeable && at_root.held.is_none());
    assert!(!at_root.stoppable);
    assert!(at_root.stop_children);
    assert_eq!(at_root.flight, view.flight, "one conversation, one class");
}

/// The §6 marks ride in badge order, and the legacy §3.3 rung is stated as its
/// own fact so no seat reads an unaddressable name as an addressable one.
#[test]
fn the_marks_and_the_legacy_naming_rung_are_stated() {
    let ws = Path::new("/w");
    let mut root = agent_row(ROOT, AgentState::Stopped, 1);
    root.goal_name = Some("relic".to_owned());
    root.notify_oid = Some("c".repeat(40));
    root.held = Some(Held {
        tool_use_id: "toolu_1".to_owned(),
        tool: "Bash".to_owned(),
        reason: "unconfined".to_owned(),
    });
    let snap = snapshot(ws, "alba", vec![root], vec![]);

    let view = agent(&snap, ws, ROOT);
    assert_eq!(view.name, "relic");
    assert!(view.display_only);
    assert_eq!(view.marks, vec![AgentMark::Notified, AgentMark::Held]);
    // The badge is the fold; the park's own sentence rides beside it, because
    // the §8.6 answer controls need the words the badge cannot carry.
    assert_eq!(view.held.map(|h| h.tool), Some("Bash".to_owned()));
}

/// **The forest already answers everything the §11 seat reads synchronously**
/// (REMOTE §9.7, bl-48ae) — the pin the migration off `focused_conversation`
/// rests on. `Query::Conversations` lands the whole descent forest, and
/// [`nav::convs::selection`](crate::nav::convs::selection) is a pure fold out of
/// it, so a seat pays no second ask for the composer's target line, §11's
/// ancestor unfold or either §8.2 gate. Every fact both projections carry is
/// asserted equal here, over a member, a root and an id nothing carries —
/// because two projections of one derivation that nothing holds together are
/// two facts waiting to disagree.
#[test]
fn the_forest_answers_every_fact_the_seat_reads_off_the_selection() {
    let ws = Path::new("/w");
    let mut root = agent_row(ROOT, AgentState::Quiescent, 1);
    root.name = Some("pennant".to_owned());
    let child = agent_row(CHILD, AgentState::InFlight, 2);
    let snap = snapshot(ws, "alba", vec![root, child], vec![]);
    let rows = crate::boundary::answer::conversations(
        &snap,
        &crate::ui_state::UiState::open("/nonexistent/ui.json".into()),
        ws,
        100,
    );
    for id in [ROOT, CHILD, "20260801T000000Z-gh0"] {
        let view = agent(&snap, ws, id);
        let seat = crate::nav::convs::selection(&rows, id);
        assert_eq!(seat.agent_id, view.agent_id, "{id}");
        assert_eq!(seat.root, view.root, "{id}");
        assert_eq!(seat.ancestors, view.ancestors, "{id}");
        assert_eq!(seat.name, view.name, "{id}");
        assert_eq!(seat.display_only, view.display_only, "{id}");
        assert_eq!(seat.flight, view.flight, "{id}");
        assert_eq!(seat.present, view.present, "{id}");
        assert_eq!(seat.stoppable, view.stoppable, "{id}");
        assert_eq!(seat.stop_children, view.stop_children, "{id}");
    }
}

/// Absence is a value: an id this workspace does not carry, and a workspace
/// with no derived tree at all, both read as their own root — unnamed,
/// stopped, unmarked, nothing offered — rather than refusing.
#[test]
fn an_agent_the_snapshot_does_not_carry_is_answered_not_refused() {
    let ws = Path::new("/w");
    let snap = snapshot(
        ws,
        "alba",
        vec![agent_row(ROOT, AgentState::Live, 1)],
        vec![],
    );
    for (at, id) in [(ws, "who"), (Path::new("/elsewhere"), ROOT)] {
        let view = agent(&snap, at, id);
        assert_eq!(view.root, id);
        assert_eq!(view.name, crate::nav::convs::id_floor(id));
        assert_eq!(view.tip, "");
        assert_eq!(view.state, AgentState::Stopped);
        assert!(view.marks.is_empty());
        assert_eq!(view.flight, None);
        assert!(view.ancestors.is_empty());
        assert!(!view.present && !view.nudgeable && !view.stoppable);
        assert!(!view.stop_children && view.held.is_none());
    }
}
