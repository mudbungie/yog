//! Tables for the query chokepoint (§8.5): each family answered from a
//! hand-built snapshot, the same derivations the frame's view-models delegate
//! to — parity is the shared implementation, and these pin its behaviour.
//!
//! The §3.6 unmaking's own derivations are in [`confirm`], beside the module
//! they exercise — split off at §12's cap on the seam this directory already
//! has (`answer::confirm` is its own file).

mod confirm;

use super::*;
use crate::boundary::tests::{agent, bound_row, snapshot};
use crate::cli_outbound::Cli;
use crate::git_tree::AgentState;
use crate::opslog::{OpRow, Origin};
use crate::projects::join::JoinState;
use std::path::{Path, PathBuf};
use std::sync::Arc;

fn ui() -> UiState {
    // A path that never exists: every watermark reads unseen, nothing writes.
    UiState::open(PathBuf::from("/nonexistent/ui.json"))
}

fn ws() -> PathBuf {
    PathBuf::from("/names/alba")
}

/// A `Deps` wrapping `snap` — the six snapshot-only queries never touch its
/// other fields, so unspawnable binaries and a hermetic, nonexistent world
/// are enough (the §9 config family's own reads are tabled separately,
/// against the real hermetic world `boundary::config::tests` builds).
fn deps(snap: Snapshot) -> Deps {
    Deps {
        lernie: Cli::new("/no/such/lernie"),
        bl: Cli::new("/no/such/bl"),
        state_root: PathBuf::from("/nonexistent/state"),
        yog_binary: PathBuf::from("/no/such/yog"),
        world: crate::test_support::no_world(),
        home: PathBuf::from("/home/x"),
        yog_data_root: PathBuf::from("/data"),
        balls_state_root: PathBuf::from("/balls"),
        snapshot: Arc::new(snap),
        caller: crate::boundary::dispatch::Caller::default(),
    }
}

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
fn the_four_query_families_answer_from_one_snapshot() {
    let project = PathBuf::from("/proj");
    let mut snap = snapshot(
        &ws(),
        "alba",
        vec![agent("c-1", AgentState::Live, 100)],
        vec![bound_row(&project, "bl-1", &ws(), "alba")],
    );
    let op = |ts: &str| OpRow {
        ts: ts.into(),
        argv: "x".into(),
        cwd: String::new(),
        exit: 0,
        stdout: String::new(),
        stderr: String::new(),
        origin: Origin::World,
    };
    snap.ops = vec![op("1"), op("2"), op("3")];
    let join_rows = snap.join_rows.clone();
    let board_snap = snap.clone();
    let d = deps(snap);

    let Ok(Reply::Workspaces(rows)) = answer(&Query::Workspaces, &d, &ui(), 200) else {
        panic!("workspaces answers workspaces");
    };
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].agents, 1);
    assert!(rows[0].running);

    let Ok(Reply::Conversations(rows)) = answer(
        &Query::Conversations {
            workspace: crate::naming::leaf(&ws()),
        },
        &d,
        &ui(),
        200,
    ) else {
        panic!("conversations answers conversations");
    };
    assert_eq!(rows.len(), 1);

    let Ok(Reply::Balls(rows)) = answer(&Query::Balls, &d, &ui(), 200) else {
        panic!("balls answers balls");
    };
    assert_eq!(rows, join_rows);

    // The board is one altitude up on the same snapshot (VISION §5 V4): with no
    // live ball projection behind these join rows, it is honestly empty — the
    // general path with no inputs, not a bootstrap branch.
    let Ok(Reply::Board(board)) = answer(&Query::Board, &d, &ui(), 200) else {
        panic!("board answers board");
    };
    assert_eq!(board, crate::board::build(&board_snap, &ui(), 0));

    let Ok(Reply::Ops(rows)) = answer(&Query::Ops { max: 2 }, &d, &ui(), 200) else {
        panic!("ops answers ops");
    };
    assert_eq!(
        rows.iter().map(|r| r.ts.as_str()).collect::<Vec<_>>(),
        ["2", "3"],
        "the tail, newest last"
    );
    let Ok(Reply::Ops(all)) = answer(&Query::Ops { max: 99 }, &d, &ui(), 200) else {
        panic!();
    };
    assert_eq!(all.len(), 3, "a max past the tail takes everything");
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

/// Help is answered from the interface, not the world: the same rows come back
/// from a snapshot that carries nothing at all, which is why every seat may
/// answer it in place instead of depositing it (§8.5).
#[test]
fn help_is_answered_without_reading_the_world() {
    let d = deps(snapshot(&ws(), "alba", vec![], vec![]));
    let all = answer(&Query::Help { verb: None }, &d, &ui(), 0);
    let Ok(Reply::Help(rows)) = all else {
        panic!("not help");
    };
    assert_eq!(rows.len(), crate::boundary::help::table().len());

    let one = answer(
        &Query::Help {
            verb: Some("ack".to_owned()),
        },
        &d,
        &ui(),
        0,
    );
    let Ok(Reply::Help(rows)) = one else {
        panic!("not help");
    };
    assert_eq!(rows.len(), 1);
    assert_eq!(rows.first().map(|row| row.verb), Some("ack"));
}

#[test]
fn search_is_answered_here_too_because_every_seat_that_reaches_this_is_off_frame() {
    let mut snap = snapshot(&ws(), "alba", vec![], vec![]);
    snap.balls_by_project.insert(
        PathBuf::from("/proj"),
        vec![crate::projects::balls::Ball {
            id: "bl-1f2a".to_owned(),
            title: "the kraken".to_owned(),
            body: String::new(),
            claimant: None,
            blockers: vec![],
            parent: None,
            priority: 3,
            tags: vec![],
            created: None,
            updated: None,
            root_commit: None,
        }],
    );
    let d = deps(snap);
    let ask = |text: &str| {
        let Ok(Reply::Search(found)) = answer(
            &Query::Search {
                text: text.to_owned(),
            },
            &d,
            &ui(),
            200,
        ) else {
            panic!("search answers search");
        };
        found
    };
    let found = ask("kraken");
    assert_eq!(found.hits.len(), 1);
    assert_eq!(
        found.hits[0].at,
        crate::search::Address::Ball {
            project: PathBuf::from("/proj"),
            id: "bl-1f2a".to_owned(),
        }
    );
    assert_eq!(ask(""), crate::search::Found::default());
}
