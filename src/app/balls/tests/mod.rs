//! Live `bl` projection wiring: ball fetch, §3.5 claimant join badges,
//! Close/Unclaim enablement target, and the ops tail cache (§15 Y16). A hermetic
//! world with a cloned project, two named workspaces, and a fake `bl` whose live
//! and closed replies the test mutates to observe the fetch cadence and the
//! on-demand delivered refresh.

use super::*;
use crate::actions::{close_enabled, unclaim_enabled};
use crate::app::Roots;
use crate::projects::join::JoinState;
use crate::test_support::FakeClock;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::sync::{Arc, Mutex};
use tempfile::{TempDir, tempdir};

// The internal-toggle (§5.1 #1), start-flow input, and fetch-cadence tests share
// this world but live in their own files to stay under the 300-line source cap.
mod world;

pub(crate) use world::*;

mod internal;
mod refresh;
// The ops-tail / surface-failure view-model tests (§4.2, §7.3) likewise share
// this world from their own file.
mod ops;
// The operator's own two trail gestures (§4.2 as amended, bl-c417) — the ack
// watermark and the clear verb, driven through the same world.
mod ack;
// The derived conversation↔ball join (§3.2/§3.3/§3.5, bl-de16): goal-stamp
// balls resolved through the claimant join. Its own hermetic fixture world.
mod convball;
// The §3.6 unmaking's claim enumeration (which bound balls it releases) — its own
// file under the 300-line cap, sharing this world's cloned project.
mod delete;
// The start flow's focus adoption (§3.4, bl-2826): it shares this world but spawns
// a fake `litany`, so it lives in its own file under the 300-line cap.
mod prepare;
// The frame-side boundary glue (dispatch / fire_prompt, §8.5) — its own file,
// same fake-`litany` discipline as `prepare`.
mod glue;
// The §8.5 line context (what a slash command elides) — its own file, this
// world's focus read as a seat.
mod line;

/// The §3.5 join row for one workspace, off the **answered** binding table
/// (`Query::Balls`) — the rows are addressed by name since bl-b4b5, so a test
/// asks the boundary rather than a deleted accessor.
fn row_for(m: &AppModel, ws: &Path) -> Option<crate::projects::join::JoinRow> {
    let name = crate::naming::leaf(ws);
    m.snap
        .join_rows
        .iter()
        .find(|r| r.workspace.as_deref() == Some(name.as_str()))
        .cloned()
}

#[test]
fn construction_fetches_balls_and_builds_the_claimant_join() {
    let w = world();
    let (_c, m) = model(&w);
    // bl-work claims the local name "cobalt" → the workspace is Bound.
    let bound = row_for(&m, &w.ws_cobalt).unwrap();
    assert_eq!(bound.state, JoinState::Bound, "claimant = workspace name");
    assert_eq!(bound.ball_id, "bl-work");
    // "spare" is claimed by nothing → an unassigned workspace.
    let spare = row_for(&m, &w.ws_spare).unwrap();
    assert_eq!(spare.state, JoinState::UnassignedWorkspace);
    // Bound cobalt renders its one ball, unbadged (Bound needs none); spare and an
    // unknown workspace render no ball at all.
    let cobalt_balls = crate::test_support::chrome::ws_balls(&m, &w.ws_cobalt);
    assert_eq!(cobalt_balls.len(), 1);
    assert_eq!(cobalt_balls[0].id, "bl-work");
    assert_eq!(cobalt_balls[0].badge, None);
    assert!(crate::test_support::chrome::ws_balls(&m, &w.ws_spare).is_empty());
    assert!(crate::test_support::chrome::ws_balls(&m, Path::new("/nope")).is_empty());
}

