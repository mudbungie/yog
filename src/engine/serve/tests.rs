//! The whole windowless face, driven to completion — the test `main.rs` could
//! never carry, and the one bl-269a made possible at all.
//!
//! **This module is the suite's one owner of the process-wide stop flag.**
//! Every raise and every restore of it happens here, so no two tests can race
//! the one static; `engine::stop`'s own tests drive the loop on a flag they own
//! instead. One owner rather than a lock, which rule 7 keeps in `state.rs`
//! anyway. The refusal test below never raises the flag — a boot that refuses
//! never reaches the loop — but `serve` installs the disposition before it
//! boots, so it hands that back for the same reason.

use super::*;
use crate::cli_outbound::sys;
use crate::engine::stop;
use crate::test_support::world_under;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};
use tempfile::tempdir;

/// `yog serve`, stopped. Catch the signal, run the handler as the kernel would
/// (see `sys::on_term` on why this is a call and not a delivered signal), then
/// boot every thread a running yog has and watch the face see the flag, return
/// from its loop, and drop the engine — stopping and joining all of them.
///
/// Before bl-269a this function's loop had no exit at all, which is why the
/// face lived in the coverage-excluded entry file and no test could reach it.
#[test]
fn serve_returns_once_a_stop_is_asked_for() {
    let root = tempdir().unwrap();
    let ambient = world_under(root.path());

    stop::catch();
    assert!(!stop::requested(), "nothing has asked for a stop yet");
    sys::on_term(libc::SIGTERM);
    assert!(
        stop::requested(),
        "the handler raises the flag and does nothing else"
    );

    let started = Instant::now();
    assert_eq!(
        Engine::serve(&ambient, &[]),
        0,
        "an engine that booted and stopped exits clean (bl-1d9b)"
    );
    assert!(
        started.elapsed() < Duration::from_secs(30),
        "the drop stops and joins every thread; it must not park on any of them"
    );
    assert!(
        crate::world::layout(&ambient).tools.join("bl").exists(),
        "the face seeds the world it is about to hand to every child it spawns"
    );

    // Hand the disposition back so the rest of the suite dies on a signal as it
    // always did, and leave the flag exactly as this test found it.
    sys::term_disposition(false);
    sys::term_flag().store(false, Ordering::SeqCst);
    assert!(!stop::requested());
}

/// **A boot that cannot take the world exits non-zero, saying so** (bl-1d9b).
/// The face owns stderr and the exit code together, so this is where the two
/// refusals become an answer a supervisor, a script and a human all read the
/// same way — and `NO_ENGINE` is not `0`, which is the whole of what the old
/// behaviour could not tell them.
#[test]
fn a_face_that_cannot_take_the_world_exits_no_engine() {
    let root = tempdir().unwrap();
    let ambient = world_under(root.path());
    // Another engine holds this world: the lock, taken exactly as a boot takes
    // it, and held across the call.
    let world = crate::world::compose(&ambient);
    let _held = crate::engine::sole::take(&world.yog_state_root()).expect("the first engine");
    assert_eq!(
        Engine::serve(&ambient, &[]),
        super::NO_ENGINE,
        "a second yog on one world is not an engine"
    );
    // `serve` catches the signal before it boots, so hand the disposition back
    // even though this face never parked.
    sys::term_disposition(false);
}
