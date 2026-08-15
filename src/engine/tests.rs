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

/// **The window's read path, end to end through the engine** (REMOTE §1.2 as
/// ruled 2026-08-14, bl-ae05). A boot on an unprovisioned box founds its own
/// loopback trust root, and the face it hands an asker to is a client of that
/// listener presenting the window leaf — a real socket, a real handshake, a
/// real certificate.
#[test]
fn a_booted_engine_hands_its_window_a_seat_on_its_own_wire() {
    let _guard = spawn_guard();
    let root = tempdir().unwrap();
    let world = world_under(root.path());
    let mut engine = Engine::boot(
        &world,
        &[],
        None,
        Arc::new(AtClock(0)),
        std::sync::Arc::new(NoRepaint),
    );
    let mut asker = engine
        .asker(&world)
        .expect("a seat on the engine's own wire");
    assert!(
        engine.asker(&world).is_none(),
        "one asker per engine: the link end is taken, not shared"
    );

    // The frame's own call, in `refresh`'s order: settle, then ask. The second
    // frame declares it, the asker answers it, the third paints it.
    let question = crate::boundary::codec::encode(&crate::boundary::Gesture::Ask(
        crate::boundary::Query::Workspaces,
    ));
    engine.model.refresh();
    assert!(engine.model.wire_ask(&question).is_none());
    engine.model.refresh();
    engine.model.wire_ask(&question);
    assert_eq!(asker.pass(), 1, "one standing question, asked");
    engine.model.refresh();
    let Some(Ok(crate::boundary::reply::Reply::Workspaces(rows))) =
        engine.model.wire_ask(&question)
    else {
        panic!("the frame paints a decoded reply that crossed the wire");
    };
    assert!(rows.is_empty(), "an empty world enumerates nothing");
    drop(engine);
}

/// **The window's two halves are one hand-over** (REMOTE §9.8, bl-4841): a boot
/// spawns both or neither, and a second call answers `None` — one asker and one
/// poster per engine, which for the act path is load-bearing, an act having to
/// be sent exactly once.
#[test]
fn a_window_takes_both_halves_of_the_wire_once() {
    let _guard = spawn_guard();
    let root = tempdir().unwrap();
    let world = world_under(root.path());
    let mut engine = Engine::boot(
        &world,
        &[],
        None,
        Arc::new(AtClock(0)),
        std::sync::Arc::new(NoRepaint),
    );
    let wire = engine.window_wire(&world).expect("both halves");
    assert!(engine.window_wire(&world).is_none(), "taken, never shared");
    drop(wire);
    drop(engine);
}
