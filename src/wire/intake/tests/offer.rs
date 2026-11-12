//! **The offer, end to end** (REMOTE §15.1, §15.2; bl-9ea6): a real listener
//! over the real intake. A device with no certificate claims a code through
//! the registration door with a key it made, dials the wire with the leaf it
//! was handed, is refused everything until an operator approves it, and is
//! answered after.

use super::*;
use crate::registry::offer::store::{self, Record};
use crate::registry::{Client, Grade};
use crate::test_support::wire::{Device, certless, registration};
use crate::wire::listener::Listener;
use crate::wire::material::{self, Role};

#[test]
fn a_device_registers_dials_is_held_and_is_answered_once_approved() {
    let tmp = tempdir().expect("tmp");
    let world = crate::test_support::world_under(tmp.path());
    let wire = material::dir(&world);
    crate::test_support::wire::mint(&wire);
    let root = tmp.path().join("state-root");
    let listener = Listener::bind(
        &crate::test_support::wire::material(&wire, Role::Server, "127.0.0.1:0"),
        Arc::new(intake_in(world, &root)),
        crate::registry::presence::Presence::default(),
    )
    .expect("bind");
    store::mint(
        &root,
        &Record {
            name: "phone".to_owned(),
            workspace: "home".to_owned(),
            grade: Grade::Operator,
            code: "ABCDEFGH".to_owned(),
            expires: i64::MAX,
            word: None,
            claimed: None,
            declined: false,
        },
    )
    .expect("an open offer");
    let anchors = wire.join(material::ANCHORS);
    let device = Device::new();

    // A wrong code is a bare 404, and so is the right one a second time.
    let post = |code: &str| {
        certless(
            &anchors,
            &listener.address(),
            &registration(code, &device.body()),
        )
    };
    assert!(post("ZZZZZZZZ").starts_with("HTTP/1.1 404 "), "unknown");
    let said = post("ABCDEFGH");
    assert!(said.starts_with("HTTP/1.1 200 OK\r\n"), "{said}");
    let (_, body) = said.split_once("\r\n\r\n").expect("a body");
    let claimed: Value = serde_json::from_str(body).expect("one JSON object");
    assert_eq!(claimed["name"], "phone");
    assert_eq!(
        post("ABCDEFGH"),
        post("ZZZZZZZZ"),
        "claimed reads as unknown"
    );

    // The device dials with what it was handed and its own key. The leaf
    // authenticates; it is held, so nothing it says is answered — a founding
    // gesture included, which a client merely registered nowhere may make.
    let phone = Client::parse("phone").expect("identity");
    let entry = tmp.path().join("device");
    let seat = crate::test_support::seat::Seat::open(&device.material(
        &entry,
        &claimed,
        &listener.address(),
    ))
    .expect("seat");
    for request in [
        json!({"op": "workspaces"}),
        json!({"op": "prepare", "workspace": "mine", "payload": {"kind": "bare"}}),
        json!({"op": "teleport"}),
    ] {
        let answer = seat.ask(&request).expect("answered");
        let refusal = answer[0]["error"].as_str().unwrap_or_default();
        assert!(refusal.contains("/approve phone"), "{answer:?}");
    }
    assert!(crate::registry::registered(&root, &phone).is_empty());
    // …and having spoken is what proves the offer.
    assert!(crate::registry::seen::read(&root, &phone).is_some());

    // Approval is the registration and the end of the hold.
    crate::registry::register(&root, &phone, "home").expect("approved");
    store::remove(&root, "phone");
    let answer = seat.ask(&json!({"op": "workspaces"})).expect("answered");
    assert_eq!(answer[0]["kind"], "workspaces", "{answer:?}");
}
