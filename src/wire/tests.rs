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

/// **A box with no certificates founds its own** (REMOTE §8 as amended,
/// bl-ae05) and listens on loopback. Absence stopped being the off switch the
/// day the window became a client of this listener: a window with no listener
/// is a window that paints nothing, and REMOTE §8 has already rejected both
/// ways around that.
///
/// The world names its own port ([`ephemeral`](crate::test_support::wire::ephemeral))
/// and no certificate, so the mint is what is under test and the operator's
/// running window keeps the one port the constant names.
#[test]
fn an_unprovisioned_box_founds_its_own_loopback_wire() {
    let _guard = crate::test_support::spawn_guard();
    let tmp = TempDir::new().expect("tmp");
    let world = crate::test_support::world_under(tmp.path());
    crate::test_support::wire::ephemeral(&world);
    let listener = listen(&world, Arc::new(Silent), Presence::default()).expect("listening");
    assert!(
        listener.address().starts_with("127.0.0.1:"),
        "loopback only"
    );
    // Every end the box needs, the window's among them.
    for role in material::LEAVES {
        assert!(
            material::read(&world, role).expect("readable").is_some(),
            "{role:?} is provisioned"
        );
    }
}

/// A box whose material directory cannot even be MADE keeps its engine and
/// says why: the mint is a capability, and a box without `openssl` or without a
/// writable data root is the one place absence is still the answer.
#[test]
fn a_box_the_mint_cannot_provision_keeps_its_engine() {
    let _guard = crate::test_support::spawn_guard();
    let tmp = TempDir::new().expect("tmp");
    let world = crate::test_support::world_under(tmp.path());
    let dir = material::dir(&world);
    std::fs::create_dir_all(dir.parent().expect("root")).expect("root");
    std::fs::write(&dir, b"a file where the directory goes").expect("block it");
    assert!(listen(&world, Arc::new(Silent), Presence::default()).is_none());
}

/// A leaf the mint can replace is replaced: the CA key is here, so a box that
/// lost half a leaf heals rather than losing its wire.
#[test]
fn a_leaf_the_mint_can_replace_is_replaced() {
    let _guard = crate::test_support::spawn_guard();
    let tmp = TempDir::new().expect("tmp");
    let world = crate::test_support::world_under(tmp.path());
    let dir = material::dir(&world);
    mint(&dir);
    std::fs::remove_file(dir.join("server.key")).expect("rm");
    assert!(listen(&world, Arc::new(Silent), Presence::default()).is_some());
}

/// A provisioned box listens, on the address its material names.
#[test]
fn a_provisioned_box_listens() {
    let _guard = crate::test_support::spawn_guard();
    let tmp = TempDir::new().expect("tmp");
    let world = crate::test_support::world_under(tmp.path());
    mint(&material::dir(&world));
    let listener = listen(&world, Arc::new(Silent), Presence::default()).expect("listening");
    assert!(listener.address().starts_with("127.0.0.1:"));
}

/// Half a provisioning the mint **cannot** heal — an operator's anchor with no
/// CA key beside it — is warned about and still not a listener: silently
/// degrading to no encryption is the one failure this design excludes, and
/// replacing an operator's trust root is the other.
#[test]
fn a_half_provisioned_box_the_mint_cannot_heal_does_not_listen() {
    let _guard = crate::test_support::spawn_guard();
    let tmp = TempDir::new().expect("tmp");
    let world = crate::test_support::world_under(tmp.path());
    let dir = material::dir(&world);
    mint(&dir);
    std::fs::remove_file(dir.join(crate::wire::provision::CA_KEY)).expect("rm");
    std::fs::remove_file(dir.join("server.key")).expect("rm");
    assert!(listen(&world, Arc::new(Silent), Presence::default()).is_none());
}

/// The address the window dials: loopback at the port the listener really
/// bound, whatever the file said the engine answers to.
#[test]
fn the_window_dials_loopback_at_the_bound_port() {
    assert_eq!(loopback("0.0.0.0:7737"), "127.0.0.1:7737");
    assert_eq!(loopback("engine.example.com:9"), "127.0.0.1:9");
    assert_eq!(loopback("nonsense"), "127.0.0.1:");
}

/// An engine that cannot bind carries on without a wire: a listener is a
/// capability, and losing it is not losing the engine.
#[test]
fn an_unbindable_address_leaves_the_engine_running() {
    let _guard = crate::test_support::spawn_guard();
    let tmp = TempDir::new().expect("tmp");
    let world = crate::test_support::world_under(tmp.path());
    let dir = material::dir(&world);
    mint(&dir);
    std::fs::write(dir.join(material::ADDRESS), "256.256.256.256:1").expect("write");
    assert!(listen(&world, Arc::new(Silent), Presence::default()).is_none());
}
