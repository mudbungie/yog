//! Presence: RAII, one entry per live connection, and the whole of what
//! "connected right now" means (REMOTE §5) — plus what each connection said it
//! can spell (REMOTE §3.2).

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
        let _live = presence.enter(&client("phone"), 18);
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
    let first = presence.enter(&phone, 18);
    let second = presence.enter(&phone, 18);
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
    let _live = presence.enter(&client("laptop"), 18);
    assert!(reader.is_live("laptop"));
    assert_eq!(reader.live(), BTreeSet::from(["laptop".to_owned()]));
}

/// Two clients are two identities, counted apart.
#[test]
fn identities_are_counted_apart() {
    let presence = Presence::default();
    let a = presence.enter(&client("phone"), 18);
    let _b = presence.enter(&client("laptop"), 18);
    assert_eq!(
        presence.live(),
        BTreeSet::from(["laptop".to_owned(), "phone".to_owned()])
    );
    drop(a);
    assert_eq!(presence.live(), BTreeSet::from(["laptop".to_owned()]));
}

/// **A client holding nothing has nothing to say**, and that is the general
/// path with no input rather than a case: the same empty answer an identity
/// that never connected gives.
#[test]
fn a_client_with_no_connection_states_no_edition() {
    assert!(Presence::default().editions("phone").is_empty());
}

/// **What each connection stated, one entry apiece and ascending** (REMOTE
/// §3.2). Two seats of one client may be two builds, so the read is a list:
/// the newest would over-promise a call landing on the older one.
#[test]
fn two_seats_of_one_client_state_their_own_editions() {
    let presence = Presence::default();
    let phone = client("phone");
    let old = presence.enter(&phone, 18);
    let _new = presence.enter(&phone, 21);
    assert_eq!(presence.editions("phone"), vec![18, 21]);
    // And the guard that leaves takes ITS edition out, not whichever was
    // pushed last — a bag popped from the end would have answered `[18]`.
    drop(old);
    assert_eq!(presence.editions("phone"), vec![21]);
    assert!(presence.is_live("phone"));
}
