//! The run holder's own facts (REMOTE §8.3, bl-c285): a started run reaches the
//! buffer and settles on its own thread, a second start replaces the first, an
//! hour-old run is swept, and a lane frame is what the standing gained.
//!
//! Two shapes of run are driven here on purpose. The **real** one is a fake `bz`
//! script — the only way to prove the spawn carries the workspace's wall and
//! that the reader thread exists at all — and the **wired** one is
//! [`Streamed::from_rx`], which drives every buffer arm with no process to race.

use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Duration;

use tempfile::tempdir;

use super::Runs;
use crate::cli_outbound::{Chunk, Cli, ExitInfo, Streamed};
use crate::login::{LoginRun, LoginView};

/// The workspace every case here signs in, and the world its wall folds out of.
fn workspace() -> PathBuf {
    crate::test_support::world::fixture_workspace()
}

fn script(dir: &Path, name: &str, body: &str) -> PathBuf {
    let path = dir.join(name);
    crate::test_support::write_exec(&path, body);
    path
}

/// A run wired to a channel — no child, so every buffer arm is deterministic.
fn wired(state_root: &Path) -> (LoginRun, mpsc::Sender<Chunk>) {
    let (tx, rx) = mpsc::channel();
    (
        LoginRun::from_streamed(Streamed::from_rx(rx), vec!["bz".to_owned()], state_root),
        tx,
    )
}

