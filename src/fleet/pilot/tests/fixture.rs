//! The board one tick is asked to decide over: the armed workspace's facts, a
//! row in a column, and the snapshot whose one tree carries the agents a reap
//! compares liveness against. Split from the decision tables at §12's budget on
//! the seam between *the world a tick reads* and *what it decides in it* —
//! every table in this corpus builds the same three, so they have one home.

use super::super::BoardRow;
use crate::app::Snapshot;
use crate::board::Column;
use crate::fleet::Facts;
use crate::git_tree::GitTree;
use crate::projects::join::JoinState;
use std::path::{Path, PathBuf};
use std::time::Duration;

pub(super) const WS: &str = "/names/otter";
pub(super) const PROJECT: &str = "/dev/yog";
pub(super) const NOW: i64 = 1_000_000;

pub(super) fn facts(cap: usize, count: usize, lease: Option<Duration>) -> Facts {
    Facts {
        workspace: crate::naming::leaf(Path::new(WS)),
        project: crate::naming::leaf(Path::new(PROJECT)),
        cap,
        count,
        tick: Duration::from_secs(15),
        lease,
        since_act: None,
        ceiling: None,
    }
}

pub(super) fn row(id: &str, column: Column, drones: Vec<&str>) -> BoardRow {
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
pub(super) fn snap(agents: Vec<crate::git_tree::Agent>) -> Snapshot {
    let mut snap = Snapshot::empty(0);
    // The armed entry names a clone directory and a board row names the
    // project (bl-b4b5), so the naming set has to hold it for the two to be
    // put into one vocabulary — which is what an armed world always is.
    snap.projects = vec![PathBuf::from(PROJECT)];
    // The facts name the workspace and the pilot needs the directory, so the
    // arming table it resolves through has to hold the entry (bl-ef16) — which
    // is what an armed world always is: `facts::of` folds these very keys.
    snap.fleet.insert(
        WS.to_owned(),
        crate::fleet::Policy {
            project: PathBuf::from(PROJECT),
            cap: 1,
            lease: None,
        },
    );
    snap.trees.insert(
        PathBuf::from(WS),
        GitTree {
            commits: vec![],
            agents,
        },
    );
    snap
}
