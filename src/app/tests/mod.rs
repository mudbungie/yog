//! Shared harness + the construction / enumeration / accessor tests for the
//! multi-workspace [`AppModel`] (§15 Y11). The tick/sweep machinery is in
//! `derive`, the roster/attention/seen surface (and the M2 convergence proof)
//! in `focus`.

mod agent_deletes;
mod attention;
mod deletes;
pub(super) mod derive;
mod drift;
mod echo;
mod focus;
pub(crate) mod harness;
mod knobs;
mod panels;
mod search;
mod started;
mod view;
mod worker;

use super::*;
use crate::binding::WorkspaceKind;
use crate::fs_watcher::RootKind;
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
            .any(|t| t.name == crate::naming::leaf(&replay)
                && t.kind == crate::nav::tabs::Kind::Replay),
        "overflow carries the replay: {bar:?}"
    );
    // Focused, it is read-only; the ad-hoc workspace beside it is writable.
    model.focus_workspace(&crate::naming::leaf(&replay));
    assert!(model.focused_is_replay(), "a focused replay is read-only");
    model.focus_workspace(&crate::naming::leaf(&h.ws));
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
    assert_eq!(model.focused_workspace(), Some(h.ws.clone()));
    assert!(
        model.focus().agent.is_none(),
        "no agent selected at startup"
    );
    assert_eq!(model.focused_tree().map(|t| t.agents.len()), Some(1));
}

#[test]
fn startup_focus_override_wins() {
    let h = Harness::new();
    // The override names a workspace (§4.1); the focus holds its §3.1 name and
    // resolves it against the enumeration like any other (bl-7407).
    let (_c, model) = h.model_focused(Some(h.ws.clone()));
    assert_eq!(model.focused_ws_name().as_deref(), Some("ws"));
    assert_eq!(model.focused_workspace(), Some(h.ws.clone()));
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
    // With no focus and an empty roster, the read-only queries are empty and the
    // jump is a no-op (the general empty path, not a special case).
    assert!(
        crate::test_support::convs::conversations(&model, 10).is_empty(),
        "no focus: no rows"
    );
    assert!(!model.focused_is_replay());
    // An unfocused window echoes nothing onto an answered list either: an echo
    // belongs to the workspace it was fired in, and there is none to compare
    // against (REMOTE §9.7, bl-44e9).
    assert!(model.echoed(Vec::new(), 10).is_empty());
    // Nor onto an answered inbox listing, which is the echo's third projection
    // (REMOTE §9.7, bl-b4b5) and takes the same door.
    assert!(model.echoed_pending("c-1", Vec::new()).is_empty());
    model.jump_next_attention();
    assert_eq!(model.focused_workspace(), None, "nothing to jump onto");
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

/// **The wire refusal is a model fact the frame paints** (bl-dc14): kept at
/// the FIRST reason — the engine's own bind refusal outranks the derived "no
/// seat" recorded after it — and `None` on a wired window, which is what lets
/// `shell::refusal` gate the whole shell on one read.
#[test]
fn the_first_wire_refusal_is_the_one_the_frame_paints() {
    let h = Harness::new();
    let (_clock, mut rig) = h.model();
    assert_eq!(
        rig.model.wire_refusal(),
        None,
        "a wired window says nothing"
    );
    rig.model
        .refuse_wire("bind 127.0.0.1:1: Address already in use".to_owned());
    rig.model
        .refuse_wire("this engine has no listener".to_owned());
    assert_eq!(
        rig.model.wire_refusal().as_deref(),
        Some("bind 127.0.0.1:1: Address already in use"),
        "the cause, never the consequence recorded after it"
    );
}
