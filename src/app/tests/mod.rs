//! Shared harness + the construction / enumeration / accessor tests for the
//! multi-workspace [`AppModel`] (§15 Y11). The tick/sweep machinery is in
//! `derive`, the roster/attention/seen surface (and the M2 convergence proof)
//! in `focus`.

mod agent_deletes;
mod attention;
mod deletes;
pub(super) mod derive;
mod drift;
mod focus;
mod harness;
mod knobs;
mod panels;
mod search;
mod spend;
mod started;
mod view;
mod worker;

use super::*;
use crate::binding::WorkspaceKind;
use crate::fs_watcher::RootKind;
use crate::git_tree::{AgentState, GitTree};
use crate::test_support::FakeClock;
pub(super) use harness::Harness;
pub(crate) use harness::Rig;
pub(crate) use harness::agent;
use harness::no_balls;
use std::path::PathBuf;
use tempfile::tempdir;

#[test]
fn args_workspace_is_optional_and_parses() {
    assert_eq!(Args::try_parse_from(["yog"]).unwrap().workspace, None);
    let a = Args::try_parse_from(["yog", "--workspace", "/tmp/x"]).unwrap();
    assert_eq!(a.workspace, Some(PathBuf::from("/tmp/x")));
}

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
    let (_c, mut model) = h.model();
    // Enumerated across the three roots and classified Replay by its root alone.
    assert!(
        model
            .workspaces()
            .iter()
            .any(|w| w.path == replay && w.kind == WorkspaceKind::Replay),
        "the replay is enumerated"
    );
    // "Replay is not a mode": it lands in the tab bar's overflow, flagged (§11).
    let bar = model.tab_bar();
    assert!(
        bar.overflow
            .iter()
            .any(|t| t.ws == replay && t.kind == crate::nav::tabs::Kind::Replay),
        "overflow carries the replay: {bar:?}"
    );
    // Focused, it is read-only; the ad-hoc workspace beside it is writable.
    model.focus_workspace(&replay);
    assert!(model.focused_is_replay(), "a focused replay is read-only");
    model.focus_workspace(&h.ws);
    assert!(
        !model.focused_is_replay(),
        "the ad-hoc workspace is writable"
    );
}

#[test]
fn startup_focus_is_the_attention_bearing_workspace() {
    let h = Harness::new();
    let (_c, model) = h.model();
    // The one workspace has an unseen stop → attention → it is the startup focus.
    assert_eq!(model.focused_workspace(), Some(h.ws.as_path()));
    assert!(
        model.focus().agent.is_none(),
        "no agent selected at startup"
    );
    assert_eq!(model.focused_tree().map(|t| t.agents.len()), Some(1));
}

#[test]
fn startup_focus_override_wins() {
    let h = Harness::new();
    let forced = PathBuf::from("/somewhere/else");
    let (_c, model) = h.model_focused(Some(forced.clone()));
    assert_eq!(model.focused_workspace(), Some(forced.as_path()));
}

#[test]
fn startup_focus_falls_back_to_first_when_nothing_needs_attention() {
    // No workspaces at all: focus derives to None (the general empty path).
    let root = tempdir().unwrap();
    let roots = Roots {
        yog_data: root.path().join("yog"),
        lernie_data: root.path().join("lernie"),
        yog_state: root.path().join("state"),
        balls_clones: root.path().join("balls").join("clones"),
        home: root.path().join("home"),
        world: crate::test_support::no_world(),
    };
    let clock = FakeClock::new();
    let (mut model, _deriver) =
        AppModel::boot(roots, None, clock.arc(), Box::new(no_balls()), None);
    assert!(model.workspaces().is_empty());
    assert_eq!(model.focused_workspace(), None);
    // With no focus and an empty roster, the read-only queries are empty and a
    // keyboard step is a no-op (the general empty path, not a special case).
    assert!(model.conversations(10).is_empty(), "no focus: no rows");
    assert!(!model.focused_is_replay());
    model.roster_step(1);
    assert_eq!(model.focused_workspace(), None, "nothing to step onto");
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
fn needs_liveness_reprobe_fires_only_on_live_or_in_flight() {
    let live = GitTree {
        commits: vec![],
        agents: vec![agent("x", AgentState::Live)],
    };
    let inflight = GitTree {
        commits: vec![],
        agents: vec![agent("y", AgentState::InFlight)],
    };
    let quiescent = GitTree {
        commits: vec![],
        agents: vec![agent("z", AgentState::Quiescent)],
    };
    assert!(needs_liveness_reprobe(&live));
    assert!(needs_liveness_reprobe(&inflight));
    assert!(!needs_liveness_reprobe(&quiescent));
    assert!(!needs_liveness_reprobe(&GitTree::default()));
}

#[test]
fn idle_tick_changes_nothing() {
    let h = Harness::new();
    let (_c, mut model) = h.model();
    assert!(!model.tick(), "no dirt, no due sweep → no change");
}
