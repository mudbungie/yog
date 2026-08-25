//! **Waiting on a filesystem** — the polling harness every watcher beat runs
//! through. Split from the beats at §12's budget on the seam between *how a
//! change is waited for* and *which changes are expected*: arming is not slept
//! on but observed (bl-1015), and that observation is one mechanism with one
//! home rather than a budget re-spelled per beat.

use super::super::{Change, ChangeKind, Watcher};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use tempfile::{TempDir, tempdir};

/// Detection budget for filesystem-watcher latency: poll up to `WAIT_TIMEOUT`,
/// sampling every `POLL_INTERVAL`. On Linux inotify the event lands on the first
/// sample; the budget exists for macOS FSEvents, and twenty seconds is what it
/// measured at (bl-1015) — delivery past five was reproducible on a three-core
/// `macos-14` runner carrying ~2000 tests in parallel. A healthy beat never
/// spends it, so the cost falls only on a beat that is about to fail anyway.
pub(super) const WAIT_TIMEOUT: Duration = Duration::from_secs(20);
pub(super) const POLL_INTERVAL: Duration = Duration::from_millis(50);

/// Poll `probe` until it yields `Some`, or `timeout` elapses. Pure over an
/// injected closure, so both paths are unit tested (`poll_until_*`).
pub(super) fn poll_until<T>(
    mut probe: impl FnMut() -> Option<T>,
    timeout: Duration,
    interval: Duration,
) -> Option<T> {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if let Some(found) = probe() {
            return Some(found);
        }
        std::thread::sleep(interval);
    }
    probe()
}

/// A fresh, canonicalized workspace root. Canonicalizing mirrors
/// `Watcher::new`, so expectations match the watcher's path spelling — on macOS
/// the tempdir is under `/tmp` → `/private/tmp`, which FSEvents resolves. The
/// guard must be kept alive.
pub(crate) fn workspace() -> (TempDir, PathBuf) {
    let dir = tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    (dir, root)
}

/// Wait until the backend is **provably** armed, then drain what arming made.
///
/// A watch is not live when `Watcher::new` returns. inotify arms inside the
/// syscall, so a sleep of any length was indistinguishable from correctness on
/// Linux; macOS FSEvents starts its stream on another thread, and a write that
/// lands before the stream is running emits **no event at all** — no timeout
/// downstream can recover it, which is why `detects_step_request_creation_at_
/// conv_repo_root` was the CI's one flaky beat (bl-1015).
///
/// So arming is not slept on, it is *observed*: rewrite a probe file on every
/// sample until the watcher reports it. The write is repeated rather than made
/// once because the pre-arming writes are exactly the ones that vanish, and one
/// event is all the evidence needed — the stream that delivered it is the same
/// stream the test's own write will go through.
pub(super) fn wait_quiet(watcher: &Watcher) {
    let probe = watcher.repo_root.join("steps/.arming-probe");
    fs::create_dir_all(probe.parent().unwrap()).unwrap();
    let armed = poll_until(
        || {
            fs::write(&probe, b"x").ok()?;
            watcher.tick().iter().any(|c| c.path == probe).then_some(())
        },
        WAIT_TIMEOUT,
        POLL_INTERVAL,
    );
    assert!(armed.is_some(), "the watcher never armed");
    // The probe file stays where it is: removing it would be one more event for
    // the next beat to trip over, and an unread file under `steps/` is invisible
    // to every assertion here. What is drained is the tail of arming's own
    // events — bounded and re-entered, because one settle that happened to be
    // short leaves the next beat reading this beat's noise.
    while poll_until(
        || (!watcher.tick().is_empty()).then_some(()),
        POLL_INTERVAL * 4,
        POLL_INTERVAL,
    )
    .is_some()
    {}
}

/// Poll until a change satisfying `pred` surfaces, returning that tick's full
/// change set (so callers can also assert on siblings, e.g. a coalesced
/// rename's dropped source). Empty on timeout.
pub(super) fn wait_for(watcher: &Watcher, pred: impl Fn(&Change) -> bool) -> Vec<Change> {
    wait_for_with(watcher, || {}, pred)
}

/// [`wait_for`] with a mutation re-made on every sample — for a beat whose
/// claim is *that* a change surfaces rather than how many times it was made.
pub(super) fn wait_for_with(
    watcher: &Watcher,
    mut make: impl FnMut(),
    pred: impl Fn(&Change) -> bool,
) -> Vec<Change> {
    let probe = || {
        make();
        let changes = watcher.tick();
        let found = changes.iter().any(&pred);
        found.then_some(changes)
    };
    poll_until(probe, WAIT_TIMEOUT, POLL_INTERVAL).unwrap_or_default()
}

/// Create `target`'s parent, watch `root`, write it, assert a `Touched` change.
pub(super) fn expect_touch(root: &Path, target: &Path) {
    fs::create_dir_all(target.parent().unwrap()).unwrap();
    let watcher = Watcher::new(root).unwrap();
    wait_quiet(&watcher);
    // Written on every sample, not once: the claim is that a write under the
    // root surfaces, and re-making the write costs nothing while making the
    // beat immune to a single delivery the backend drops under load (bl-1015).
    let changes = wait_for_with(
        &watcher,
        || fs::write(target, b"x").unwrap(),
        |c| c.path == *target,
    );
    let hit = changes.iter().find(|c| c.path == *target).expect("event");
    assert_eq!(hit.kind, ChangeKind::Touched);
}
