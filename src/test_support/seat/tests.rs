//! The seat's end: what it dials, what it verifies, and how it fails.

use super::*;
use crate::test_support::wire::{material, mint};
use crate::wire::material::Role;
use serde_json::json;
use tempfile::TempDir;

/// The name verified is read off the address and nothing else: a literal is an
/// IP identity, a hostname is a DNS one.
#[test]
fn the_server_name_comes_off_the_address() {
    assert_eq!(
        server_name("127.0.0.1:7737").expect("ip"),
        ServerName::IpAddress("127.0.0.1".parse::<IpAddr>().expect("ip").into())
    );
    assert_eq!(
        server_name("[::1]:7737").expect("ip"),
        ServerName::IpAddress("::1".parse::<IpAddr>().expect("ip").into())
    );
    assert_eq!(
        server_name("engine.example.com:7737").expect("dns"),
        ServerName::try_from("engine.example.com").expect("dns")
    );
    // A bare host with no port is still a name — the address file is the
    // operator's, and refusing it here would be a second opinion about it.
    assert_eq!(
        server_name("engine.example.com").expect("dns"),
        ServerName::try_from("engine.example.com").expect("dns")
    );
}

/// An address that names nothing verifiable refuses, naming it.
#[test]
fn an_unverifiable_address_refuses() {
    let err = server_name("not a host:1").expect_err("refused");
    assert!(err.contains("not a host:1"), "{err}");
}

/// A seat is built from material and remembers what it dials — no connection
/// is made, because a seat is a fact about this machine, not about an engine.
#[test]
fn opening_a_seat_dials_nothing() {
    let tmp = TempDir::new().expect("tmp");
    mint(tmp.path());
    let seat = Seat::open(&material(tmp.path(), Role::Client, "127.0.0.1:7737")).expect("seat");
    assert_eq!(seat.address(), "127.0.0.1:7737");
}

/// Material that will not build a configuration refuses at open.
#[test]
fn unusable_material_refuses_at_open() {
    let tmp = TempDir::new().expect("tmp");
    mint(tmp.path());
    std::fs::write(tmp.path().join("ca.pem"), "").expect("write");
    assert!(Seat::open(&material(tmp.path(), Role::Client, "127.0.0.1:7737")).is_err());
}

/// An address that builds a configuration but names nothing verifiable refuses
/// at open too — before a connection is attempted, not after.
#[test]
fn an_unverifiable_address_refuses_at_open() {
    let tmp = TempDir::new().expect("tmp");
    mint(tmp.path());
    assert!(Seat::open(&material(tmp.path(), Role::Client, "not a host:1")).is_err());
}

/// Nothing listening is a refusal naming the address, never a hang.
///
/// **Port zero, because it is the one port nothing can ever answer on**
/// (bl-e4c8). The beat used to bind `:0`, read back the port the kernel picked
/// and drop the listener, calling the port "certainly dead" — but a just-freed
/// ephemeral port is precisely the port the kernel hands the next binder, so on
/// a box running this suite in parallel another test takes it between the drop
/// and the dial. The connect then SUCCEEDS and the failure arrives from the
/// write instead (`send: Connection reset by peer`), which names no address:
/// the beat's premise was only ever true on an idle box.
///
/// Zero is not an ephemeral port that happens to be free — in a `bind` it is
/// the *request* meaning "pick one", so no socket on any box is ever listening
/// there and `connect` has nothing to reach. It needs no privilege to be
/// unbindable (unlike a port below 1024, which a suite running as root could
/// take) and no routing assumption (unlike a TEST-NET address, which refuses on
/// one box and hangs on another). Whatever errno the platform names it with,
/// the failure is the connect's and the connect's sentence carries the address.
#[test]
fn a_dead_address_refuses_naming_itself() {
    let tmp = TempDir::new().expect("tmp");
    mint(tmp.path());
    let address = "127.0.0.1:0";
    let seat = Seat::open(&material(tmp.path(), Role::Client, address)).expect("seat");
    let err = seat.ask(&json!({"op": "workspaces"})).expect_err("refused");
    assert!(err.contains(address), "{err}");
}

/// A peer that accepts and then says nothing TLS-shaped is a receive failure,
/// not an answer — the seat never invents one.
#[test]
fn a_peer_that_is_not_an_engine_fails_to_answer() {
    let tmp = TempDir::new().expect("tmp");
    mint(tmp.path());
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
    let address = listener.local_addr().expect("addr").to_string();
    let peer = std::thread::spawn(move || {
        let _ = listener.accept();
    });
    let seat = Seat::open(&material(tmp.path(), Role::Client, &address)).expect("seat");
    seat.ask(&json!({"op": "workspaces"}))
        .expect_err("no answer");
    peer.join().expect("peer");
}

/// Loopback at the port really bound, whatever the `address` file said.
#[test]
fn a_local_seat_dials_loopback_at_the_bound_port() {
    use super::loopback;
    assert_eq!(loopback("0.0.0.0:7737"), "127.0.0.1:7737");
    assert_eq!(loopback("engine.example.com:9"), "127.0.0.1:9");
    assert_eq!(loopback("nonsense"), "127.0.0.1:");
}
