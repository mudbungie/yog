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
/// Nothing is seeded first (bl-dc14 dissolved bl-4c50's fixture seed): the
/// default request is `127.0.0.1:0` — a kernel-chosen port — so the bare boot
/// under test is the same bare boot a real box performs, and neither takes the
/// port any running yog holds.
#[test]
fn an_unprovisioned_box_founds_its_own_loopback_wire() {
    let tmp = TempDir::new().expect("tmp");
    let world = crate::test_support::world_under(tmp.path());
    let listener = listen(&world, Arc::new(Silent), Presence::default()).expect("listening");
    assert!(
        listener.address().starts_with("127.0.0.1:"),
        "loopback only"
    );
    assert!(
        !listener.address().ends_with(":0"),
        "the bound answer to the `:0` request, never the request itself"
    );
    // Every end the box needs, the window's among them.
    for role in material::LEAVES {
        assert!(
            material::read(&world, role).expect("readable").is_some(),
            "{role:?} is provisioned"
        );
    }
}

/// **Two engines on one box each get their own wire** (I0, bl-dc14): the
/// default request is `:0`, so a second listener — a second world, or a second
/// window on this one — binds its own kernel-chosen port instead of losing a
/// race for a process-global one. Same world here, which is the harder case:
/// one address file, two live listeners, two distinct ports.
#[test]
fn two_engines_on_one_box_each_get_their_own_wire() {
    let tmp = TempDir::new().expect("tmp");
    let world = crate::test_support::world_under(tmp.path());
    let first = listen(&world, Arc::new(Silent), Presence::default()).expect("the first wire");
    let second = listen(&world, Arc::new(Silent), Presence::default()).expect("the second wire");
    assert_ne!(
        first.address(),
        second.address(),
        "each instance owns a distinct endpoint"
    );
}

/// A *stated* port another process holds is a refusal that names the bind
/// (bl-dc14): an operator-written address is intent, so the engine does not
/// slide to another port behind it — it says exactly what could not be had,
/// and the window paints that sentence rather than opening inert.
#[test]
fn a_stated_port_another_process_holds_is_a_refusal_that_names_it() {
    let tmp = TempDir::new().expect("tmp");
    let world = crate::test_support::world_under(tmp.path());
    let dir = material::dir(&world);
    mint(&dir);
    // A throwaway listener this test owns stands in for the other instance.
    let holder = std::net::TcpListener::bind("127.0.0.1:0").expect("a port of our own");
    let held = holder.local_addr().expect("bound").to_string();
    std::fs::write(dir.join(material::ADDRESS), format!("{held}\n")).expect("write");
    let refusal = listen(&world, Arc::new(Silent), Presence::default())
        .err()
        .expect("the held port refuses");
    assert!(refusal.contains(&format!("bind {held}")), "{refusal}");
}

/// A box whose material directory cannot even be MADE keeps its engine and
/// says why: the mint is a capability, and a box without `openssl` or without a
/// writable data root is the one place absence is still the answer.
#[test]
fn a_box_the_mint_cannot_provision_keeps_its_engine() {
    let tmp = TempDir::new().expect("tmp");
    let world = crate::test_support::world_under(tmp.path());
    let dir = material::dir(&world);
    std::fs::create_dir_all(dir.parent().expect("root")).expect("root");
    std::fs::write(&dir, b"a file where the directory goes").expect("block it");
    let refusal = listen(&world, Arc::new(Silent), Presence::default())
        .err()
        .expect("a box the mint cannot provision refuses");
    assert!(refusal.contains("wire"), "{refusal}");
}

/// A leaf the mint can replace is replaced: the CA key is here, so a box that
/// lost half a leaf heals rather than losing its wire.
#[test]
fn a_leaf_the_mint_can_replace_is_replaced() {
    let tmp = TempDir::new().expect("tmp");
    let world = crate::test_support::world_under(tmp.path());
    let dir = material::dir(&world);
    mint(&dir);
    std::fs::remove_file(dir.join("server.key")).expect("rm");
    assert!(listen(&world, Arc::new(Silent), Presence::default()).is_ok());
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

/// Half a provisioning the mint **cannot** heal — an operator's anchor with no
/// CA key beside it — is warned about and still not a listener: silently
/// degrading to no encryption is the one failure this design excludes, and
/// replacing an operator's trust root is the other.
#[test]
fn a_half_provisioned_box_the_mint_cannot_heal_does_not_listen() {
    let tmp = TempDir::new().expect("tmp");
    let world = crate::test_support::world_under(tmp.path());
    let dir = material::dir(&world);
    mint(&dir);
    std::fs::remove_file(dir.join(crate::wire::provision::CA_KEY)).expect("rm");
    std::fs::remove_file(dir.join("server.key")).expect("rm");
    let refusal = listen(&world, Arc::new(Silent), Presence::default())
        .err()
        .expect("half-provisioned refuses");
    assert!(refusal.contains("half-provisioned"), "{refusal}");
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
    let tmp = TempDir::new().expect("tmp");
    let world = crate::test_support::world_under(tmp.path());
    let dir = material::dir(&world);
    mint(&dir);
    std::fs::write(dir.join(material::ADDRESS), "256.256.256.256:1").expect("write");
    assert!(listen(&world, Arc::new(Silent), Presence::default()).is_err());
}

/// A mint failure on a box whose server material is already readable is a
/// warning, not a refusal: the wire the box had is the wire it keeps. The
/// failure is manufactured by seating a directory where a missing leaf's key
/// goes, so the re-mint of that one leaf cannot write.
#[test]
fn a_mint_failure_with_readable_material_keeps_the_wire() {
    let tmp = TempDir::new().expect("tmp");
    let world = crate::test_support::world_under(tmp.path());
    let dir = material::dir(&world);
    mint(&dir);
    std::fs::remove_file(dir.join("client.pem")).expect("rm");
    std::fs::remove_file(dir.join("client.key")).expect("rm");
    std::fs::create_dir(dir.join("client.key")).expect("a directory where the key goes");
    let listener = listen(&world, Arc::new(Silent), Presence::default()).expect("the wire is kept");
    assert!(listener.address().starts_with("127.0.0.1:"));
}
