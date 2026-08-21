//! What one tick decides, and the one thing only a real thread can prove.
//!
//! The decision ([`plan`]) is pure over a published snapshot and the board built
//! from it, so it is a table; the burden check is the first table here, because
//! it is the rung's own condition and must be mechanical rather than promised.
//!
//! What one tick *does* — the thread, the two fires, the rows — is [`fire`],
//! split at §12's cap on the seam between deciding and doing; [`stillbirth`] is
//! the other decision table, cut off at the same cap on the seam the plan draws
//! — a lease compares a drone's idleness, a stillbirth compares the loop's own
//! birth record against a world that grew no conversation from it (bl-ab13).

use super::*;
use crate::app::Snapshot;
use crate::board::Column;
use crate::boundary::tests::agent;
use crate::git_tree::GitTree;
use crate::projects::join::JoinState;
use std::path::Path;

pub(super) const WS: &str = "/names/otter";
pub(super) const PROJECT: &str = "/dev/yog";
const NOW: i64 = 1_000_000;

fn facts(cap: usize, count: usize, lease: Option<Duration>) -> Facts {
    Facts {
        workspace: PathBuf::from(WS),
        project: PathBuf::from(PROJECT),
        cap,
        count,
        tick: Duration::from_secs(15),
        lease,
        since_act: None,
        ceiling: None,
    }
}

fn row(id: &str, column: Column, drones: Vec<&str>) -> BoardRow {
    let mine = column == Column::Claimed;
    BoardRow {
        project: crate::naming::leaf(Path::new(PROJECT)),
        id: id.to_owned(),
        title: format!("title of {id}"),
        priority: 0,
        column,
        state: if mine {
            JoinState::Bound
        } else {
            JoinState::ReadyStartable
        },
        workspace: mine.then(|| crate::naming::leaf(Path::new(WS))),
        claimant: mine.then(|| "otter".to_owned()),
        parent: None,
        gates: vec![],
        drones: drones
            .into_iter()
            .map(|root| crate::board::Drone {
                root_id: root.to_owned(),
                name: root.to_owned(),
            })
            .collect(),
        spend: None,
        rollup: None,
    }
}

/// A snapshot whose one workspace tree holds `agents` — the liveness a reap
/// compares against.
fn snap(agents: Vec<crate::git_tree::Agent>) -> Snapshot {
    let mut snap = Snapshot::empty(0);
    // The armed entry names a clone directory and a board row names the
    // project (bl-b4b5), so the naming set has to hold it for the two to be
    // put into one vocabulary — which is what an armed world always is.
    snap.projects = vec![PathBuf::from(PROJECT)];
    snap.trees.insert(
        PathBuf::from(WS),
        GitTree {
            commits: vec![],
            agents,
        },
    );
    snap
}

#[test]
fn a_workspace_under_its_cap_takes_the_top_ready_ball() {
    let rows = vec![
        row("bl-2", Column::Ready, vec![]),
        row("bl-9", Column::Ready, vec![]),
    ];
    let one = plan(&snap(vec![]), &facts(2, 1, None), &rows, NOW);
    assert_eq!(
        one,
        Some(Move::Spawn {
            row: rows[0].clone()
        }),
        "board order is the pick — there is no second ranking"
    );
}

#[test]
fn a_full_workspace_a_gated_ball_and_a_bound_ceiling_all_spawn_nothing() {
    let ready = vec![row("bl-2", Column::Ready, vec![])];
    assert_eq!(
        plan(&snap(vec![]), &facts(1, 1, None), &ready, NOW),
        None,
        "at the cap"
    );
    let gated = vec![row("bl-2", Column::Gated, vec![])];
    assert_eq!(
        plan(&snap(vec![]), &facts(3, 0, None), &gated, NOW),
        None,
        "a gated ball can be started but not delivered, so the loop leaves it"
    );
    let mut bound = facts(3, 0, None);
    bound.ceiling = Some("spend ceiling reached".to_owned());
    assert_eq!(
        plan(&snap(vec![]), &bound, &ready, NOW),
        None,
        "the ceiling binds on the next spawn, so there is no next spawn"
    );
}

#[test]
fn a_ready_ball_in_another_project_is_not_this_loops_work() {
    let mut elsewhere = row("bl-2", Column::Ready, vec![]);
    elsewhere.project = "lernie".to_owned();
    assert_eq!(
        plan(&snap(vec![]), &facts(3, 0, None), &[elsewhere], NOW),
        None
    );
}

