//! The **free-function derivations** the query chokepoint is built out of
//! (§8.5) — the descent forest a seat folds, a conversation's bound ball, a
//! workspace's attention/running rollup, and the mint's occupied name set.
//! Split from [`super`] at §12's budget on the seam that file already had:
//! these are claims about one derivation over a hand-built snapshot, and the
//! beats left there are claims about `answer` **dispatching** to them.

use super::{ui, ws};
use crate::boundary::answer::*;
use crate::boundary::tests::{agent, bound_row, snapshot};
use crate::git_tree::AgentState;
use crate::projects::join::JoinState;
use std::path::{Path, PathBuf};

/// REMOTE §9.7's altitude ruling (bl-44e9): the answer is the whole descent
/// forest with its per-row rollups, and the all-collapsed list every seat used
/// to be handed is the **root subset** a seat selects out of it.
#[test]
fn conversations_answer_the_whole_forest_and_the_seat_folds_it() {
    let snap = snapshot(
        &ws(),
        "alba",
        vec![
            agent("c-1", AgentState::Live, 100),
            agent("c-1-w-1", AgentState::Quiescent, 90),
            agent("c-2", AgentState::Stopped, 50),
        ],
        vec![],
    );
    let rows = conversations(&snap, &ui(), &ws(), 200);
    assert_eq!(
        rows.iter().map(|r| r.root_id.as_str()).collect::<Vec<_>>(),
        ["c-1", "c-1-w-1", "c-2"],
        "every member of the forest, in paint order"
    );
    assert_eq!(
        rows[0].members, 2,
        "the rollup is the subtree's, not the fold's"
    );
    // No expanded set crosses, so the answer carries none: a seat holding no
    // fold selects the roots, which is the list this query used to answer.
    let collapsed = crate::nav::convs::visible(&rows, &std::collections::HashSet::new());
    assert_eq!(
        collapsed
            .iter()
            .map(|r| r.root_id.as_str())
            .collect::<Vec<_>>(),
        ["c-1", "c-2"]
    );
    assert!(conversations(&snap, &ui(), Path::new("/other"), 200).is_empty());
}

#[test]
fn conv_ball_reads_the_join_or_renders_the_stray_id() {
    let project = PathBuf::from("/proj");
    let snap = snapshot(
        &ws(),
        "alba",
        vec![],
        vec![bound_row(&project, "bl-1", &ws(), "alba")],
    );
    let hit = conv_ball(&snap, "bl-1");
    assert_eq!(hit.state, Some(JoinState::Bound));
    assert_eq!(hit.title.as_deref(), Some("title of bl-1"));
    let miss = conv_ball(&snap, "bl-9");
    assert_eq!(miss.id, "bl-9");
    assert_eq!(miss.state, None);
}

#[test]
fn workspace_stats_roll_up_attention_and_running() {
    let mut waiting = agent("c-2", AgentState::Quiescent, 10);
    waiting.notify_oid = Some("n".repeat(40));
    let snap = snapshot(
        &ws(),
        "alba",
        vec![agent("c-1", AgentState::InFlight, 100), waiting],
        vec![],
    );
    let (attention, agents, running) = workspace_stats(&snap, &ui(), &ws());
    assert_eq!(agents, 2);
    assert!(running, "an InFlight member runs");
    assert_eq!(attention, 1, "the notify mark begs attention");
    assert_eq!(
        workspace_stats(&snap, &ui(), Path::new("/other")),
        (0, 0, false),
        "an underived workspace contributes zeros"
    );
}
#[test]
fn names_in_reads_the_name_fact_children_included() {
    // The mint's occupied set (§3.3, bl-08f2): each agent's name_fact — the
    // lernie-stored blob, with the legacy goal stamp while pre-0.0.4 roots
    // live. A named descent child occupies too: lernie refuses a taken name at
    // fire, so the mint must see everything lernie would.
    let mut named = agent("c-1", AgentState::Live, 1);
    named.name = Some("pale-otter".into());
    let mut legacy = agent("c-2", AgentState::Live, 2);
    legacy.goal_name = Some("brave-fox".into());
    let mut child = agent("c-1-x1", AgentState::Live, 3);
    child.name = Some("quiet-heron".into());
    let snap = snapshot(
        &ws(),
        "alba",
        vec![named, legacy, child, agent("c-3", AgentState::Live, 4)],
        vec![],
    );
    assert_eq!(
        names_in(&snap, &ws()),
        ["pale-otter", "brave-fox", "quiet-heron"]
    );
    assert!(names_in(&snap, Path::new("/other")).is_empty());
}
