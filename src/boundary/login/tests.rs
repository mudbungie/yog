//! The sign-in at the boundary (REMOTE §8.3, bl-c285): the act fired through
//! the real chokepoint, the read answered through the real chokepoint, and the
//! lane driven look by look with no clock and no sleep.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::mpsc;
use std::time::Duration;

use tempfile::tempdir;

use super::{Frame, Lane};
use crate::boundary::answer::answer;
use crate::boundary::dispatch::{Caller, Deps, dispatch};
use crate::boundary::reply::Reply;
use crate::boundary::{Action, Query};
use crate::cli_outbound::{Chunk, Cli, ExitInfo, Streamed};
use crate::login::runs::Runs;
use crate::login::{LoginRun, LoginView};
use crate::ui_state::UiState;

/// The §3.1 sphere every case here signs in — the one the fixture world's wall
/// is folded off, so a spawn lands where the fixture reads.
fn workspace() -> PathBuf {
    crate::test_support::world::fixture_workspace()
}

/// A hermetic environment whose sign-in runs spawn `bz`.
fn deps_with(root: &Path, bz: &Path) -> Deps {
    let world = crate::test_support::world::world_under(root);
    fs::create_dir_all(world.yog_state_root()).expect("state root");
    Deps {
        litany: Cli::new("/definitely/not/a/litany-xyz"),
        bl: Cli::new("/definitely/not/a/bl-xyz"),
        state_root: world.yog_state_root(),
        yog_binary: root.join("yog"),
        world,
        home: root.join("home"),
        yog_data_root: root.join("data/yog"),
        balls_state_root: root.join("state/balls"),
        snapshot: Arc::new(crate::boundary::tests::snapshot(
            &workspace(),
            "alba",
            vec![],
            vec![],
        )),
        caller: Caller {
            logins: Runs::of(Cli::new(bz)),
            ..Caller::default()
        },
    }
}

fn script(dir: &Path, name: &str, body: &str) -> PathBuf {
    let path = dir.join(name);
    fs::write(&path, body).expect("write");
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).expect("chmod");
    path
}

/// Ask the lane's read through the real chokepoint, so what is tested is the
/// query and not a private helper.
fn ask(deps: &Deps, provider: &str) -> LoginView {
    let ui = UiState::open(PathBuf::from("/nonexistent/ui.json"));
    let asked = Query::LoginTail {
        workspace: crate::naming::leaf(&workspace()),
        provider: provider.to_owned(),
    };
    match answer(&asked, deps, &ui, 0) {
        Ok(Reply::Login(view)) => view,
        other => panic!("the sign-in lane answers a standing: {other:?}"),
    }
}

