//! bl-b54e acceptance: a process that has made a §4.1 `ui.json` change and is
//! then **signalled** — not closed — still has the change on disk.
//!
//! `make ux` runs `pkill -x yog` every iteration, and SIGTERM reaches no
//! eframe `on_exit` hook. yog no longer has one: §4.1 state is write-through at
//! the gesture (`yog::ui_state`), so there is no in-flight window for a signal
//! to take. This test proves that across a real process boundary, which is the
//! only place the claim can be proved — a same-process test can only observe
//! the file, never the absence of a shutdown path.
//!
//! Shape: the parent re-executes **this test binary** with
//! `YOG_SIGTERM_FIXTURE` set, which selects the child arm below. The child
//! makes one gesture, announces itself on stdout, and then blocks forever with
//! no flush, no tick and no exit path of any kind. The parent SIGTERMs it by
//! pid (`kill -TERM`, what `pkill` posts) and reads the file back. Before the
//! write-through change the file did not exist at that point at all — the pin
//! was sitting in a 250 ms debounce window.

use std::io::{BufRead, BufReader, Write};
use std::os::unix::process::ExitStatusExt;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;

use tempfile::tempdir;
use yog::ui_state::UiState;

/// Names the child arm and carries the state dir it should write into.
const FIXTURE_ENV: &str = "YOG_SIGTERM_FIXTURE";
/// The child's "the gesture is dispatched" handshake, on stdout.
const READY: &str = "yog-sigterm-child-ready";
/// The durable §4.1 assertion under test (a pin).
const PIN: &str = "/pinned/before/the/signal";
/// The child arm's libtest name, for the `--exact` filter the parent re-execs
/// with. Module-qualified: this file is a module of the consolidated
/// `tests/integration` binary, not a binary of its own, so the bare function
/// name matches nothing and the child would run zero tests and never hand
/// back its handshake.
const CHILD_ARM: &str = "sigterm_durability::ui_json_gesture_then_block";

/// The child arm. Inert in a normal test run (no `FIXTURE_ENV`, no child).
#[test]
fn ui_json_gesture_then_block() {
    let Ok(dir) = std::env::var(FIXTURE_ENV) else {
        return;
    };
    let mut ui = UiState::open(PathBuf::from(dir).join("ui.json"));
    ui.set_pinned(vec![PIN.to_string()]);
    // Nothing follows the gesture: no flush, no tick, no exit hook. If the
    // pin is not already on disk, the signal below destroys it.
    // The leading newline terminates libtest's own unfinished `test … ` line,
    // so the handshake arrives as a line of its own on the parent's reader.
    println!("\n{READY}");
    std::io::stdout().flush().unwrap();
    std::thread::sleep(Duration::from_mins(1));
}

#[test]
fn sigterm_keeps_a_ui_json_change_made_moments_before() {
    let d = tempdir().unwrap();
    let exe = std::env::current_exe().unwrap();
    let mut child = yog::git_env::command(&exe)
        .args([CHILD_ARM, "--exact", "--nocapture", "--test-threads=1"])
        .env(FIXTURE_ENV, d.path())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    // Block on the handshake — no polling, no sleeps: libtest's own banner
    // lines stream past until the child's own line arrives.
    let mut lines = BufReader::new(child.stdout.take().unwrap()).lines();
    let ready = lines.any(|l| l.is_ok_and(|l| l.trim() == READY));
    assert!(ready, "child never reported its gesture");

    // Exactly what `pkill -x yog` posts, and exactly what eframe's graceful
    // close path never sees.
    let signalled = yog::git_env::command(Path::new("kill"))
        .args(["-TERM", &child.id().to_string()])
        .status()
        .unwrap();
    assert!(signalled.success(), "kill -TERM failed");

    let status = child.wait().unwrap();
    assert_eq!(
        status.signal(),
        Some(15),
        "the child died on SIGTERM, never through an exit path: {status:?}"
    );

    let back = std::fs::read_to_string(d.path().join("ui.json")).unwrap();
    assert!(
        back.contains(PIN),
        "the pin survived the signal, unflushed by anything: {back}"
    );
}
