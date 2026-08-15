//! Every fact is a query, and the burden check is the first of these tables:
//! an unarmed world derives no loop at all.

use super::*;
use crate::app::Snapshot;
use crate::budgets::{BudgetSpend, StepBill};
use crate::opslog::OpRow;
use crate::projects::join::JoinState;
use crate::spend::Prices;
use serde_json::json;
use std::path::Path;

const WS: &str = "/names/otter";
const PROJECT: &str = "/dev/yog";
const NOW: i64 = 1_000_000;

fn snap(fleet: Vec<(&str, super::super::Policy)>) -> Snapshot {
    let mut snap = Snapshot::empty(0);
    snap.fleet = fleet.into_iter().map(|(k, p)| (k.to_owned(), p)).collect();
    snap
}

fn policy(cap: usize, lease: Option<Duration>) -> super::super::Policy {
    super::super::Policy {
        project: PathBuf::from(PROJECT),
        cap,
        lease,
    }
}

fn row(id: &str, column: Column, workspace: Option<&str>) -> BoardRow {
    BoardRow {
        project: crate::naming::leaf(Path::new(PROJECT)),
        id: id.to_owned(),
        title: format!("title of {id}"),
        priority: 0,
        column,
        state: JoinState::Bound,
        workspace: workspace.map(|w| crate::naming::leaf(Path::new(w))),
        claimant: workspace.map(|_| "otter".to_owned()),
        parent: None,
        gates: vec![],
        drones: vec![],
        spend: None,
        rollup: None,
    }
}

#[test]
fn an_unarmed_world_derives_no_loop_at_all() {
    let snap = snap(vec![]);
    let rows = vec![row("bl-1", Column::Claimed, Some(WS))];
    assert!(
        of(&snap, &Prices::default(), Ceiling::default(), &rows, NOW).is_empty(),
        "no entry, no facts — the board is exactly today's balls section"
    );
}

#[test]
fn the_count_is_the_boards_own_claimed_rows_for_this_workspace() {
    let snap = snap(vec![(WS, policy(3, None))]);
    let rows = vec![
        row("bl-1", Column::Claimed, Some(WS)),
        row("bl-2", Column::Claimed, Some(WS)),
        // Another workspace's claim, a ready ball, and a blocked one: none of
        // them is a drone here.
        row("bl-3", Column::Claimed, Some("/names/other")),
        row("bl-4", Column::Ready, None),
        row("bl-5", Column::Blocked, None),
    ];
    let facts = of(&snap, &Prices::default(), Ceiling::default(), &rows, NOW);
    let one = facts.first().expect("one armed workspace");
    assert_eq!(one.count, 2);
    assert_eq!(one.cap, 3);
    assert_eq!(one.project, PathBuf::from(PROJECT));
    assert!(one.has_room(), "under the cap and ungated");
    assert_eq!(one.since_act, None, "it has never acted");
    assert_eq!(one.tick, snap.cadence.full_sweep, "the clock's own period");
}

#[test]
fn a_full_workspace_has_no_room_and_says_so_in_one_line() {
    let snap = snap(vec![(WS, policy(1, Some(Duration::from_mins(30))))]);
    let rows = vec![row("bl-1", Column::Claimed, Some(WS))];
    let facts = of(&snap, &Prices::default(), Ceiling::default(), &rows, NOW);
    let one = facts.first().expect("armed");
    assert!(!one.has_room());
    let label = one.label();
    assert!(label.contains("1/1 drones"), "{label}");
    assert!(label.contains("lease 30m"), "{label}");
    assert!(label.contains("nothing yet"), "{label}");
}

#[test]
fn the_last_tick_is_the_newest_row_the_loop_left() {
    let mut snap = snap(vec![(WS, policy(2, None))]);
    let line =
        super::super::row::spawned((NOW - 240).to_string(), Path::new(WS), "bl-1", "otter-one");
    snap.ops = vec![OpRow::from(&line)];
    let facts = of(&snap, &Prices::default(), Ceiling::default(), &[], NOW);
    let one = facts.first().expect("armed");
    assert_eq!(one.since_act, Some(240));
    assert!(one.label().contains("last 4m ago"), "{}", one.label());
}

#[test]
fn the_ceiling_renders_where_it_will_bind_and_closes_the_room() {
    let mut snap = snap(vec![(WS, policy(4, None))]);
    snap.bills.insert(
        PathBuf::from(WS),
        vec![StepBill {
            conv: "otter-one".to_owned(),
            seq: "001".to_owned(),
            model: Some("opus".to_owned()),
            spend: BudgetSpend {
                input_tokens: 3_000_000,
                ..BudgetSpend::default()
            },
            last_usage: BudgetSpend::default(),
        }],
    );
    let prices = Prices::from_json(&json!({ "opus": { "input": 1 } }));
    let ceiling = Ceiling::from_json(Some(&json!(2)));
    let facts = of(&snap, &prices, ceiling, &[], NOW);
    let one = facts.first().expect("armed");
    let refusal = one.ceiling.as_deref().expect("the next spawn would bind");
    assert!(refusal.contains("spend ceiling reached"), "{refusal}");
    assert!(
        !one.has_room(),
        "an empty workspace still has no room when the next birth would be refused"
    );
}