#[test]
fn the_focused_workspaces_ball_row_is_the_one_the_verbs_aim_at() {
    let w = world();
    let (_c, mut m) = model(&w);
    m.focus_workspace(&crate::naming::leaf(&w.ws_cobalt));
    // The seat's own row: the first of the focused workspace's answered
    // listing, which is what `shell::ball_bar` paints and what `c`/`r` fire on.
    let ws = m.focused_workspace().unwrap();
    let row = crate::test_support::chrome::ws_balls(&m, &ws)
        .first()
        .cloned()
        .unwrap();
    assert_eq!(row.id, "bl-work");
    assert!(close_enabled(row.state), "Bound ⇒ Close offered");
    assert!(unclaim_enabled(row.state));

    // "spare" is a named workspace no ball claims: its only row is the §3.5
    // UnassignedWorkspace one, which names no ball and no project. The listing
    // must not hand that row out — the ball row and the marks knob would read
    // it as a ball ("ball " with an empty id) and as a project.
    m.focus_workspace(&crate::naming::leaf(&w.ws_spare));
    let ws = m.focused_workspace().unwrap();
    assert!(
        crate::test_support::chrome::ws_balls(&m, &ws).is_empty(),
        "a workspace with no ball answers no ball"
    );
    // The join row itself still exists — the roster renders the workspace; it
    // is only "the focused ball" that is absent.
    let spare = row_for(&m, &w.ws_spare).unwrap().state;
    assert_eq!(spare, JoinState::UnassignedWorkspace);
    assert!(!close_enabled(spare), "unassigned workspace ⇒ no Close");
    assert!(!unclaim_enabled(spare));
}

#[test]
fn a_bound_ball_gets_one_roster_row_not_two() {
    // bl-abbe: the §11 roster's ball rows partition the §3.5 states — a Bound
    // ball is rendered in full by ▶ Continue, so the section's own list must
    // not also emit it as a bare id with no title, state or verb.
    let w = world();
    let (_c, m) = model(&w);
    let rows = crate::test_support::chrome::ws_balls(&m, &w.ws_cobalt);
    assert_eq!(rows.len(), 1, "cobalt holds bl-work");
    assert_eq!(m.resumable().len(), 1, "▶ Continue renders it in full");
    assert!(
        crate::nav::balls::roster(&rows).is_empty(),
        "one ball, one row"
    );
    // The Continue row carries the ball's own object, so its §11 accelerator
    // menu acts on the ball without re-deriving anything from the focus.
    let ball = crate::nav::balls::bound(&rows, "bl-work").unwrap();
    assert_eq!(ball.owner, "cobalt");
    assert_eq!(ball.state, JoinState::Bound);
    assert!(crate::nav::balls::bound(&rows, "bl-nope").is_none());
}

#[test]
fn identity_and_state_root_are_exposed() {
    let w = world();
    let (_c, m) = model(&w);
    assert_eq!(
        m.identity(),
        "me",
        "no recorded identity ⇒ the $USER fallback"
    );
    assert_eq!(m.state_root(), w.roots.yog_state.as_path());
}

#[test]
fn an_unlistable_clone_renders_orphaned() {
    let w = world();
    // A second clone whose `bl list` fails (a process failure, not an empty
    // listing): it is enumerated but absent from the ball map, so the join marks
    // it orphaned (§3.5), distinct from a listable-but-empty project.
    fs::create_dir_all(w.roots.balls_clones.join("%2Fproj%2Fgone")).unwrap();
    let bad = PathBuf::from("/proj/gone");
    w.fail.lock().unwrap().insert(bad.clone());
    let (_c, m) = model(&w);
    let orphan = m
        .snap
        .join_rows
        .iter()
        .find(|r| r.project == crate::naming::leaf(&bad))
        .expect("the unlistable clone has a row");
    assert_eq!(orphan.state, JoinState::OrphanedProject);
}

/// An empty-world model — no workspaces, no balls — so startup derives **no**
/// focus (§4.1). The one state where a bare start takes §3.1's default name and `focused_join`
/// is `None`. Returns the tempdir so its dirs outlive construction.
pub(super) fn empty_model() -> (TempDir, AppModel) {
    let root = tempdir().unwrap();
    let roots = Roots {
        yog_data: root.path().join("yog"),
        litany_data: root.path().join("litany"),
        yog_state: root.path().join("state"),
        balls_clones: root.path().join("clones"),
        home: root.path().join("home"),
        world: crate::test_support::no_world(),
    };
    let empty = FakeBl {
        live: Arc::new(Mutex::new(HashMap::new())),
        closed: Arc::new(Mutex::new(HashMap::new())),
        fail: Arc::new(Mutex::new(HashSet::new())),
    };
    let (m, _deriver) = AppModel::boot(roots, None, FakeClock::new().arc(), Box::new(empty), None);
    (root, m)
}

#[test]
fn the_seats_ball_row_is_none_without_a_focused_workspace() {
    // An empty world: no workspaces ⇒ no startup focus ⇒ nothing to answer.
    let (_root, m) = empty_model();
    assert!(m.focused_workspace().is_none());
    assert!(m.snap.ops.is_empty());
    assert_eq!(m.identity(), "", "no recorded identity, no user ⇒ empty");
}
