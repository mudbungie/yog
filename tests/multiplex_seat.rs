//! End-to-end drive of the REMOTE §9.5 `seat` and §5 `tool-host` multiplex arms
//! through
//! `yog::multiplex::dispatch` — `yog seat '<json>'` composes the world, reads
//! this machine's wire material out of it, and sends the envelope to an engine
//! over mTLS.
//!
//! **This test binary owns its process environment** (the
//! `tests/multiplex_gesture.rs` precedent): the arm composes the world from
//! `$HOME`/`$XDG_*` live at the process boundary, so this file mutates its own
//! env — lawful exactly here, one `#[test]`, its own process.
//!
//! The engine end is a real listener over freshly minted material: certificates
//! are the operator's out-of-channel act (REMOTE §1.4) and are **never**
//! committed, so the test performs the same act with the same tool.

#![allow(clippy::unwrap_used)]

use std::path::Path;
use std::sync::Arc;
use yog::multiplex::dispatch;
use yog::registry::presence::Presence;
use yog::wire::server::{Answerer, Listener};

fn set(key: &str, value: &Path) {
    // SAFETY: single-threaded env mutation before any reader thread exists
    // (module doc; the listener thread below only reads the filesystem).
    unsafe { std::env::set_var(key, value) };
}

fn argv(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|s| (*s).to_string()).collect()
}

/// An engine stand-in: whatever it is asked, one reply frame saying yes.
struct Yes;

impl Answerer for Yes {
    fn answer(
        &self,
        _client: &yog::registry::Client,
        request: serde_json::Value,
    ) -> Box<dyn Iterator<Item = serde_json::Value>> {
        Box::new(std::iter::once(
            serde_json::json!({"ok": true, "kind": "balls", "asked": request}),
        ))
    }
}

#[test]
fn yog_seat_sends_over_the_wire_and_exits_on_the_reply() {
    let root = tempfile::tempdir().unwrap();
    set("HOME", &root.path().join("home"));
    set("XDG_DATA_HOME", &root.path().join("data"));
    set("XDG_STATE_HOME", &root.path().join("state"));
    set("XDG_CONFIG_HOME", &root.path().join("config"));
    let world = yog::world::compose(&yog::xdg::Env::from_env());
    let dir = yog::wire::material::dir(&world);

    // With nothing provisioned the seat refuses and names the remedy — it can
    // only ever point at the out-of-channel act, never perform it.
    assert_eq!(
        dispatch(&argv(&["yog", "seat", r#"{"op":"balls"}"#])),
        Some(2)
    );

    // The operator's act, the very verb the refusal names (bl-ae05): one
    // recipe, reached here exactly as `make wire-certs` reaches it.
    assert_eq!(
        yog::wire::provision::verb::perform(&yog::wire::provision::verb::Plan {
            dir: dir.clone(),
            address: "127.0.0.1:0".to_owned(),
            force: false,
        }),
        0,
        "the mint runs"
    );

    // An engine on the other end, bound where the material says.
    let material = yog::wire::material::read(&world, yog::wire::material::Role::Server)
        .unwrap()
        .unwrap();
    let listener = Listener::bind(&material, Arc::new(Yes), Presence::default()).unwrap();
    std::fs::write(dir.join("address"), listener.address()).unwrap();

    // A bad envelope never reaches the wire.
    assert_eq!(dispatch(&argv(&["yog", "seat", "not json"])), Some(2));

    // And a good one is answered, its verdict this process's exit.
    assert_eq!(
        dispatch(&argv(&["yog", "seat", r#"{"op":"balls"}"#])),
        Some(0),
        "the reply's verdict is the exit"
    );

    // The other wire client mode (REMOTE §5, bl-024b) composes the same world
    // through the same arm. This machine is provisioned but carries no
    // tool-host config, so it refuses before it dials anything — which is the
    // point: a tool host with nothing to offer has nothing to do.
    assert_eq!(dispatch(&argv(&["yog", "tool-host"])), Some(1));
    assert_eq!(
        dispatch(&argv(&["yog", "tool-host", "--follow"])),
        Some(2),
        "the mode takes no arguments"
    );
    // …but `--help` is not an argument, it is the bl-52ed invariant: the page
    // is answered above the router, exit 0, no world composed and nothing
    // dialled (bl-4667 — this exact ask used to be the Some(2) refusal above).
    assert_eq!(
        dispatch(&argv(&["yog", "tool-host", "--help"])),
        Some(0),
        "every command answers --help"
    );
}
