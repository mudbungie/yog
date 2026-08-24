//! Tick / sweep / re-derivation tests (§7.2, §7.3), plus the §6 attention facts
//! that ride on a derivation: the held acknowledgement and what a
//! conversation's classified rest does to the strip. What a pass **adopts** off
//! the yog-state root is [`adopt`], split off at the cap.

mod adopt;

use super::{Harness, agent};
use crate::git_tree::{AgentState, GitTree};
use crate::watch::Mark;
use std::path::{Path, PathBuf};
use std::time::Duration;

fn agent_count(model: &super::Rig, ws: &Path) -> usize {
    model.tree(ws).map_or(0, |t| t.agents.len())
}

/// Drive one dirty workspace root through the 100 ms debounce and re-derive it.
pub(super) fn settle(model: &mut super::Rig, clock: &crate::test_support::FakeClock) {
    clock.advance(Duration::from_millis(10));
    model.tick(); // opens the debounce window
    clock.advance(Duration::from_millis(150));
    model.tick(); // window elapsed → re-derive
}

#[test]
fn a_dirty_workspace_root_re_derives_its_snapshot() {
    let h = Harness::new();
    let (clock, mut model) = h.model();
    assert_eq!(agent_count(&model, &h.ws), 1);
    h.build_more("c-2", "world"); // a second agent lands on disk
    assert_eq!(agent_count(&model, &h.ws), 1, "stale until re-derived");

    model.dirty_handle().mark_all([(h.ws.clone(), Mark::Watch)]);
    clock.advance(Duration::from_millis(10));
    assert!(!model.tick(), "debounced within the window");
    clock.advance(Duration::from_millis(150));
    assert!(model.tick(), "window elapsed → snapshot replaced");
    assert_eq!(agent_count(&model, &h.ws), 2);
}

#[test]
fn an_unchanged_re_derivation_replaces_nothing() {
    let h = Harness::new();
    let (clock, mut model) = h.model();
    model.dirty_handle().mark_all([(h.ws.clone(), Mark::Watch)]);
    clock.advance(Duration::from_millis(10));
    model.tick();
    clock.advance(Duration::from_millis(150));
    assert!(
        !model.tick(),
        "an unchanged tree suppresses the replacement"
    );
}

#[test]
fn a_failed_re_read_keeps_the_last_snapshot() {
    let h = Harness::new();
    let (clock, mut model) = h.model();
    // A bogus workspace root that cannot derive: routed as a workspace, debounced,
    // then `probes.derive` fails → no snapshot, no change.
    let bogus = PathBuf::from("/nonexistent/workspace");
    model
        .dirty_handle()
        .mark_all([(bogus.clone(), Mark::Watch)]);
    settle(&mut model, &clock);
    assert!(model.tree(&bogus).is_none(), "a failed read stores nothing");
    assert_eq!(agent_count(&model, &h.ws), 1, "the good snapshot is intact");
}

#[test]
fn the_full_sweep_re_derives_every_workspace() {
    let h = Harness::new();
    let (clock, mut model) = h.model();
    h.build_more("c-2", "yo");
    clock.advance(Duration::from_secs(15)); // full sweep marks everything dirty
    model.tick();
    clock.advance(Duration::from_millis(150)); // debounce elapses
    model.tick();
    assert_eq!(agent_count(&model, &h.ws), 2, "the 15 s sweep re-derived");
}

#[test]
fn the_cheap_sweep_re_probes_and_invalidates_only_live_workspaces() {
    let h = Harness::new();
    let (clock, mut model) = h.model();
    // The fixture agent is Stopped: a cheap sweep leaves it alone.
    clock.advance(crate::app::dirty::CHEAP_SWEEP);
    assert!(!model.tick(), "a quiescent workspace: no targeted re-probe");
    // Inject a Live agent: the next cheap sweep invalidates its lock cache and
    // schedules a re-derive (which re-reads the real Stopped fixture).
    model.deriver.trees.insert(
        h.ws.clone(),
        GitTree {
            commits: vec![],
            agents: vec![agent("c-1", AgentState::Live)],
        },
    );
    clock.advance(crate::app::dirty::CHEAP_SWEEP);
    model.tick(); // cheap sweep: invalidate_liveness + mark dirty
    clock.advance(Duration::from_millis(150));
    assert!(model.tick(), "a live agent: the cheap sweep re-derives");
    assert_eq!(agent_count(&model, &h.ws), 1, "re-read the real fixture");
}

#[test]
fn an_enumeration_root_event_reconciles_the_workspace_set() {
    let mut h = Harness::new();
    let (clock, mut model) = h.model();
    assert_eq!(model.workspaces().len(), 1);
    // A second workspace appears on disk, then the workspaces enum root fires.
    let ws2 = h.add_workspace("ws2", "d-1");
    model
        .dirty_handle()
        .mark_all([(h.roots.workspaces(), Mark::Watch)]);
    model.tick(); // dispatch → reconcile: re-enumerate + watch + mark new dirty
    assert_eq!(
        model.workspaces().len(),
        2,
        "the new workspace is enumerated"
    );
    // Before its debounce elapses it has no snapshot → the roster counts zero.
    assert_eq!(model.workspace_stats(&ws2), (0, 0, false));
    // After the debounce it derives.
    clock.advance(Duration::from_millis(150));
    model.tick();
    assert_eq!(agent_count(&model, &ws2), 1);
}

#[test]
fn a_vanished_workspace_is_pruned_on_reconcile() {
    let mut h = Harness::new();
    let ws2 = h.add_workspace("ws2", "d-1");
    let (_c, mut model) = h.model();
    assert_eq!(
        model.workspaces().len(),
        2,
        "both enumerated at construction"
    );
    assert_eq!(agent_count(&model, &ws2), 1);
    // Remove the symlink and reconcile via the enum root.
    std::fs::remove_file(&ws2).unwrap();
    model
        .dirty_handle()
        .mark_all([(h.roots.workspaces(), Mark::Watch)]);
    model.tick();
    assert_eq!(model.workspaces().len(), 1, "the removed workspace is gone");
    assert!(model.tree(&ws2).is_none(), "its snapshot is pruned");
}

#[test]
fn every_enum_root_and_the_state_root_are_routed() {
    let h = Harness::new();
    let (_c, mut model) = h.model();
    // One drain carrying all enum roots, the state root, and a workspace root
    // exercises each dispatch arm (adopt / reconcile ×3 / workspace debounce).
    model.dirty_handle().mark_all([
        (h.roots.yog_state.clone(), Mark::Watch),
        (h.roots.names(), Mark::Watch),
        (h.roots.workspaces(), Mark::Watch),
        (h.roots.replays(), Mark::Watch),
        (h.ws.clone(), Mark::Watch),
    ]);
    assert!(
        !model.tick(),
        "no snapshot delta this tick (workspace only debounced)"
    );
    assert_eq!(model.workspaces().len(), 1, "reconcile kept the set");
}
