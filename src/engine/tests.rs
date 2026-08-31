//! The engine's one table (STORIES S14-T8): a boot answers a deposit.
//!
//! This is the whole server claim in one test — no window, no display, no
//! face — and it is the test main.rs could never carry, which is why the
//! assembly moved out of it. Everything the engine used to hand a window (the
//! asker, the poster, the follow lane, the §8.5 searcher) went with the window
//! (bl-7942); what a seat gets now is a socket, and the listener's own tests
//! are where that is proven.

use super::*;
use crate::boundary::deposit;
use crate::test_support::world_under;
use crate::ui_state::Clock;
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

/// The rung: boot the engine into a hermetic world, drop a gesture in its
/// inbox, and read the answer back. Every thread a running yog has is up, and
/// none of them is a frame.
#[test]
fn a_booted_engine_answers_a_deposit_and_stops_on_drop() {
    let root = tempdir().unwrap();
    let world = world_under(root.path());
    // A boot listens on the default request — `127.0.0.1:0`, a kernel-chosen
    // port (bl-dc14) — so nothing here contends with the operator's own
    // running window, and nothing needs seeding to avoid it.
    // A staging dir older than the §5.2 horizon: the sweep is the engine's, so
    // booting one is what drops it.
    let stale = world.yog_stage_root().join("nonce");
    std::fs::create_dir_all(&stale).unwrap();
    let far_future = 1_000_000_000_000;
    let state_root = world.yog_state_root();
    deposit::deposit(&state_root, "e-1", &json!({"op": "attention"})).unwrap();

    let engine = Engine::boot(&world, &[], Arc::new(AtClock(far_future)));
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
        assert!(Instant::now() < deadline, "the engine answered no deposit");
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

/// **An engine that cannot get its wire up runs anyway** (bl-dc14, narrowed by
/// bl-7942): the refusal is said on stderr — the unit's journal is where a
/// server's words go — and everything else boots. A deposit still converges
/// through the inbox, so only a seat is shut out, which is exactly the
/// difference between losing a capability and losing the engine.
#[test]
fn a_boot_whose_stated_address_cannot_bind_still_answers_the_inbox() {
    let root = tempdir().unwrap();
    let world = world_under(root.path());
    let dir = crate::wire::material::dir(&world);
    crate::test_support::wire::mint(&dir);
    // An address no socket can take, so the listener's bind is the thing that
    // fails — not the mint, whose own refusal is `wire::tests`'.
    std::fs::write(
        dir.join(crate::wire::material::ADDRESS),
        "256.256.256.256:1\n",
    )
    .unwrap();
    let state_root = world.yog_state_root();
    deposit::deposit(&state_root, "e-2", &json!({"op": "attention"})).unwrap();

    let engine = Engine::boot(&world, &[], Arc::new(AtClock(0)));
    let deadline = Instant::now() + Duration::from_secs(10);
    let reply = loop {
        if let Some(reply) = deposit::read_reply(&state_root, "e-2") {
            break reply;
        }
        assert!(
            Instant::now() < deadline,
            "a wireless engine still answers its own residents"
        );
        std::thread::sleep(Duration::from_millis(10));
    };
    assert_eq!(reply["ok"], true);
    drop(engine);
}
