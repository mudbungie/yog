//! The V4 board fixture — cut out of [`listings`](super::listings) on the very
//! seam `reply/board` itself is cut on: the board is the one listing whose rows
//! carry derived sub-objects of their own (gates, drones, two §3.5 figures) and
//! whose reply carries a second list beside the rows.

use std::time::Duration;

use super::spend;
use crate::board::{Board, BoardRow, Column, Drone, Gate};
use crate::projects::join::JoinState;
use crate::spend::{Attribution, Cost, Figure};

/// The V4 board: a gated row carrying every derived sub-object, a bare ready
/// row carrying none, and one armed loop's facts beside them.
pub(super) fn board() -> Board {
    let priced = Figure {
        tokens: spend(),
        cost: Some(Cost {
            micro_usd: 1_500_000,
            unpriced_tokens: 4,
        }),
        attribution: Attribution::Workspace,
    };
    let unpriced = Figure {
        tokens: spend(),
        cost: None,
        attribution: Attribution::Conversations(1),
    };
    Board {
        rows: vec![
            BoardRow {
                project: "p".into(),
                id: "bl-1".into(),
                title: "t".into(),
                priority: 2,
                column: Column::Gated,
                state: JoinState::Bound,
                workspace: Some("ws".into()),
                claimant: Some("alba".into()),
                parent: Some("bl-epic".into()),
                gates: vec![Gate {
                    id: "bl-gate".into(),
                    title: "g".into(),
                }],
                drones: vec![Drone {
                    root_id: "c-1".into(),
                    name: "Cobalt".into(),
                }],
                spend: Some(priced),
                rollup: Some(unpriced),
            },
            BoardRow {
                project: "p".into(),
                id: "bl-2".into(),
                title: "u".into(),
                priority: 0,
                column: Column::Ready,
                state: JoinState::ReadyStartable,
                workspace: None,
                claimant: None,
                parent: None,
                gates: vec![],
                drones: vec![],
                spend: None,
                rollup: None,
            },
        ],
        fleet: vec![crate::fleet::Facts {
            workspace: "ws".into(),
            project: "p".into(),
            cap: 4,
            count: 1,
            tick: Duration::from_secs(90),
            lease: Some(Duration::from_mins(10)),
            since_act: Some(30),
            ceiling: Some("over budget".into()),
        }],
    }
}