/// Wait for the reader thread to settle the run — bounded, because the fake
/// exits at once and a hang here is the defect, not a slow box. Running out
/// **panics**: handing back an unsettled view would turn a dead reader thread
/// into a quiet assertion failure three lines later, and would leave this
/// function with a tail no passing run ever reaches.
fn settled(runs: &Runs, provider: &str) -> LoginView {
    for _ in 0..600 {
        let view = runs.standing(&workspace(), provider);
        if view.outcome.is_some() {
            return view;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    panic!("the reader thread never settled {provider}'s run");
}

#[test]
fn a_started_run_carries_the_wall_streams_to_the_buffer_and_settles_itself() {
    let dir = tempdir().expect("tmp");
    let world = crate::test_support::world::world_under(dir.path());
    let state = tempdir().expect("tmp");
    // bz writes its whole human-facing flow to stderr (§8.3 rule 3); the wall
    // is echoed from the child's own environment, which is the only place it
    // can be observed.
    let bz = script(
        dir.path(),
        "bz",
        "#!/bin/sh\nprintf '%s\\n' \"$YOG_WALL\" 1>&2\nexit 0\n",
    );
    let runs = Runs::of(Cli::new(bz));

    // The act answers at once, and what it answers is the standing — a run that
    // has just started has said nothing yet, which is not the same as absent.
    let receipt = runs
        .start(&world, &workspace(), "openai", state.path(), "100")
        .expect("started");
    assert_eq!(receipt.outcome, None, "the act never waits for the run");

    // Nobody polled it: the reader thread did (module doc), and settling is
    // what leaves the ONE §4.2 outcome row.
    let view = settled(&runs, "openai");
    assert_eq!(view.outcome, Some(0));
    assert_eq!(
        view.lines
            .iter()
            .map(|l| l.text.clone())
            .collect::<Vec<_>>(),
        [crate::world::wall::root_of(&world, &workspace())
            .display()
            .to_string()],
        "the child runs inside the NAMED workspace's wall (bl-fcd5)"
    );
    assert!(view.lines.iter().all(|l| l.err), "bz speaks on stderr");
    assert_eq!(crate::opslog::tail(state.path(), 8).len(), 1);

    // A second `Login` on a live pair replaces it: the standing starts over,
    // which is what a seat re-asking the lane then replays.
    let again = runs
        .start(&world, &workspace(), "openai", state.path(), "101")
        .expect("restarted");
    assert_eq!(again.outcome, None, "the settled run was replaced");
    assert_eq!(settled(&runs, "openai").outcome, Some(0));
}

#[test]
fn a_spawn_that_cannot_start_refuses_and_seats_nothing() {
    let dir = tempdir().expect("tmp");
    let world = crate::test_support::world::world_under(dir.path());
    let state = tempdir().expect("tmp");
    let runs = Runs::of(Cli::new("/definitely/not/a/real/bz-xyz"));
    let refusal = runs
        .start(&world, &workspace(), "openai", state.path(), "100")
        .expect_err("an absent binary cannot start a sign-in");
    assert!(!refusal.is_empty(), "the refusal names something");
    // `login::start` already left the synthetic row; nothing is seated, so the
    // pair reads as never signed in rather than as a run that says nothing.
    assert_eq!(
        runs.standing(&workspace(), "openai"),
        LoginView::default(),
        "a failed spawn leaves no run to follow"
    );
}

#[test]
fn a_reader_reads_only_its_own_run_and_stops_when_it_is_replaced() {
    let dir = tempdir().expect("tmp");
    let (run, tx) = wired(dir.path());
    let runs = Runs::default();
    let first = runs.seat(&workspace(), "openai", run, 0);

    // Its own run, still live: the look drains and asks for another.
    tx.send(Chunk::Stderr(b"open https://x/auth\n".to_vec()))
        .expect("send");
    assert!(runs.read_once(&workspace(), "openai", first));
    assert_eq!(
        runs.standing(&workspace(), "openai").lines.len(),
        1,
        "the look put the line in the buffer"
    );

    // Replaced: the slot is another run's, so this reader retires with no flag
    // anyone had to set — and the run it was reading was dropped at the swap.
    let (second_run, second_tx) = wired(dir.path());
    let second = runs.seat(&workspace(), "openai", second_run, 0);
    assert!(!runs.read_once(&workspace(), "openai", first));
    assert!(runs.read_once(&workspace(), "openai", second));

    // Its own run, settled: the look folds the outcome in and retires too.
    second_tx
        .send(Chunk::Exited(ExitInfo::Code(0)))
        .expect("send");
    assert!(!runs.read_once(&workspace(), "openai", second));
    assert_eq!(runs.standing(&workspace(), "openai").outcome, Some(0));

    // A key nobody seated is nobody's run: no reader, and no frame.
    assert!(!runs.read_once(Path::new("/other"), "openai", second));
    assert!(runs.frame(Path::new("/other"), "openai", 0).is_none());
    assert_eq!(
        runs.standing(Path::new("/other"), "openai"),
        LoginView::default(),
        "a pair with no run is emptiness, never a refusal"
    );
}

#[test]
fn a_frame_is_what_the_standing_gained_since_the_frame_before_it() {
    let dir = tempdir().expect("tmp");
    let (run, tx) = wired(dir.path());
    let runs = Runs::default();
    let serial = runs.seat(&workspace(), "openai", run, 0);
    tx.send(Chunk::Stderr(b"one\ntwo\n".to_vec()))
        .expect("send");
    runs.read_once(&workspace(), "openai", serial);

    let whole = runs.frame(&workspace(), "openai", 0).expect("a run stands");
    assert_eq!(whole.lines.len(), 2, "from zero, the whole buffer");
    let tail = runs.frame(&workspace(), "openai", 1).expect("a run stands");
    assert_eq!(tail.lines.len(), 1, "from one, the append");
    // A cursor past the end is an empty append, not a panic and not a replay.
    assert!(
        runs.frame(&workspace(), "openai", 9)
            .expect("a run stands")
            .lines
            .is_empty()
    );
}

#[test]
fn a_run_older_than_an_hour_is_swept_at_the_next_start() {
    let dir = tempdir().expect("tmp");
    let world = crate::test_support::world::world_under(dir.path());
    let state = tempdir().expect("tmp");
    let bz = script(dir.path(), "bz", "#!/bin/sh\nexit 0\n");
    let runs = Runs::of(Cli::new(bz));
    runs.seat(&workspace(), "stale", wired(dir.path()).0, 0);
    runs.seat(&workspace(), "recent", wired(dir.path()).0, 1_000);

    // The sweep happens at the one moment the map can grow (the mailbox's own
    // discipline), and the bound is the whole run's age.
    runs.start(&world, &workspace(), "openai", state.path(), "4000")
        .expect("started");
    assert_eq!(
        runs.standing(&workspace(), "stale"),
        LoginView::default(),
        "an hour-old run is gone"
    );
    assert!(
        runs.frame(&workspace(), "recent", 0).is_some(),
        "a run inside the bound is untouched"
    );
    assert_eq!(settled(&runs, "openai").outcome, Some(0));
}
