//! The V4 board reply's spelling (§8.5, STORIES S13) — cut from the sibling
//! table at the §12 budget, at its own seam: the board is the one reply whose
//! rows carry derived sub-objects (gates, drones, figures).

use super::super::*;
use crate::projects::join::JoinState;
use std::path::PathBuf;

/// The V4 board's spelling: the column leads, the binding state rides beside
/// it, and every optional field is present or absent rather than nulled — both
/// directions, so a reader never has to guess which shape it got.
#[test]
fn board_rows_encode_the_column_the_gate_the_drones_and_the_figures() {
    use crate::board::{Board, BoardRow, Column, Drone, Gate};
    use crate::budgets::BudgetSpend;
    use crate::spend::{Attribution, Cost, Figure};

    let figure = |cost, attribution| Figure {
        tokens: BudgetSpend {
            input_tokens: 100,
            ..BudgetSpend::default()
        },
        cost,
        attribution,
    };
    let full = BoardRow {
        project: PathBuf::from("/p"),
        id: "bl-1".into(),
        title: "t".into(),
        priority: 2,
        column: Column::Gated,
        state: JoinState::Bound,
        workspace: Some(PathBuf::from("/ws")),
        claimant: Some("alba".into()),
        parent: Some("bl-epic".into()),
        gates: vec![Gate {
            id: "bl-gate".into(),
            title: "the gate".into(),
        }],
        drones: vec![Drone {
            root_id: "conv1".into(),
            name: "Cobalt".into(),
        }],
        spend: Some(figure(
            Some(Cost {
                micro_usd: 1_500_000,
                unpriced_tokens: 4,
            }),
            Attribution::Workspace,
        )),
        rollup: Some(figure(None, Attribution::Conversations(1))),
    };
    let bare = BoardRow {
        project: PathBuf::from("/p"),
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
    };
    let v = encode(&Reply::Board(Board {
        rows: vec![full, bare],
        fleet: vec![],
    }));
    let rows = v["rows"].as_array().unwrap();
    assert_eq!(v["kind"], "board");
    assert!(
        v.get("fleet").is_none(),
        "an unarmed world answers no fleet key at all — V4's burden check on the wire"
    );
    assert_eq!(rows[0]["column"], "gated");
    assert_eq!(rows[0]["state"], "bound");
    assert_eq!(rows[0]["workspace"], "/ws");
    assert_eq!(rows[0]["claimant"], "alba");
    assert_eq!(rows[0]["parent"], "bl-epic");
    assert_eq!(rows[0]["gates"][0]["id"], "bl-gate");
    assert_eq!(rows[0]["gates"][0]["mints"], "close");
    assert_eq!(rows[0]["drones"][0]["name"], "Cobalt");
    assert_eq!(rows[0]["spend"]["usd"], "$1.50");
    assert_eq!(rows[0]["spend"]["micro_usd"], 1_500_000);
    assert_eq!(rows[0]["spend"]["unpriced_tokens"], 4);
    assert_eq!(rows[0]["spend"]["attribution"], "workspace-wide");
    assert_eq!(rows[0]["rollup"]["tokens"], 100);
    assert!(
        rows[0]["rollup"].get("usd").is_none(),
        "an unpriced world renders tokens and no money at all"
    );
    assert!(
        rows[0]["rollup"].get("attribution").is_none(),
        "one stamped conversation is exactly what the seat already claims"
    );
    assert_eq!(rows[1]["column"], "ready");
    for absent in ["workspace", "claimant", "parent", "spend", "rollup"] {
        assert!(rows[1].get(absent).is_none(), "{absent} must be absent");
    }
    assert!(rows[1]["gates"].as_array().unwrap().is_empty());
    assert!(rows[1]["drones"].as_array().unwrap().is_empty());
}

/// The armed loop's facts on the wire (VISION §5 V4 item 2): every one derived,
/// the two `None`s absent rather than zeroed, and the label the seats render.
#[test]
fn an_armed_loops_facts_encode_beside_the_rows() {
    let facts = crate::fleet::Facts {
        workspace: std::path::PathBuf::from("/ws"),
        project: std::path::PathBuf::from("/proj"),
        cap: 3,
        count: 1,
        tick: std::time::Duration::from_secs(15),
        lease: Some(std::time::Duration::from_mins(30)),
        since_act: Some(240),
        ceiling: Some("spend ceiling reached".to_owned()),
    };
    let v = encode(&Reply::Board(Board {
        rows: vec![],
        fleet: vec![facts.clone()],
    }));
    let one = &v["fleet"][0];
    assert_eq!(one["workspace"], "/ws");
    assert_eq!(one["project"], "/proj");
    assert_eq!(one["cap"], 3);
    assert_eq!(one["count"], 1);
    assert_eq!(one["tick_secs"], 15);
    assert_eq!(one["lease_secs"], 1800);
    assert_eq!(one["last_act_secs_ago"], 240);
    assert_eq!(one["ceiling"], "spend ceiling reached");
    assert_eq!(
        one["room"], false,
        "the ceiling binds the next spawn, so there is no room"
    );
    assert_eq!(one["label"], facts.label());

    // A loop that has never acted, with no lease and no ceiling, says the two
    // absences by omitting them — `0` would be a different fact.
    let fresh = crate::fleet::Facts {
        lease: None,
        since_act: None,
        ceiling: None,
        ..facts
    };
    let v = encode(&Reply::Board(Board {
        rows: vec![],
        fleet: vec![fresh],
    }));
    for absent in ["lease_secs", "last_act_secs_ago", "ceiling"] {
        assert!(v["fleet"][0].get(absent).is_none(), "{absent}");
    }
    assert_eq!(v["fleet"][0]["room"], true);
}
