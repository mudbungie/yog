//! The engine's one table (STORIES S14-T8): a windowless boot answers a deposit.
//!
//! This is the whole headless claim in one test — no window, no display, no
//! face — and it is the test main.rs could never carry, which is why the
//! assembly moved out of it.

use super::*;
use crate::boundary::deposit;
use crate::test_support::{spawn_guard, world_under};
use crate::ui_state::Clock;
use crate::watch::NoRepaint;
use serde_json::json;
use std::time::{Duration, Instant};
use tempfile::tempdir;

/// A clock that states a chosen wall time — [`FakeClock`](crate::test_support::FakeClock)
/// stamps a placeholder, and the §5.2 sweep reads the stamp as a unix second.
struct AtClock(i64);

impl Clock for AtClock {
    fn now(&self) -> Instant {
        Instant::now()
    }
    fn stamp(&self) -> String {
        self.0.to_string()
    }
}

/// The rung: boot the engine into a hermetic world with **no window**, drop a
/// gesture in its inbox, and read the answer back. Every thread a running yog
/// has is up, and none of them is a frame.
#[test]
fn a_windowless_engine_answers_a_deposit_and_stops_on_drop() {
    let _guard = spawn_guard();
    let root = tempdir().unwrap();
    let world = world_under(root.path());
    // A staging dir older than the §5.2 horizon: the sweep is the engine's, so
    // booting one is what drops it.
    let stale = world.yog_stage_root().join("nonce");
    std::fs::create_dir_all(&stale).unwrap();
    let far_future = 1_000_000_000_000;
    let state_root = world.yog_state_root();
    deposit::deposit(&state_root, "e-1", &json!({"op": "attention"})).unwrap();

    let engine = Engine::boot(
        &world,
        &[],
        None,
        Arc::new(AtClock(far_future)),
        std::sync::Arc::new(NoRepaint),
    );
    assert!(!stale.exists(), "the §5.2 startup sweep is the engine's");
    assert_eq!(
        engine.model.ui_json_path(),
        state_root.join("ui.json"),
        "the model is rooted in the world it was booted into"
    );

    let deadline = Instant::now() + Duration::from_secs(10);
    let reply = loop {
        if let Some(reply) = deposit::read_reply(&state_root, "e-1") {
            break reply;
        }
        assert!(Instant::now() < deadline, "no face answered the deposit");
        std::thread::sleep(Duration::from_millis(10));
    };
    assert_eq!(reply["ok"], true);
    assert_eq!(reply["kind"], "attention", "the §6 queue, headless");
    assert_eq!(
        reply["rows"].as_array().map(Vec::len),
        Some(0),
        "an empty world asks nothing of anyone"
    );
    drop(engine); // every thread stops and joins — the Drop is the shutdown
}
