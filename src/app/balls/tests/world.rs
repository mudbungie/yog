//! The hermetic world every `app::balls` test drives: a cloned project, two
//! named workspaces, and a fake `bl` whose live and closed replies the test
//! rewrites between fetches to observe the cadence.
//!
//! Split from `tests/mod.rs` at the cap — the fixture and the assertions
//! change for unrelated reasons.

use super::*;
use crate::app::Roots;
use crate::app::tests::Rig;
use crate::opslog;
use crate::projects::balls::{Ball, parse_list};
use crate::projects::runner::BlRunner;
use crate::test_support::FakeClock;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::sync::{Arc, Mutex};
use tempfile::{TempDir, tempdir};

/// A fake `bl` whose per-project `list` and `list_closed` replies the test can
/// rewrite between fetches (shared via the retained `Arc`s). A project in `fail`
/// makes `list` error — a process failure, distinct from an empty listing.
#[derive(Clone)]
pub(crate) struct FakeBl {
    pub(crate) live: Arc<Mutex<HashMap<PathBuf, String>>>,
    pub(crate) closed: Arc<Mutex<HashMap<PathBuf, String>>>,
    pub(crate) fail: Arc<Mutex<HashSet<PathBuf>>>,
}

impl BlRunner for FakeBl {
    fn live(&self, project: &Path) -> std::io::Result<Vec<Ball>> {
        if self.fail.lock().unwrap().contains(project) {
            return Err(std::io::Error::other("store unreadable"));
        }
        Ok(canned(&self.live, project))
    }
    fn closed(&self, project: &Path) -> std::io::Result<Vec<Ball>> {
        Ok(canned(&self.closed, project))
    }
    fn detail(&self, project: &Path, id: &str) -> Option<Ball> {
        canned(&self.live, project).into_iter().find(|b| b.id == id)
    }
}

/// The canned bedrock listing for `project`, parsed through the same forgiving
/// reader the real closed listing uses; an unkeyed project lists empty.
pub(crate) fn canned(src: &Arc<Mutex<HashMap<PathBuf, String>>>, project: &Path) -> Vec<Ball> {
    parse_list(
        src.lock()
            .unwrap()
            .get(project)
            .map_or("[]", String::as_str),
    )
}

pub(crate) struct World {
    pub(crate) _root: TempDir,
    pub(crate) roots: Roots,
    pub(crate) project: PathBuf,
    pub(crate) ws_cobalt: PathBuf,
    pub(crate) ws_spare: PathBuf,
    pub(crate) live: Arc<Mutex<HashMap<PathBuf, String>>>,
    pub(crate) closed: Arc<Mutex<HashMap<PathBuf, String>>>,
    pub(crate) fail: Arc<Mutex<HashSet<PathBuf>>>,
}

/// Two live balls in one cloned project: `bl-work` (claimant = the local
/// workspace name "cobalt", so bound) and `bl-boss` (claimant "boss", a name no
/// local workspace carries → claimed-elsewhere).
const LIST: &str = r#"[{"id":"bl-work","title":"Work","claimant":"cobalt"},
{"id":"bl-boss","title":"Boss","claimant":"boss"}]"#;

pub(crate) fn world() -> World {
    let root = tempdir().unwrap();
    let roots = Roots {
        yog_data: root.path().join("yog"),
        lernie_data: root.path().join("lernie"),
        yog_state: root.path().join("state"),
        balls_clones: root.path().join("clones"),
        home: root.path().join("home"),
        world: crate::test_support::no_world(),
    };
    // A clone whose percent-encoded basename decodes to /proj/a.
    fs::create_dir_all(roots.balls_clones.join("%2Fproj%2Fa")).unwrap();
    fs::create_dir_all(&roots.yog_state).unwrap();
    let project = PathBuf::from("/proj/a");
    // Named workspaces under yog's flat names root: the leaf is the name.
    let ws = |name: &str| {
        let p = roots.yog_data.join("workspaces").join(name);
        fs::create_dir_all(p.join("repo.git")).unwrap();
        p
    };
    let ws_cobalt = ws("cobalt");
    let ws_spare = ws("spare");
    let live = Arc::new(Mutex::new(HashMap::from([(
        project.clone(),
        LIST.to_string(),
    )])));
    let closed = Arc::new(Mutex::new(HashMap::new()));
    let fail = Arc::new(Mutex::new(HashSet::new()));
    World {
        _root: root,
        roots,
        project,
        ws_cobalt,
        ws_spare,
        live,
        closed,
        fail,
    }
}

pub(crate) fn model(w: &World) -> (FakeClock, Rig) {
    build_model(w, None)
}

/// A model with an explicit startup focus — the start-target derivation tests.
pub(crate) fn model_focused(w: &World, ws: &std::path::Path) -> (FakeClock, Rig) {
    build_model(w, Some(ws.to_path_buf()))
}

pub(crate) fn build_model(w: &World, focus: Option<PathBuf>) -> (FakeClock, Rig) {
    let clock = FakeClock::new();
    let (model, deriver) = AppModel::boot(
        w.roots.clone(),
        focus,
        clock.arc(),
        Box::new(FakeBl {
            live: w.live.clone(),
            closed: w.closed.clone(),
            fail: w.fail.clone(),
        }),
        Some("me".to_string()),
    );
    (clock, Rig { model, deriver })
}

pub(crate) fn set_list(w: &World, json: &str) {
    w.live
        .lock()
        .unwrap()
        .insert(w.project.clone(), json.to_string());
}

pub(crate) fn set_closed(w: &World, json: &str) {
    w.closed
        .lock()
        .unwrap()
        .insert(w.project.clone(), json.to_string());
}

pub(crate) fn append_op(m: &AppModel) {
    opslog::append(
        m.state_root(),
        &opslog::OpEntry {
            ts: "TS".into(),
            argv: vec!["bl".into(), "close".into(), "bl-work".into()],
            cwd: "/proj/a".into(),
            exit: 0,
            stdout: String::new(),
            stderr: String::new(),
            origin: opslog::Origin::Balls,
        },
    )
    .unwrap();
}
