//! Presence: RAII, refcounted, and the whole of what "connected right now"
//! means (REMOTE §5).

use super::*;

fn client(name: &str) -> Client {
    Client::parse(name).expect("a usable identity")
}

/// A box with no connections says so — the general path with no input.
#[test]
fn a_fresh_map_holds_nobody() {
    let presence = Presence::default();
    assert!(presence.live().is_empty());
    assert!(!presence.is_live("phone"));
}

/// Entering and leaving is the whole protocol, and leaving is a drop: there is
/// no verb to forget.
#[test]
fn a_connection_is_present_while_its_guard_lives() {
    let presence = Presence::default();
    {
        let _live = presence.enter(&client("phone"));
        assert!(presence.is_live("phone"));
        assert_eq!(presence.live(), BTreeSet::from(["phone".to_owned()]));
    }
    assert!(!presence.is_live("phone"), "the guard's drop released it");
    assert!(presence.live().is_empty());
}

/// **Two seats of one client**: the second closing must not unsay the first,
/// which is why the map counts rather than holds a set.
#[test]
fn a_second_connection_from_one_client_outlives_the_first() {
    let presence = Presence::default();
    let phone = client("phone");
    let first = presence.enter(&phone);
    let second = presence.enter(&phone);
    drop(first);
    assert!(presence.is_live("phone"), "one connection still stands");
    drop(second);
    assert!(!presence.is_live("phone"));
}

/// The handle is shared, not copied: what the listener enters is what an answer
/// reads.
#[test]
fn a_cloned_handle_sees_the_same_connections() {
    let presence = Presence::default();
    let reader = presence.clone();
    let _live = presence.enter(&client("laptop"));
    assert!(reader.is_live("laptop"));
    assert_eq!(reader.live(), BTreeSet::from(["laptop".to_owned()]));
}

/// Two clients are two identities, counted apart.
#[test]
fn identities_are_counted_apart() {
    let presence = Presence::default();
    let a = presence.enter(&client("phone"));
    let _b = presence.enter(&client("laptop"));
    assert_eq!(
        presence.live(),
        BTreeSet::from(["laptop".to_owned(), "phone".to_owned()])
    );
    drop(a);
    assert_eq!(presence.live(), BTreeSet::from(["laptop".to_owned()]));
}
