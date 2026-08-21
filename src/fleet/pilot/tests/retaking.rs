//! **The loop does not take back a ball it gave back** (bl-3988, DESIGN §11):
//! the third decision table, cut off [`super`] at §12's cap on the seam the
//! plan draws — the two reap tables say when a claim comes back, and this one
//! says what the loop may take next.

use super::*;
use crate::opslog::OpRow;

/// One `["yog-fleet",<verb>,…]` row in [`WS`], as the loop leaves it.
fn act(verb: &str, ball: &str) -> OpRow {
    let entry = if verb == crate::fleet::row::REAP {
        crate::fleet::row::reaped(
            "1".to_owned(),
            Path::new(WS),
            ball,
            "otter",
            "lease expired 1m ago",
        )
    } else {
        crate::fleet::row::spawned("1".to_owned(), Path::new(WS), ball, "OtterBrook")
    };
    OpRow::from(&entry)
}

/// A world as [`snap`] makes one, carrying `ops` as its trail.
fn with(ops: Vec<OpRow>) -> Snapshot {
    Snapshot {
        ops,
        ..snap(vec![])
    }
}

/// The storm, gone: a ball the loop reaped is not the ball the next tick takes,
/// and the lower-priority work that was starved behind it runs.
#[test]
fn a_reaped_ball_is_not_retaken_and_the_next_ready_ball_runs() {
    let board = vec![
        row("bl-high", Column::Ready, vec![]),
        row("bl-low", Column::Ready, vec![]),
    ];
    let one = plan(
        &with(vec![act(crate::fleet::row::REAP, "bl-high")]),
        &facts(1, 0, None),
        &board,
        NOW,
    );
    let Some(Move::Spawn { row }) = one else {
        panic!("a spawn");
    };
    assert_eq!(
        row.id, "bl-low",
        "the board's first ready ball is the one the loop just gave back"
    );
}

/// With nothing else ready, a ball the loop gave back is simply not taken —
/// the loop stops rather than betting the same fire has a different outcome.
#[test]
fn a_reaped_ball_alone_on_the_board_stops_the_loop_rather_than_looping() {
    let board = vec![row("bl-high", Column::Ready, vec![])];
    assert_eq!(
        plan(
            &with(vec![act(crate::fleet::row::REAP, "bl-high")]),
            &facts(1, 0, None),
            &board,
            NOW
        ),
        None
    );
}

/// Only the **newest** act decides, and only this workspace's: a ball the loop
/// took after giving it back is takeable again, and another workspace's reap is
/// not this loop's memory.
#[test]
fn the_newest_act_in_this_workspace_is_the_whole_memory() {
    let board = vec![row("bl-high", Column::Ready, vec![])];
    let retaken = vec![
        act(crate::fleet::row::REAP, "bl-high"),
        act(crate::fleet::row::SPAWN, "bl-high"),
    ];
    assert!(
        matches!(
            plan(&with(retaken), &facts(1, 0, None), &board, NOW),
            Some(Move::Spawn { .. })
        ),
        "the newest act is a spawn, so the loop holds it rather than having given it back"
    );
    let mut elsewhere = act(crate::fleet::row::REAP, "bl-high");
    elsewhere.cwd = "/names/heron".to_owned();
    assert!(
        matches!(
            plan(&with(vec![elsewhere]), &facts(1, 0, None), &board, NOW),
            Some(Move::Spawn { .. })
        ),
        "another workspace's loop gave that ball back, not this one"
    );
}
