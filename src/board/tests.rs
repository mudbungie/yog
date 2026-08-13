//! The V4 board's derivations (STORIES S13).
//!
//! Every assertion here is against a fact that already had an owner: balls'
//! ladder and its close-blocker rule, the §3.5 join, the §3.3 goal stamp, and
//! the worker's `steps/` fold. The board's own contribution is the crossing,
//! and that is what these hold.

mod rollup;

mod fixture;

use fixture::{NOW, WS_A, WS_B, agent, ball, blocks, join, ui_doc, world};

use super::{Board, Column, build, column, descendants};
use crate::app::Snapshot;
use crate::projects::balls::Status;
use crate::projects::join::{JoinRow, JoinState};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Instant;

/// The column table, exhaustively — the two axes crossed. The three ladder
/// rungs are balls' words verbatim, which is the "no second status model"
/// claim made mechanical.
#[test]
fn the_four_columns_are_the_ladder_crossed_with_the_close_gate() {
    assert_eq!(column(Status::Ready, false), Column::Ready);
    assert_eq!(column(Status::Ready, true), Column::Gated);
    assert_eq!(column(Status::Blocked, false), Column::Blocked);
    assert_eq!(column(Status::Blocked, true), Column::Blocked);
    assert_eq!(column(Status::Claimed, false), Column::Claimed);
    assert_eq!(
        column(Status::Claimed, true),
        Column::Claimed,
        "a drone holds it and is working; the gate renders on the row, not as its bucket"
    );
    for status in Status::ALL {
        assert_eq!(
            column(status, false).word(),
            status.word(),
            "the board speaks balls' own vocabulary"
        );
    }
    assert_eq!(Column::ALL.len(), 4);
    assert_eq!(Column::Gated.word(), "gated");
}

/// A board over one project: each ball lands in the column its own stored
/// facts put it in, and a gated row names the ball whose close mints its gate.
#[test]
fn every_ball_lands_in_its_column_and_a_gate_names_what_mints_it() {
    let w = world(
        vec![
            ball("bl-ready", None, vec![]),
            ball("bl-gated", None, vec![blocks("bl-claim", "close")]),
            ball("bl-blockd", None, vec![blocks("bl-claim", "claim")]),
            ball("bl-claim", Some("alfa"), vec![]),
        ],
        vec![
            join("bl-ready", JoinState::ReadyStartable, None, None),
            join("bl-gated", JoinState::ReadyStartable, None, None),
            join("bl-blockd", JoinState::Blocked, None, None),
            join("bl-claim", JoinState::Bound, Some(WS_A), Some("alfa")),
        ],
        vec![(WS_A, vec![agent("conv1", Some("bl-claim"), Some("Cobalt"))])],
    );
    let board = w.board();
    let word = |id: &str| {
        board
            .rows
            .iter()
            .find(|r| r.id == id)
            .map(|r| r.column.word())
    };
    assert_eq!(word("bl-ready"), Some("ready"));
    assert_eq!(word("bl-gated"), Some("gated"));
    assert_eq!(word("bl-blockd"), Some("blocked"));
    assert_eq!(word("bl-claim"), Some("claimed"));
    for column in Column::ALL {
        assert_eq!(board.count(column), 1, "{column:?}");
        assert_eq!(board.in_column(column).len(), 1);
    }

    let gated = board.rows.iter().find(|r| r.id == "bl-gated").unwrap();
    assert_eq!(gated.gates.len(), 1);
    assert_eq!(gated.gates[0].id, "bl-claim");
    assert_eq!(
        gated.gates[0].title, "bl-claim title",
        "what mints the gate"
    );

    // The claimed row shows its drone as the conversation it is, and its spend.
    let claimed = board.rows.iter().find(|r| r.id == "bl-claim").unwrap();
    assert_eq!(claimed.drones.len(), 1);
    assert_eq!(claimed.drones[0].root_id, "conv1");
    assert_eq!(claimed.drones[0].name, "Cobalt");
    assert_eq!(claimed.state, JoinState::Bound);
    assert_eq!(claimed.spend.as_ref().unwrap().cost.unwrap().usd(), "$1.00");
    assert!(
        board
            .rows
            .iter()
            .find(|r| r.id == "bl-ready")
            .unwrap()
            .spend
            .is_none(),
        "an unclaimed ball is bound to no workspace and has spent nothing"
    );
}

/// A close-blocker onto a ball that has since closed is resolved — the live
/// set is the resolver, exactly as it is for the claim ladder.
#[test]
fn a_gate_onto_a_closed_ball_is_no_gate_at_all() {
    let w = world(
        vec![ball("bl-x", None, vec![blocks("bl-gone", "close")])],
        vec![join("bl-x", JoinState::ReadyStartable, None, None)],
        vec![],
    );
    let row = w.board().rows.into_iter().next().unwrap();
    assert!(row.gates.is_empty());
    assert_eq!(row.column, Column::Ready);
}

/// Rows that name no live ball — delivered, unassigned-workspace,
/// orphaned-project — leave the board rather than arriving with a made-up
/// status, and the order is deterministic.
#[test]
fn the_board_is_the_join_filtered_to_live_balls_and_ordered() {
    let mut low = ball("bl-aaa", None, vec![]);
    low.priority = 1;
    let w = world(
        vec![low, ball("bl-zzz", None, vec![])],
        vec![
            join("bl-zzz", JoinState::ReadyStartable, None, None),
            join("bl-aaa", JoinState::ReadyStartable, None, None),
            join("bl-dead", JoinState::Delivered, Some(WS_A), Some("alfa")),
            JoinRow {
                project: PathBuf::new(),
                ball_id: String::new(),
                state: JoinState::UnassignedWorkspace,
                workspace: Some(PathBuf::from(WS_A)),
                claimant: None,
                title: None,
            },
        ],
        vec![],
    );
    let ids: Vec<String> = w.board().rows.into_iter().map(|r| r.id).collect();
    assert_eq!(ids, vec!["bl-aaa", "bl-zzz"], "priority first, then id");
}

/// The board is pure over the snapshot: an empty one answers its own empty
/// state, and a workspace with no derived tree contributes no drones.
#[test]
fn an_empty_world_is_an_empty_board() {
    let snap = Snapshot::empty(Instant::now());
    assert_eq!(build(&snap, &ui_doc("{}"), NOW), Board::default());
    assert_eq!(
        super::stamped_roots(&HashMap::new(), Path::new(WS_A), "bl-x"),
        Vec::<String>::new()
    );
}

/// A parent cycle terminates: a ball is visited at most once.
#[test]
fn a_parent_cycle_terminates() {
    let mut a = ball("bl-a", None, vec![]);
    a.parent = Some("bl-b".to_owned());
    let mut b = ball("bl-b", None, vec![]);
    b.parent = Some("bl-a".to_owned());
    let by_id = HashMap::from([("bl-a", &a), ("bl-b", &b)]);
    assert_eq!(descendants("bl-a", &by_id), vec!["bl-a", "bl-b"]);
}
