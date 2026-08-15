//! Whether an engine has a wire at all.

use super::*;
use crate::registry::presence::Presence;
use crate::test_support::wire::mint;
use serde_json::{Value, json};
use tempfile::TempDir;

struct Silent;

impl server::Answerer for Silent {
    fn answer(&self, _client: &crate::registry::Client, _request: Value) -> Vec<Value> {
        vec![json!({"ok": true})]
    }
}

/// A box with no certificates has no wire, and nothing is said about it:
/// absence is the off switch, and removing the directory deleted config rather
/// than editing code.
#[test]
fn an_unprovisioned_box_listens_on_nothing() {
    let tmp = TempDir::new().expect("tmp");
    let world = crate::test_support::world_under(tmp.path());
    assert!(listen(&world, Arc::new(Silent), Presence::default()).is_none());
}

/// A provisioned box listens, on the address its material names.
#[test]
fn a_provisioned_box_listens() {
    let tmp = TempDir::new().expect("tmp");
    let world = crate::test_support::world_under(tmp.path());
    mint(&material::dir(&world));
    let listener = listen(&world, Arc::new(Silent), Presence::default()).expect("listening");
    assert!(listener.address().starts_with("127.0.0.1:"));
}

/// Half a provisioning is warned about and still not a listener — silently
/// degrading to no encryption is the one failure this design excludes.
#[test]
fn a_half_provisioned_box_does_not_listen() {
    let tmp = TempDir::new().expect("tmp");
    let world = crate::test_support::world_under(tmp.path());
    let dir = material::dir(&world);
    mint(&dir);
    std::fs::remove_file(dir.join("server.key")).expect("rm");
    assert!(listen(&world, Arc::new(Silent), Presence::default()).is_none());
}

/// An engine that cannot bind carries on without a wire: a listener is a
/// capability, and losing it is not losing the engine.
#[test]
fn an_unbindable_address_leaves_the_engine_running() {
    let tmp = TempDir::new().expect("tmp");
    let world = crate::test_support::world_under(tmp.path());
    let dir = material::dir(&world);
    mint(&dir);
    std::fs::write(dir.join(material::ADDRESS), "256.256.256.256:1").expect("write");
    assert!(listen(&world, Arc::new(Silent), Presence::default()).is_none());
}