/// Wait for the engine's own reader thread to settle the run, asking through
/// the real chokepoint each look. Running out panics rather than handing back
/// an unsettled view, for `runs::tests::settled`'s reason exactly.
fn settle(deps: &Deps, provider: &str) -> LoginView {
    for _ in 0..600 {
        let view = ask(deps, provider);
        if view.outcome.is_some() {
            return view;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    panic!("the engine never settled {provider}'s run");
}

/// A run wired to a channel — no child, so every lane arm is deterministic.
fn wired(state_root: &Path) -> (LoginRun, mpsc::Sender<Chunk>) {
    let (tx, rx) = mpsc::channel();
    (
        LoginRun::from_streamed(Streamed::from_rx(rx), vec!["bz".to_owned()], state_root),
        tx,
    )
}

/// A lane over `runs`, held for two quiet looks with no wait between them.
fn lane(runs: &Runs, provider: &str) -> Lane {
    Lane::holding(
        runs.clone(),
        workspace(),
        provider.to_owned(),
        2,
        Duration::ZERO,
    )
}

#[test]
fn the_act_starts_the_run_answers_its_standing_and_the_read_says_the_rest() {
    let dir = tempdir().expect("tmp");
    let bz = script(
        dir.path(),
        "bz",
        "#!/bin/sh\nprintf 'open https://x/auth\\n' 1>&2\nexit 0\n",
    );
    let deps = deps_with(dir.path(), &bz);
    let mut ui = UiState::open(PathBuf::from("/nonexistent/ui.json"));

    // A provider nobody has signed in to answers emptiness, not a refusal.
    assert_eq!(ask(&deps, "openai"), LoginView::default());

    let act = Action::Login {
        workspace: crate::naming::leaf(&workspace()),
        provider: "openai".to_owned(),
    };
    let Ok(Reply::Login(receipt)) = dispatch(&deps, &mut ui, "100", &act) else {
        panic!("the sign-in act answers a standing");
    };
    assert_eq!(receipt.outcome, None, "the act never waits for the run");

    // The rest of the run reaches the same read, on the engine's own thread.
    let settled = settle(&deps, "openai");
    assert_eq!(settled.outcome, Some(0));
    assert_eq!(
        settled
            .lines
            .iter()
            .map(|l| l.text.clone())
            .collect::<Vec<_>>(),
        ["open https://x/auth".to_owned()]
    );
}

#[test]
fn a_lane_on_a_pair_with_no_run_opens_on_the_emptiness_and_then_ends() {
    let runs = Runs::default();
    let mut lane = lane(&runs, "openai");
    // Legible emptiness: one frame saying nothing has been said, and no
    // silence for a seat to misread as a lost connection.
    assert_eq!(lane.next(), Some(Reply::Login(LoginView::default())));
    assert_eq!(lane.next(), None);
}

#[test]
fn a_lane_replays_from_the_start_appends_and_ends_on_the_settled_exit() {
    let dir = tempdir().expect("tmp");
    let runs = Runs::default();
    let (run, tx) = wired(dir.path());
    let serial = runs.seat(&workspace(), "openai", run, 0);
    tx.send(Chunk::Stderr(b"open https://x/auth\n".to_vec()))
        .expect("send");
    runs.read_once(&workspace(), "openai", serial);

    let mut lane = lane(&runs, "openai");
    // The first frame is the whole buffer — a lane holds nothing when it opens,
    // so a re-ask replays rather than losing what was already said.
    let Some(Reply::Login(first)) = lane.next() else {
        panic!("a lane opens on the standing");
    };
    assert_eq!(first.lines.len(), 1);
    assert_eq!(first.outcome, None);

    // Nothing new: the hold's own answer, and never an end.
    assert!(matches!(lane.poll(), Frame::Waiting));

    // An append is what landed since the frame before it.
    tx.send(Chunk::Stderr(b"waiting\n".to_vec())).expect("send");
    runs.read_once(&workspace(), "openai", serial);
    let Frame::Ready(second) = lane.poll() else {
        panic!("an append is a frame");
    };
    assert_eq!(
        second
            .lines
            .iter()
            .map(|l| l.text.clone())
            .collect::<Vec<_>>(),
        ["waiting".to_owned()],
        "the append, not the whole buffer again"
    );

    // The settled exit is the last frame, and the lane is done after it.
    tx.send(Chunk::Exited(ExitInfo::Code(78))).expect("send");
    runs.read_once(&workspace(), "openai", serial);
    let Some(Reply::Login(last)) = lane.next() else {
        panic!("the exit is a frame");
    };
    assert_eq!(last.outcome, Some(78));
    assert!(
        last.fallback.is_some(),
        "a non-zero exit carries the §8.3 fallback"
    );
    assert!(matches!(lane.poll(), Frame::Over));
    assert_eq!(lane.next(), None);
}

#[test]
fn a_lane_ends_when_the_run_it_was_reading_is_gone() {
    let dir = tempdir().expect("tmp");
    let world = crate::test_support::world::world_under(dir.path());
    let state = tempdir().expect("tmp");
    let bz = script(dir.path(), "bz-quiet", "#!/bin/sh\nexit 0\n");
    let runs = Runs::of(Cli::new(bz));
    let (run, _tx) = wired(dir.path());
    runs.seat(&workspace(), "openai", run, 0);
    let mut lane = lane(&runs, "openai");
    assert!(matches!(lane.poll(), Frame::Ready(_)), "it opened on a run");

    // The next sign-in sweeps the hour-old run this lane was reading. The lane
    // ends rather than sliding onto a successor nobody asked it about; the seat
    // re-asks, and a re-ask replays from the start.
    runs.start(&world, &workspace(), "other", state.path(), "4000")
        .expect("started");
    assert!(matches!(lane.poll(), Frame::Over));
    assert_eq!(lane.next(), None);
}

#[test]
fn a_quiet_lane_ends_on_its_hold_rather_than_holding_forever() {
    let dir = tempdir().expect("tmp");
    let runs = Runs::default();
    let (run, _tx) = wired(dir.path());
    runs.seat(&workspace(), "openai", run, 0);
    let mut lane = lane(&runs, "openai");
    assert!(lane.next().is_some(), "the opening frame");
    // Two quiet looks is this lane's whole patience; production's is thirty
    // seconds of them, which is the §5.3 mailbox bound.
    assert_eq!(lane.next(), None);
}
