//! STORIES **S1-T2** restart-equivalence (I0): two `AppModel`s built over the
//! same fixture disk derive identical view-models — "restart is re-read; nothing to
//! restore, nothing to resume" and "a second instance alongside converges
//! identically" (STORIES S1.1, DESIGN §2 I0/I1, §15 M6 Z7). Pure derivation over
//! frozen disk; the only spawns are the read-only `git` calls behind
//! `GitTree::from_repo`.

#![allow(clippy::unwrap_used)]

use crate::support::build_workspace;
use balls::layout::Xdg;
use std::sync::Arc;
use tempfile::tempdir;
use yog::cli_outbound::Cli;
use yog::projects::runner::BlStore;
use yog::ui_state::SystemClock;
use yog::{AppModel, Roots};

/// STORIES **S1-T2** restart-equivalence.
#[test]
fn s1_t2_two_appmodels_over_one_disk_derive_identical_view_models() {
    let root = tempdir().unwrap();
    let roots = Roots {
        yog_data: root.path().join("yog"),
        lernie_data: root.path().join("lernie"),
        yog_state: root.path().join("state"),
        // A non-existent clones root ⇒ no projects ⇒ the injected `bl` runner is
        // never consulted (no live spawn), like the `AppModel` unit harness.
        balls_clones: root.path().join("balls/clones"),
        home: root.path().join("home"),
        world: yog::world::compose(&yog::xdg::Env::from_env()),
    };
    // One ad-hoc workspace on disk, under the lernie workspaces root (§3.1).
    let ws = roots.lernie_data.join("workspaces").join("alpha");
    std::fs::create_dir_all(&ws).unwrap();
    build_workspace(&ws);

    let build = || {
        AppModel::boot(
            roots.clone(),
            None,
            Arc::new(SystemClock),
            Box::new(BlStore::new(
                Xdg::with(root.path(), None, None),
                Cli::new("bl"),
            )),
            Some("me".to_owned()),
        )
        .0
    };
    // Two independent instances — the relaunch (I1) and the second-instance
    // convergence (I0) are the same derivation over the same ground truth.
    let first = build();
    let second = build();

    // The fixture is live: the workspace derived one agent (else the equivalence
    // would be vacuous).
    let tree = first.tree(&ws).unwrap();
    assert_eq!(tree.agents.len(), 1, "the fixture's single agent");

    // Restart-equivalence: identical per-workspace snapshots AND identical §11
    // altitude-0 view-models — no stored state, so re-reading disk yields the
    // same tab bar and the same conversation list (age injected, so it can't
    // diverge by wall clock).
    assert_eq!(first.tree(&ws), second.tree(&ws), "snapshots diverged");
    assert_eq!(
        crate::support::tab_bar(&first),
        crate::support::tab_bar(&second),
        "tab bars diverged across a restart"
    );
    let (mut a, mut b) = (first, second);
    a.focus_workspace(&yog::naming::leaf(&ws));
    b.focus_workspace(&yog::naming::leaf(&ws));
    assert_eq!(
        crate::support::conversation_rows(&a, 1000),
        crate::support::conversation_rows(&b, 1000),
        "conversation lists diverged across a restart"
    );
}
