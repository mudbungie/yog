//! Where pending offers live: the single use, the expiry, and the hold.

use crate::registry::offer::store::*;
use crate::registry::{Client, Grade};
use tempfile::tempdir;

fn record(name: &str, code: &str, expires: i64) -> Record {
    Record {
        name: name.to_owned(),
        workspace: "home".to_owned(),
        grade: Grade::Operator,
        code: code.to_owned(),
        expires,
        word: None,
        claimed: None,
        declined: false,
    }
}

fn client(name: &str) -> Client {
    Client::parse(name).expect("identity")
}

/// A minted offer is pending until its code expires, and an expired one is
/// deleted by the read that noticed — nothing sweeps.
#[test]
fn an_open_offer_is_pending_until_it_expires() {
    let root = tempdir().expect("tmp");
    assert!(pending(root.path(), 0).is_empty(), "nothing minted");
    mint(root.path(), &record("phone", "AAAAAAAA", 100)).expect("mint");
    mint(root.path(), &record("tablet", "BBBBBBBB", 200)).expect("mint");
    assert_eq!(pending(root.path(), 99).len(), 2);
    assert!(
        !held(root.path(), &client("phone")),
        "an open offer holds nobody"
    );

    let later = pending(root.path(), 100);
    assert_eq!(
        later.len(),
        1,
        "the code's last second is not a second of life"
    );
    assert_eq!(later[0].name, "tablet");
    assert_eq!(
        take(root.path(), "AAAAAAAA", 0),
        None,
        "and it is gone, not hidden"
    );
}

/// Minting again under one name replaces the code: the old one claims nothing.
#[test]
fn minting_again_replaces_an_open_offer() {
    let root = tempdir().expect("tmp");
    mint(root.path(), &record("phone", "AAAAAAAA", 100)).expect("mint");
    mint(root.path(), &record("phone", "CCCCCCCC", 100)).expect("again");
    assert_eq!(take(root.path(), "AAAAAAAA", 0), None);
    assert!(take(root.path(), "CCCCCCCC", 0).is_some());
}

/// **A code is spent exactly once**: the taker holds the name, a second
/// claimant of the same code finds nothing, and so do an unknown code and an
/// expired one — one answer for all three.
#[test]
fn a_code_is_taken_once() {
    let root = tempdir().expect("tmp");
    mint(root.path(), &record("phone", "AAAAAAAA", 100)).expect("mint");
    assert_eq!(take(root.path(), "ZZZZZZZZ", 0), None, "unknown");
    assert_eq!(take(root.path(), "AAAAAAAA", 100), None, "expired");
    mint(root.path(), &record("phone", "AAAAAAAA", 100)).expect("mint");

    let taken = take(root.path(), "AAAAAAAA", 0).expect("the one taker");
    assert_eq!(taken.name, "phone");
    assert!(held(root.path(), &client("phone")));
    assert_eq!(take(root.path(), "AAAAAAAA", 0), None, "claimed");
    assert_eq!(pending(root.path(), 0).len(), 1, "still pending, now held");
}

/// A claim that could not be signed puts the offer back for another try.
#[test]
fn a_taken_offer_can_be_restored() {
    let root = tempdir().expect("tmp");
    mint(root.path(), &record("phone", "AAAAAAAA", 100)).expect("mint");
    let taken = take(root.path(), "AAAAAAAA", 0).expect("taken");
    restore(root.path(), &taken);
    assert!(!held(root.path(), &client("phone")));
    assert!(
        take(root.path(), "AAAAAAAA", 0).is_some(),
        "claimable again"
    );
}

/// A settled record reads back whole; a declined one is held and never listed;
/// a removed one is forgotten.
#[test]
fn a_held_offer_is_settled_declined_or_removed() {
    let root = tempdir().expect("tmp");
    mint(root.path(), &record("phone", "AAAAAAAA", 100)).expect("mint");
    let taken = take(root.path(), "AAAAAAAA", 0).expect("taken");
    let claimed = Record {
        word: Some("K7M2QX".to_owned()),
        claimed: Some(7),
        ..taken
    };
    settle(root.path(), &claimed).expect("settle");
    // A claimed offer outlives its code's expiry: it waits for the operator.
    assert_eq!(pending(root.path(), 1_000), vec![claimed.clone()]);

    let declined = Record {
        declined: true,
        ..claimed
    };
    settle(root.path(), &declined).expect("decline");
    assert!(pending(root.path(), 0).is_empty(), "never listed");
    assert!(held(root.path(), &client("phone")), "and held for good");
    assert!(refusal(&client("phone")).contains("/approve phone"));

    remove(root.path(), "phone");
    assert!(!held(root.path(), &client("phone")));
}

/// A file that is not a record is no offer, and a store that cannot be written
/// says which file.
#[test]
fn debris_is_no_offer_and_an_unwritable_store_refuses() {
    let root = tempdir().expect("tmp");
    mint(root.path(), &record("phone", "AAAAAAAA", 100)).expect("mint");
    let open = root.path().join(DIR).join("open");
    std::fs::write(open.join("junk"), "not json").expect("debris");
    std::fs::write(open.join("half"), r#"{"name":"half"}"#).expect("debris");
    assert_eq!(pending(root.path(), 0).len(), 1);

    let blocked = tempdir().expect("tmp");
    std::fs::write(blocked.path().join(DIR), "a file where the directory goes").expect("block");
    let refusal = mint(blocked.path(), &record("phone", "AAAAAAAA", 100)).expect_err("refused");
    assert!(refusal.contains("phone"), "{refusal}");
    assert_eq!(take(blocked.path(), "AAAAAAAA", 0), None);
}
