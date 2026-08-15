//! The sweep-instrumentation proofs (bl-49f4, DESIGN §7.2).
//!
//! The 15 s sweep used to repair a dropped event and say nothing, which made the
//! drop rate unmeasurable and the sweep load-bearing over an unknown defect.
//! These tests assert the opposite property end to end: a sweep that finds
//! something says so, on the surface the operator already has (`ops.jsonl` →
//! the §11 activity chip), attributed to a root and a kind — and a sweep that
//! finds nothing stays silent.

use super::super::*;
use super::Harness;
use crate::git_tree::{AgentState, GitTree};
use crate::watch::Mark;
use std::path::Path;
use std::time::Duration;

/// The drift lines the ops tail holds, as `(kind, roots-text)` pairs.
fn drift_rows(model: &AppModel) -> Vec<(String, String)> {
    model
        .snap
        .ops
        .iter()
        .filter(|r| r.drift())
        .map(|r| {
            let kind = r.argv.split_whitespace().nth(1).unwrap_or("").to_string();
            (kind, r.stderr.clone())
        })
        .collect()
}

/// Run the 15 s full sweep and the debounced re-derivation it schedules.
fn full_sweep(model: &mut super::Rig, clock: &crate::test_support::FakeClock) {
    clock.advance(Duration::from_secs(15));
    model.tick();
    clock.advance(Duration::from_millis(150));
    model.tick();
}

#[test]
fn a_full_sweep_that_changes_an_unannounced_snapshot_names_the_dropped_event() {
    let h = Harness::new();
    let (clock, mut model) = h.model();
    // Disk moves and *nothing* announces it — exactly what a dropped inotify
    // event looks like from inside yog.
    h.build_more("c-2", "yo");
    full_sweep(&mut model, &clock);
    assert_eq!(model.tree(&h.ws).map_or(0, |t| t.agents.len()), 2);
    let rows = drift_rows(&model);
    assert_eq!(rows.len(), 1, "one drift line: {rows:?}");
    assert_eq!(rows[0].0, "unannounced", "the kind names the class");
    assert_eq!(
        rows[0].1,
        format!("{}\n", h.ws.display()),
        "attributed to the root it happened on"
    );
    // And it reaches the operator through the §11 chip, as a query over the tail.
    let activity = model.activity();
    assert_eq!(activity.drifts, 1);
    assert_eq!(activity.errors, 0, "a drift is not a failed action");
    assert!(activity.chip().contains("1 drift"));
    assert!(
        model.last_failure(crate::opslog::Origin::World).is_none(),
        "and never hijacks the §7.3 failure banner"
    );
}

#[test]
fn a_quiet_full_sweep_reports_nothing() {
    let h = Harness::new();
    let (clock, mut model) = h.model();
    full_sweep(&mut model, &clock);
    assert!(
        drift_rows(&model).is_empty(),
        "a sweep that catches nothing is silent — the target state"
    );
    assert_eq!(model.activity().drifts, 0);
}

#[test]
fn an_announced_change_is_never_counted_as_a_drop() {
    let h = Harness::new();
    let (clock, mut model) = h.model();
    h.build_more("c-2", "yo");
    // The watcher announces the same root the blanket sweep will also mark:
    // the strongest explanation wins, so the sweep takes no credit for a catch
    // the watcher made.
    model.dirty_handle().mark_all([(h.ws.clone(), Mark::Watch)]);
    full_sweep(&mut model, &clock);
    assert_eq!(model.tree(&h.ws).map_or(0, |t| t.agents.len()), 2);
    assert!(drift_rows(&model).is_empty());
}

#[test]
fn the_liveness_poll_explains_its_own_change_and_is_not_a_drop() {
    let h = Harness::new();
    let (clock, mut model) = h.model();
    // A Live agent that is really Stopped on disk: the cheap sweep's targeted
    // re-probe changes the snapshot with no filesystem event behind it. That is
    // a poll of *process* state doing its job — no watcher could have announced
    // a released flock — so it must not read as a dropped event.
    model.deriver.trees.insert(
        h.ws.clone(),
        GitTree {
            commits: vec![],
            agents: vec![super::agent("c-1", AgentState::Live)],
        },
    );
    clock.advance(crate::app::dirty::CHEAP_SWEEP);
    model.tick();
    clock.advance(Duration::from_millis(150));
    assert!(model.tick(), "the cheap sweep re-derived");
    assert!(drift_rows(&model).is_empty(), "the poll is not a drop");
}

#[test]
fn the_first_start_on_a_pristine_world_does_not_accuse_itself_of_drift() {
    // bl-f726: a fresh seeded world has no names root at all, so nothing is
    // watching the directory the start flow is about to found. The workspace it
    // then mints is *unwatchable at birth* — and the first sweep to meet it used
    // to file both drift kinds against it: `unenumerated` for the membership
    // delta, `unannounced` for its very first snapshot. Two red rows on a
    // pristine, healthy first run, permanent because the trail persists.
    let mut h = Harness::pristine();
    let (clock, mut model) = h.model();
    assert!(model.workspaces().is_empty(), "nothing exists yet");
    let ws = h.mint_named("home", "c-1");
    full_sweep(&mut model, &clock);
    assert!(
        model.tree(&ws).is_some(),
        "the newborn is enumerated+derived"
    );
    assert!(
        drift_rows(&model).is_empty(),
        "a healthy first start paints nothing red: {:?}",
        drift_rows(&model)
    );
    assert_eq!(model.activity().drifts, 0);
}

