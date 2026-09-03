//! The §11 conversation seat's derivation (REMOTE §9.4, bl-1eb0). Every field
//! is a fold some other module tests; what is pinned here is which fold each
//! field is, and that absence is answered rather than refused.

use std::path::Path;

use super::*;
use crate::boundary::tests::{agent as agent_row, snapshot};
use crate::control::hold::Held;
use crate::git_tree::AgentState;
use crate::ui_state::UiState;

/// A durable document nothing writes and no price table backs — so every §3.5
/// figure this file reads is tokens-only, which is the severability gate's own
/// empty arm.
fn ui() -> UiState {
    UiState::open(std::path::PathBuf::from("/nonexistent/ui.json"))
}

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

    let view = agent(&snap, &ui(), ws, CHILD, 0);
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
    let at_root = agent(&snap, &ui(), ws, ROOT, 0);
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

    let view = agent(&snap, &ui(), ws, ROOT, 0);
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
        let view = agent(&snap, &ui(), ws, id, 0);
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
        let view = agent(&snap, &ui(), at, id, 0);
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

/// One bill for one agent, with a distinct prompt so a transposed subject is
/// visible in both figures at once.
fn bill(conv: &str, tokens: u64) -> crate::budgets::StepBill {
    let spend = crate::budgets::BudgetSpend {
        input_tokens: tokens,
        output_tokens: 0,
        cache_read_tokens: 0,
        cache_write_tokens: 0,
    };
    crate::budgets::StepBill {
        conv: conv.to_owned(),
        seq: "001".to_owned(),
        model: Some("m".to_owned()),
        spend,
        last_usage: spend,
        wall_secs: 0,
    }
}

/// **A subagent's cost is its own** (bl-131d). The spend fold and the fullness
/// beside it are taken over the agent this answer is *about*, not over its
/// root: a child asked what it spent used to be handed its parent's totals byte
/// for byte, under an attribution reading *one conversation* and with nothing
/// in the reply saying whose number it was. Selecting the root still folds the
/// whole tree, because a root's branch is the tree.
#[test]
fn a_child_answers_its_own_spend_and_its_own_context() {
    let ws = Path::new("/w");
    let mut snap = snapshot(
        ws,
        "alba",
        vec![
            agent_row(ROOT, AgentState::Quiescent, 1),
            agent_row(CHILD, AgentState::InFlight, 2),
        ],
        vec![],
    );
    snap.bills
        .insert(ws.to_path_buf(), vec![bill(ROOT, 70), bill(CHILD, 30)]);
    snap.windows.insert("m".to_owned(), 100);

    let at_child = agent(&snap, &ui(), ws, CHILD, 0);
    assert_eq!(at_child.spend.tokens.input_tokens, 30, "the child's own");
    assert_eq!(
        at_child.spend.attribution,
        crate::spend::Attribution::Conversations(1),
        "one subject, and now it is the selected one"
    );
    assert_eq!(at_child.context.map(|f| f.percent()), Some(30));

    // The root's branch is the whole tree, so nothing was lost: the number the
    // old answer gave for both is still exactly one selection away.
    let at_root = agent(&snap, &ui(), ws, ROOT, 0);
    assert_eq!(at_root.spend.tokens.input_tokens, 100, "root and descent");
    assert_eq!(at_root.context.map(|f| f.percent()), Some(70));
}
