//! Tables for the query chokepoint (§8.5): each family answered from a
//! hand-built snapshot, the same derivations the frame's view-models delegate
//! to — parity is the shared implementation, and these pin its behaviour.
//!
//! The §3.6 unmaking's own derivations are in [`confirm`], beside the module
//! they exercise — split off at §12's cap on the seam this directory already
//! has (`answer::confirm` is its own file).

/// The free-function derivations `answer` dispatches to — its own file per
/// §12's budget, on the seam between a derivation and the dispatch over it.
mod derive;

mod confirm;
/// The one query whose answer is a **sequence** (REMOTE §3, bl-73e7), answered
/// here as one frame — its own file at §12's budget, on the seam that every
/// other beat in this directory is about a derivation and this one is about a
/// *cadence* every intake shares the bottom of.
mod follow;

use super::*;
use crate::boundary::tests::{agent, bound_row, snapshot};
use crate::cli_outbound::Cli;
use crate::git_tree::AgentState;
use crate::opslog::{OpRow, Origin};
use crate::projects::join::JoinState;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub(super) fn ui() -> UiState {
    // A path that never exists: every watermark reads unseen, nothing writes.
    UiState::open(PathBuf::from("/nonexistent/ui.json"))
}

pub(super) fn ws() -> PathBuf {
    PathBuf::from("/names/alba")
}

/// A `Deps` wrapping `snap` — the six snapshot-only queries never touch its
/// other fields, so unspawnable binaries and a hermetic, nonexistent world
/// are enough (the §9 config family's own reads are tabled separately,
/// against the real hermetic world `boundary::config::tests` builds).
pub(super) fn deps(snap: Snapshot) -> Deps {
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

    let Ok(Reply::Workspaces(view)) = answer(&Query::Workspaces, &d, &ui(), 200) else {
        panic!("workspaces answers workspaces");
    };
    assert_eq!(view.rows.len(), 1);
    assert_eq!(view.rows[0].agents, 1);
    assert!(view.rows[0].running);
    // The §7.2 notes ride the same answer (bl-b4b5): the fixture's derivation
    // is stamped at epoch and the caller's clock reads 200, so the answer says
    // how far behind what it is showing is — a subtraction the boundary can
    // make only because the stamp is wall-clock.
    assert_eq!(view.stale.as_deref(), Some("derivation 200 s behind"));
    assert_eq!(view.growth, None, "a quiet world says nothing");

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