#[test]
fn a_newborn_workspace_is_still_accused_once_it_has_a_baseline() {
    // The other direction of the same rule: enumerated-at-birth is not a
    // permanent amnesty. Once the workspace has a snapshot to diverge FROM — and
    // a watch armed over it — a change nothing announced is a dropped event like
    // any other.
    let mut h = Harness::pristine();
    let (clock, mut model) = h.model();
    let ws = h.mint_named("home", "c-1");
    full_sweep(&mut model, &clock);
    assert!(drift_rows(&model).is_empty(), "the birth was clean");
    h.last_added().build_agent("c-2", "yo");
    full_sweep(&mut model, &clock);
    let rows = drift_rows(&model);
    assert_eq!(rows.len(), 1, "the dropped event is named: {rows:?}");
    assert_eq!(rows[0].0, "unannounced");
    assert_eq!(rows[0].1, format!("{}\n", ws.display()));
}

#[test]
fn a_workspace_appearing_between_sweeps_is_reported_as_unenumerated() {
    let mut h = Harness::new();
    let (clock, mut model) = h.model();
    // A workspace lands on disk and the NamesRoot/WorkspacesRoot watch never
    // says so — the enumeration-side drop.
    let ws2 = h.add_workspace("ws2", "d-1");
    full_sweep(&mut model, &clock);
    let rows = drift_rows(&model);
    let unenumerated: Vec<&(String, String)> =
        rows.iter().filter(|r| r.0 == "unenumerated").collect();
    assert_eq!(unenumerated.len(), 1, "{rows:?}");
    assert_eq!(unenumerated[0].1, format!("{}\n", ws2.display()));
    // One-shot: the next sweep has nothing new to say about it.
    let before = drift_rows(&model).len();
    full_sweep(&mut model, &clock);
    assert_eq!(
        drift_rows(&model).len(),
        before,
        "a finding is reported once, not re-accused every 15 s"
    );
}

#[test]
fn a_workspace_vanishing_between_sweeps_is_reported_too() {
    let mut h = Harness::new();
    let ws2 = h.add_workspace("ws2", "d-1");
    let (clock, mut model) = h.model();
    std::fs::remove_file(&ws2).unwrap();
    full_sweep(&mut model, &clock);
    let rows = drift_rows(&model);
    assert!(
        rows.iter()
            .any(|r| r.0 == "unenumerated" && r.1 == format!("{}\n", ws2.display())),
        "removal is a membership delta like any other: {rows:?}"
    );
}

#[test]
fn an_undeliverable_workspace_does_not_re_accuse_the_watcher_every_sweep() {
    // The trap the membership-delta rule dissolves: a workspace that enumerates
    // but never derives has no snapshot, forever. Keying "unenumerated" on
    // "missing a snapshot" would file that as fresh drift on every sweep and
    // bury the real findings under it.
    let h = Harness::new();
    let broken = h.roots.workspaces().join("broken");
    std::os::unix::fs::symlink("/nonexistent/target", &broken).unwrap();
    let (clock, mut model) = h.model();
    full_sweep(&mut model, &clock);
    let first = drift_rows(&model).len();
    full_sweep(&mut model, &clock);
    full_sweep(&mut model, &clock);
    assert_eq!(drift_rows(&model).len(), first, "no repeat accusation");
}

#[test]
fn a_backend_announced_loss_is_recorded_and_re_derived_at_once() {
    let h = Harness::new();
    let (clock, mut model) = h.model();
    h.build_more("c-2", "yo");
    // The kernel says "I lost events under this root" (inotify queue overflow,
    // or a watch that could not be armed). yog does not wait for the sweep: the
    // root is dirty, it re-derives now — and the loss is on the record.
    model
        .dirty_handle()
        .mark_all([(h.ws.clone(), Mark::Desync)]);
    model.tick();
    clock.advance(Duration::from_millis(150));
    assert!(model.tick(), "re-derived on the desync, not on the sweep");
    assert_eq!(model.tree(&h.ws).map_or(0, |t| t.agents.len()), 2);
    let rows = drift_rows(&model);
    assert_eq!(rows.len(), 1, "{rows:?}");
    assert_eq!(rows[0].0, "desync");
    assert_eq!(rows[0].1, format!("{}\n", h.ws.display()));
}

#[test]
fn construction_arms_the_watch_before_taking_the_first_snapshot() {
    // Arm-then-read: a watch armed after the initial derive is blind to whatever
    // landed in between, and that gap is a dropped event by construction. The
    // observable is that the watch is live for a workspace the model has
    // already snapshotted — i.e. both happened, in that order, inside `new`.
    let h = Harness::new();
    let (_c, model) = h.model();
    assert!(model.tree(&h.ws).is_some(), "the first snapshot is taken");
    assert!(
        crate::state::lock_watchset(&model.deriver.watchset_handle())
            .watches(Path::new(&h.ws), crate::fs_watcher::RootKind::Workspace),
        "and its watch is armed"
    );
}
