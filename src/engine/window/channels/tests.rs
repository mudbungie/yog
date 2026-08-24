//! **A booted engine attaches to every channel it holds** (REMOTE §8.2,
//! bl-670c): its own, over loopback, and one per entry — each on that entry's
//! own material, each answered by its own asker, and each failing alone.
//!
//! Two real listeners on one box, which is the only way to make the claim: the
//! engine under test binds its own, and a second stands in for the host an
//! entry names. The stand-in is an [`Answerer`] rather than a second
//! `Engine::boot` because what is under test is the *channel set*, not the
//! host's scoping — a registration is the host operator's file (§1.4) and
//! nothing here could write one anyway.

use crate::boundary::reply::{Reply, Workspaces, WsRow};
use crate::boundary::{Gesture, Query, codec};
use crate::engine::Engine;
use crate::registry::presence::Presence;
use crate::test_support::wire::{EPHEMERAL, material, mint};
use crate::ui_state::Clock;
use crate::watch::NoRepaint;
use crate::wire::material::{ADDRESS, ANCHORS, Role};
use crate::wire::server::{Answerer, Listener};
use serde_json::Value;
use std::sync::Arc;
use tempfile::TempDir;

/// A clock that never moves — the engine's own fixture shape.
struct AtClock(i64);

impl Clock for AtClock {
    fn now(&self) -> std::time::Instant {
        std::time::Instant::now()
    }
    fn stamp(&self) -> String {
        self.0.to_string()
    }
}

/// A host that answers every gesture with one workspace row, named as *it*
/// names it.
struct Hosts(&'static str);

impl Answerer for Hosts {
    fn answer(
        &self,
        _client: &crate::registry::Client,
        _request: Value,
    ) -> Box<dyn Iterator<Item = Value>> {
        Box::new(std::iter::once(crate::boundary::reply::encode(
            &Reply::Workspaces(Workspaces {
                rows: vec![WsRow {
                    workspace: self.0.to_owned(),
                    kind: crate::binding::WorkspaceKind::Foreign,
                    attention: 3,
                    agents: 1,
                    running: true,
                    pinned: None,
                    config_tip: None,
                }],
                stale: None,
                growth: None,
            }),
        )))
    }
}

/// **The window's roster fills from every channel it holds.** The entry wears
/// its claim row from the moment it is provisioned (bl-028a); once its own
/// asker answers, that row carries the host's facts, named in this box's
/// spelling.
#[test]
fn every_channel_gets_an_asker_and_fills_its_own_slice() {
    let host = TempDir::new().expect("tmp");
    mint(host.path());
    let listener = Listener::bind(
        &material(host.path(), Role::Server, EPHEMERAL),
        Arc::new(Hosts("home")),
        Presence::default(),
    )
    .expect("bind");

    let root = TempDir::new().expect("tmp");
    let world = crate::test_support::world_under(root.path());
    let flat = crate::wire::material::dir(&world);
    provision_entry(
        &flat.join(crate::wire::entries::ENTRIES).join("cobalt"),
        host.path(),
        &crate::wire::loopback(&listener.address()),
        Some("home"),
    );

    let mut engine = Engine::boot(&world, &[], None, Arc::new(AtClock(0)), Arc::new(NoRepaint));
    let wire = engine.window_wire(&world).expect("every channel");
    assert!(
        engine.window_wire(&world).is_none(),
        "taken once, entry ends included"
    );
    assert!(
        awaited(&mut engine, "cobalt", 3),
        "the entry's own facts land"
    );
    drop(wire);
    drop(engine);
}

/// **A dead entry costs its own slice and nothing else.** Its channel refuses
/// with the entry's own sentence while the local channel answers on its own
/// thread — the isolation §8.2 asks for, stated as a fact about one roster.
#[test]
fn an_entry_that_cannot_be_dialled_leaves_the_local_channel_alone() {
    let root = TempDir::new().expect("tmp");
    let world = crate::test_support::world_under(root.path());
    let flat = crate::wire::material::dir(&world);
    std::fs::create_dir_all(flat.join(crate::wire::entries::ENTRIES).join("cobalt"))
        .expect("mkdir");

    let mut engine = Engine::boot(&world, &[], None, Arc::new(AtClock(0)), Arc::new(NoRepaint));
    let wire = engine.window_wire(&world).expect("every channel");
    assert!(
        engine.model.wire_refusal().is_none(),
        "one entry's refusal is never the whole shell's (bl-dc14)"
    );
    let about = codec::encode(&Gesture::Ask(Query::WorkspaceBalls {
        workspace: "cobalt".to_owned(),
    }));
    assert!(
        awaited_refusal(&mut engine, &about),
        "the entry answers its own sentence in place of every question"
    );
    assert!(
        awaited(&mut engine, "cobalt", 0),
        "and it still wears its row, with the zeros it honestly has"
    );
    drop(wire);
    drop(engine);
}

/// Write the four files `Role::Client` reads, copied from a minted directory,
/// plus the optional host-side name.
fn provision_entry(
    dir: &std::path::Path,
    from: &std::path::Path,
    address: &str,
    host: Option<&str>,
) {
    std::fs::create_dir_all(dir).expect("mkdir");
    for name in [ANCHORS, "client.pem", "client.key"] {
        std::fs::copy(from.join(name), dir.join(name)).expect("copy");
    }
    std::fs::write(dir.join(ADDRESS), address).expect("write");
    if let Some(name) = host {
        std::fs::write(dir.join(crate::wire::entries::WORKSPACE), name).expect("write");
    }
}

/// Settle frames until the union roster carries `leaf` with `attention`,
/// bounded — a wait on another thread's publish, never a claim about timing.
fn awaited(engine: &mut Engine, leaf: &str, attention: usize) -> bool {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
    loop {
        engine.model.refresh();
        let seen = engine
            .model
            .wire_roster()
            .into_iter()
            .any(|r| r.row.workspace == leaf && r.row.attention == attention);
        if seen {
            return true;
        }
        if std::time::Instant::now() > deadline {
            return false;
        }
        std::thread::yield_now();
    }
}

/// The same wait, for a question that must land a refusal.
fn awaited_refusal(engine: &mut Engine, question: &Value) -> bool {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
    loop {
        engine.model.refresh();
        if matches!(engine.model.wire_ask(question), Some(Err(_))) {
            return true;
        }
        if std::time::Instant::now() > deadline {
            return false;
        }
        std::thread::yield_now();
    }
}