#[test]
fn a_quiet_ball_past_its_lease_is_reaped_with_the_comparison_as_its_reason() {
    let quiet = agent("root-1", crate::git_tree::AgentState::Quiescent, NOW - 2820);
    let rows = vec![row("bl-1", Column::Claimed, vec!["root-1"])];
    let one = plan(
        &snap(vec![quiet]),
        &facts(3, 1, Some(Duration::from_mins(30))),
        &rows,
        NOW,
    );
    let Some(Move::Reap {
        row,
        claimant,
        since,
    }) = one
    else {
        panic!("a reap");
    };
    assert_eq!(row.id, "bl-1");
    assert_eq!(claimant, "otter", "released from whoever the row names");
    assert_eq!(
        since, "lease expired 17m ago",
        "the reason is the comparison itself, never a diagnosis"
    );
}

#[test]
fn a_running_drone_is_never_reaped_however_old_its_claim() {
    for state in [
        crate::git_tree::AgentState::Live,
        crate::git_tree::AgentState::InFlight,
    ] {
        let busy = agent("root-1", state, NOW - 999_999);
        let rows = vec![row("bl-1", Column::Claimed, vec!["root-1"])];
        assert_eq!(
            plan(
                &snap(vec![busy]),
                &facts(1, 1, Some(Duration::from_mins(1))),
                &rows,
                NOW
            ),
            None,
            "{state:?}: killing mid-ball destroys uncommitted work"
        );
    }
}

#[test]
fn no_lease_reaps_nothing_and_a_ball_still_inside_its_lease_is_left_alone() {
    let quiet = agent("root-1", crate::git_tree::AgentState::Stopped, NOW - 60);
    let rows = vec![row("bl-1", Column::Claimed, vec!["root-1"])];
    assert_eq!(
        plan(&snap(vec![quiet.clone()]), &facts(1, 1, None), &rows, NOW),
        None,
        "an absent lease reaps nothing rather than reaping on a default"
    );
    assert_eq!(
        plan(
            &snap(vec![quiet]),
            &facts(1, 1, Some(Duration::from_mins(30))),
            &rows,
            NOW
        ),
        None,
        "still inside the lease"
    );
}

/// A row that names nobody cannot be released from anyone, so it is not a
/// reap — and yog does not guess whose claim it was.
#[test]
fn a_row_that_names_no_claimant_is_not_reapable() {
    let quiet = agent("root-1", crate::git_tree::AgentState::Stopped, NOW - 3600);
    let anonymous = BoardRow {
        claimant: None,
        ..row("bl-1", Column::Claimed, vec!["root-1"])
    };
    assert_eq!(
        plan(
            &snap(vec![quiet]),
            &facts(3, 1, Some(Duration::from_mins(1))),
            &[anonymous],
            NOW
        ),
        None
    );
}

#[test]
fn a_claim_with_no_conversation_and_no_birth_row_is_not_reapable_and_reaps_go_first() {
    let droneless = vec![row("bl-1", Column::Claimed, vec![])];
    assert_eq!(
        plan(
            &snap(vec![]),
            &facts(3, 1, Some(Duration::from_secs(1))),
            &droneless,
            NOW
        ),
        None,
        "no conversation and no birth of the loop's own: nothing to compare, \
         and yog does not guess whose claim it was"
    );
    // With one reapable ball and one ready ball, the reap is the tick's move.
    let quiet = agent("root-1", crate::git_tree::AgentState::Stopped, NOW - 600);
    let both = vec![
        row("bl-1", Column::Claimed, vec!["root-1"]),
        row("bl-2", Column::Ready, vec![]),
    ];
    let one = plan(
        &snap(vec![quiet]),
        &facts(9, 1, Some(Duration::from_mins(1))),
        &both,
        NOW,
    );
    assert!(
        matches!(one, Some(Move::Reap { .. })),
        "freeing a slot goes before filling one"
    );
}

/// A workspace the derivation never reached has no tree, so nothing about its
/// claims is knowable — and an unknowable comparison is not a reap.
#[test]
fn a_workspace_with_no_derived_tree_reaps_nothing() {
    let mut none = Snapshot::empty(0);
    none.trees.clear();
    let rows = vec![row("bl-1", Column::Claimed, vec!["root-1"])];
    assert_eq!(
        plan(
            &none,
            &facts(1, 1, Some(Duration::from_secs(1))),
            &rows,
            NOW
        ),
        None
    );
}

mod fire;
mod stillbirth;
