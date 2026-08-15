//! Fetch-cadence wiring (§7.2, §5.1 #4): the sweep, the clones/project/
//! yog-state dirtiness refreshes, and the two post-verb hooks, against the
//! shared cloned-project world in [`super`]. Split from `mod` to stay under the
//! 300-line source cap.
//!
//! Every one of these is the *worker's* work now (bl-ee0a). A frame that
//! dispatched a verb only names the root it changed; the convergence lands on
//! the next pass, which the rig's `tick` runs by hand.

use super::{append_op, model, set_closed, set_list, world};
use crate::projects::join::JoinState;
use crate::watch::Mark;
use std::time::Duration;

#[test]
fn balls_section_carries_the_delivered_badge_after_a_close() {
    let w = world();
    let (_c, mut m) = model(&w);
    // bl-work closes: gone from live, present in the closed listing under cobalt.
    set_list(&w, r#"[{"id":"bl-boss","claimant":"boss"}]"#);
    set_closed(&w, r#"[{"id":"bl-work","claimant":"cobalt"}]"#);
    m.after_bl_verb(&w.project);
    m.tick(); // the worker's next pass re-fetches the project it was told about
    assert_eq!(
        m.ws_balls(&w.ws_cobalt),
        vec![crate::nav::BoundBall {
            id: "bl-work".to_string(),
            badge: Some("delivered".to_string()),
            // The row names its own object, so its §11 menu acts on it without
            // re-deriving anything from the focus.
            project: w.project.clone(),
            owner: "cobalt".to_string(),
            state: JoinState::Delivered,
        }],
        "the §11 balls section groups the delivered ball under cobalt"
    );
    // bl-abbe: Delivered is the state ▶ Continue does *not* reach, so this is
    // the row the section renders itself — the list is not dead, it is disjoint.
    assert_eq!(m.roster_ball_rows(&w.ws_cobalt), m.ws_balls(&w.ws_cobalt));
    assert!(
        m.resumable().is_empty(),
        "a delivered ball is not resumable"
    );
}

#[test]
fn after_bl_verb_reflects_a_closed_ball_as_delivered() {
    let w = world();
    let (_c, mut m) = model(&w);
    append_op(&m);
    // bl-work closes: it leaves the live set; the on-demand closed listing binds
    // it to cobalt, so the workspace renders Delivered.
    set_list(&w, r#"[{"id":"bl-boss","claimant":"boss"}]"#);
    set_closed(&w, r#"[{"id":"bl-work","claimant":"cobalt"}]"#);
    m.after_bl_verb(&w.project);
    m.tick();
    let delivered = m.ws_balls(&w.ws_cobalt);
    assert_eq!(delivered.len(), 1);
    assert_eq!(delivered[0].badge, Some("delivered".to_string()));
    // The op appended before the refresh is now tailed.
    assert_eq!(m.snap.ops.len(), 1);
    assert_eq!(m.snap.ops[0].argv, "bl close bl-work");
}

#[test]
fn after_lernie_verb_refreshes_only_the_ops_tail() {
    let w = world();
    let (_c, mut m) = model(&w);
    assert!(m.snap.ops.is_empty());
    assert_eq!(m.activity().total, 0, "the §11 chip reads the same cache");
    append_op(&m);
    m.after_lernie_verb();
    m.tick();
    assert_eq!(m.snap.ops.len(), 1);
    assert_eq!(m.activity().total, 1);
}

#[test]
fn full_sweep_refetches_balls_and_ops() {
    let w = world();
    let (clock, mut m) = model(&w);
    append_op(&m);
    set_list(&w, r#"[{"id":"bl-boss","claimant":"boss"}]"#); // bl-work gone
    clock.advance(Duration::from_secs(20)); // > FULL_SWEEP
    m.tick();
    // The cadence does not fetch the closed listing (§5.1 #4): cobalt now has no
    // bound ball, so it is unassigned, not delivered.
    assert_eq!(
        m.row_for(&w.ws_cobalt).unwrap().state,
        JoinState::UnassignedWorkspace
    );
    assert_eq!(m.snap.ops.len(), 1, "ops re-read on the full sweep");
}

#[test]
fn clones_dirtiness_refetches_balls() {
    let w = world();
    let (_c, mut m) = model(&w);
    set_list(&w, "[]"); // both balls gone
    m.dirty_handle()
        .mark_all([(w.roots.balls_clones.clone(), Mark::Watch)]);
    m.tick();
    assert_eq!(
        m.row_for(&w.ws_cobalt).unwrap().state,
        JoinState::UnassignedWorkspace
    );
}

#[test]
fn yog_state_dirtiness_refreshes_the_ops_tail() {
    let w = world();
    let (_c, mut m) = model(&w);
    append_op(&m);
    m.dirty_handle()
        .mark_all([(w.roots.yog_state.clone(), Mark::Watch)]);
    m.tick();
    assert_eq!(m.snap.ops.len(), 1);
}

#[test]
fn a_names_root_reconcile_rebinds_a_ball_to_a_fresh_workspace() {
    // A ball claimed by a name with no local workspace yet → claimed-elsewhere.
    let w = world();
    set_list(&w, r#"[{"id":"bl-new","title":"Go","claimant":"newname"}]"#);
    let (_c, mut m) = model(&w);
    let names = w.roots.yog_data.join("workspaces");
    let newname = names.join("newname");
    assert!(
        m.row_for(&newname).is_none(),
        "no workspace named the claimant yet"
    );

    // The start flow's `lernie new` raises the workspace; the NamesRoot event
    // reconciles. reconcile() must rebuild the join over the already-fetched
    // balls (addendum) — else the just-claimed ball renders claimed-elsewhere
    // until the 15 s sweep. No refresh_balls runs on this path.
    std::fs::create_dir_all(newname.join("repo.git")).unwrap();
    m.dirty_handle().mark_all([(names, Mark::Watch)]);
    m.tick();
    let row = m.row_for(&newname).unwrap();
    assert_eq!(
        row.state,
        JoinState::Bound,
        "rebound on reconcile, no re-fetch"
    );
    assert_eq!(row.ball_id, "bl-new");
}
