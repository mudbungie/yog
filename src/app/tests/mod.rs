//! Shared harness + the construction / enumeration / accessor tests for the
//! multi-workspace [`AppModel`] (§15 Y11). The tick/sweep machinery is in
//! `derive`.
//!
//! What a seat is *looking at* is no longer here to test (bl-7942): the focus,
//! its startup derivation and the optimistic echo folded over it were the
//! window's, and a server holds none of them (REMOTE §7).

mod delete_exec;
pub(super) mod derive;
mod drift;
pub(crate) mod harness;
mod worker;

use super::*;
use crate::binding::WorkspaceKind;
use crate::fs_watcher::RootKind;
use crate::test_support::FakeClock;
pub(super) use harness::Harness;
pub(crate) use harness::Rig;
pub(crate) use harness::agent;
use harness::no_balls;
use tempfile::tempdir;

#[test]
fn new_enumerates_and_snapshots_every_workspace() {
    let h = Harness::new();
    let (_c, model) = h.model();
    assert_eq!(model.workspaces().len(), 1, "the one ad-hoc workspace");
    let tree = model.tree(&h.ws).expect("snapshot derived at construction");
    assert_eq!(tree.agents.len(), 1);
    assert_eq!(tree.agents[0].agent_id, "c-1");
}

#[test]
fn replays_enumerate_into_the_tab_overflow_and_render_read_only() {
    let mut h = Harness::new();
    let replay = h.add_replay("99XYZ", "r-1");
    let (_c, model) = h.model();
    // Enumerated across the three roots and classified Replay by its root alone.
    assert!(
        model
            .workspaces()
            .iter()
            .any(|w| w.path == replay && w.kind == WorkspaceKind::Replay),
        "the replay is enumerated"
    );
    // "Replay is not a mode": it lands in the tab bar's overflow, flagged, and
    // the answered row is where a seat reads that from.
    let bar = model.tab_bar(Some(&crate::naming::leaf(&h.ws)));
    assert!(
        bar.overflow
            .iter()
            .any(|t| t.name == crate::naming::leaf(&replay)
                && t.kind == crate::nav::tabs::Kind::Replay),
        "overflow carries the replay: {bar:?}"
    );
}

#[test]
fn an_empty_world_answers_every_read_with_nothing() {
    // No workspaces at all — the general empty path, not a case of its own.
    let root = tempdir().unwrap();
    let roots = Roots {
        yog_data: root.path().join("yog"),
        litany_data: root.path().join("litany"),
        yog_state: root.path().join("state"),
        balls_clones: root.path().join("balls").join("clones"),
        home: root.path().join("home"),
        world: crate::test_support::no_world(),
    };
    let clock = FakeClock::new();
    let (model, _deriver) = AppModel::boot(roots, clock.arc(), Box::new(no_balls()), None);
    assert!(model.workspaces().is_empty());
    assert!(model.ws_listing().rows.is_empty(), "nothing to enumerate");
    let deps = model.boundary_deps(
        &crate::cli_outbound::Cli::new("litany"),
        &crate::cli_outbound::Cli::new("bl"),
    );
    assert!(
        model
            .answer(
                &deps,
                &crate::boundary::Query::Conversations {
                    workspace: "ws".to_owned(),
                },
                10,
            )
            .is_err(),
        "a name nothing answers is a refusal, which is a seat's no rows"
    );
    assert!(model.workspace_path("ws").is_none());
}

#[test]
fn desired_watches_covers_enum_roots_and_every_workspace() {
    let h = Harness::new();
    let watches = desired_watches(
        &h.roots,
        &[Workspace {
            path: h.ws.clone(),
            kind: WorkspaceKind::Foreign,
        }],
    );
    assert!(watches.contains(&(h.roots.names(), RootKind::NamesRoot)));
    assert!(watches.contains(&(h.roots.workspaces(), RootKind::WorkspacesRoot)));
    assert!(watches.contains(&(h.roots.replays(), RootKind::WorkspacesRoot)));
    assert!(watches.contains(&(h.roots.yog_state.clone(), RootKind::YogState)));
    assert!(watches.contains(&(h.ws.clone(), RootKind::Workspace)));
}

#[test]
fn construction_arms_the_workspace_and_enum_watchers() {
    let h = Harness::new();
    let (_c, model) = h.model();
    let handle = model.deriver.watchset_handle();
    let ws = crate::state::lock_watchset(&handle);
    assert!(ws.watches(&h.ws, RootKind::Workspace), "workspace armed");
    assert!(ws.watches(&h.roots.workspaces(), RootKind::WorkspacesRoot));
    assert!(ws.watches(&h.roots.yog_state, RootKind::YogState));
}

#[test]
fn idle_tick_changes_nothing() {
    let h = Harness::new();
    let (_c, mut model) = h.model();
    assert!(!model.tick(), "no dirt, no due sweep → no change");
}
