//! The V4 board reply's **decoders** (§8.5, REMOTE §9 step 2, bl-7067) — one
//! per encoder in [`super`], undoing the row, the armed loop's facts and the
//! §3.5 figure the row carries twice.

use serde_json::Value;
use std::time::Duration;

use crate::board::{Board, BoardRow, Column, Drone, Gate};
use crate::boundary::codec::fields::{
    i64_of, list_of, opt, opt_str_of, opt_val, pick, str_of, u64_of, usize_of,
};
use crate::boundary::codec::parse_join;
use crate::fleet::Facts;
use crate::spend::{Attribution, Cost, Figure};

/// The board's four columns, [`Column::word`]'s other half.
const COLUMNS: [(&str, Column); 4] = [
    ("ready", Column::Ready),
    ("gated", Column::Gated),
    ("claimed", Column::Claimed),
    ("blocked", Column::Blocked),
];

/// The whole board: its rows, and the loop facts that ride only when a §4.3
/// loop is armed. An absent `fleet` is the unarmed world, which is exactly the
/// empty list — the shape the encoder chose so a reader never has to tell "no
/// loop" from "a loop with nothing in it", read back as the same fact.
pub(crate) fn board(obj: &serde_json::Map<String, Value>) -> Result<Board, String> {
    Ok(Board {
        rows: list_of(obj, "rows", board_row)?,
        fleet: match obj.get("fleet") {
            None => Vec::new(),
            Some(_) => list_of(obj, "fleet", fleet_facts)?,
        },
    })
}

fn board_row(v: &Value) -> Result<BoardRow, String> {
    let o = v.as_object().ok_or("board row: not an object")?;
    Ok(BoardRow {
        project: str_of(o, "project")?,
        id: str_of(o, "id")?,
        title: str_of(o, "title")?,
        priority: i64_of(o, "priority")?,
        column: pick(o, "column", &COLUMNS)?,
        state: parse_join(&str_of(o, "state")?)?,
        workspace: opt_str_of(o, "workspace")?,
        claimant: opt_str_of(o, "claimant")?,
        parent: opt_str_of(o, "parent")?,
        gates: list_of(o, "gates", gate)?,
        drones: list_of(o, "drones", drone)?,
        spend: opt_val(o, "spend", figure)?,
        rollup: opt_val(o, "rollup", figure)?,
    })
}

/// One unresolved close-blocker. `mints` is not read back: it is the constant
/// `"close"` the encoder writes, a statement about what minted the gate rather
/// than a field of one.
fn gate(v: &Value) -> Result<Gate, String> {
    let o = v.as_object().ok_or("gate: not an object")?;
    Ok(Gate {
        id: str_of(o, "id")?,
        title: str_of(o, "title")?,
    })
}

fn drone(v: &Value) -> Result<Drone, String> {
    let o = v.as_object().ok_or("drone: not an object")?;
    Ok(Drone {
        root_id: str_of(o, "root_id")?,
        name: str_of(o, "name")?,
    })
}

/// One armed loop's facts. `room` and `label` are derived on the way out and
/// dropped on the way in — [`Facts::has_room`] and [`Facts::label`] are the one
/// authority for both, and reading them back would be a second one.
///
/// The two periods ride as **whole seconds**, which is the granularity the
/// `cadence.yaml` entry they come from is written in; a sub-second `tick` is
/// not a value this surface can carry, and none is ever built.
fn fleet_facts(v: &Value) -> Result<Facts, String> {
    let o = v.as_object().ok_or("fleet facts: not an object")?;
    Ok(Facts {
        workspace: str_of(o, "workspace")?,
        project: str_of(o, "project")?,
        cap: usize_of(o, "cap")?,
        count: usize_of(o, "count")?,
        tick: Duration::from_secs(u64_of(o, "tick_secs")?),
        lease: opt(o, "lease_secs", u64_of)?.map(Duration::from_secs),
        since_act: opt(o, "last_act_secs_ago", i64_of)?,
        ceiling: opt_str_of(o, "ceiling")?,
    })
}

/// One §3.5 figure. `usd` is dropped for the reason `room` is: it is
/// [`Cost::usd`]'s rendering of `micro_usd`, which rides beside it.
pub(crate) fn figure(v: &Value) -> Result<Figure, String> {
    let o = v.as_object().ok_or("figure: not an object")?;
    let cost = match o.get("micro_usd") {
        None => None,
        Some(_) => Some(Cost {
            micro_usd: u64_of(o, "micro_usd")?,
            unpriced_tokens: u64_of(o, "unpriced_tokens")?,
        }),
    };
    Ok(Figure {
        tokens: crate::steps_view::wire::decode::spend(
            o.get("tokens").ok_or("figure: missing tokens")?,
        )?,
        cost,
        attribution: attribution(o.get("attribution").ok_or("figure: missing attribution")?)?,
    })
}

fn attribution(v: &Value) -> Result<Attribution, String> {
    let o = v.as_object().ok_or("attribution: not an object")?;
    match str_of(o, "kind")?.as_str() {
        "conversations" => Ok(Attribution::Conversations(usize_of(o, "count")?)),
        "workspace" => Ok(Attribution::Workspace),
        other => Err(format!("attribution: unknown kind {other:?}")),
    }
}
