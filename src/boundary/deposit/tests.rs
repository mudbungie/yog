//! The deposit protocol's tables (§8.5): create-only delivery, the listing's
//! filters, claim-by-rename, and the reply files.

use super::*;
use serde_json::json;
use tempfile::tempdir;

#[test]
fn ids_are_filenames_and_nothing_more() {
    assert!(valid_id("1700-42"));
    assert!(valid_id("abc_DEF-9"));
    assert!(!valid_id(""));
    assert!(!valid_id(".hidden"));
    assert!(!valid_id("a/b"));
    assert!(!valid_id("a.json"));
}

/// bl-aa9f: minting is a reservation, not a guess. One seed hands out as many
/// ids as there are askers, and an id is spent for good — a claimed deposit or
/// an answered one never frees its name back for a second caller to be handed
/// the first one's reply.
#[test]
fn a_minted_id_is_won_from_the_world_and_never_handed_out_twice() {
    let root = tempdir().unwrap();
    let first = mint(root.path(), "1786491765-2").unwrap();
    let second = mint(root.path(), "1786491765-2").unwrap();
    assert_eq!(
        (first.as_str(), second.as_str()),
        ("1786491765-2-0", "1786491765-2-1")
    );
    assert!(
        read_reply(root.path(), &first).is_none(),
        "the reservation reads as not-yet-answered"
    );
    // Spend the first id all the way through the lifecycle, then re-mint: the
    // name must not come back.
    deposit(root.path(), &first, &json!({"op": "balls"})).unwrap();
    claim(root.path(), &first).unwrap();
    write_reply(root.path(), &first, &json!({"ok": true})).unwrap();
    assert_eq!(mint(root.path(), "1786491765-2").unwrap(), "1786491765-2-2");
    assert_eq!(
        mint(root.path(), "no/slash").unwrap_err().kind(),
        std::io::ErrorKind::InvalidInput,
        "a seed that is not a filename mints nothing"
    );
    // A world that cannot hold the reservation refuses it, rather than
    // retrying a name it will never win: an unmakeable replies dir, and a
    // name the filesystem itself will not take.
    assert!(mint(root.path(), &"a".repeat(300)).is_err());
    let blocked = tempdir().unwrap();
    std::fs::create_dir_all(gestures_dir(blocked.path())).unwrap();
    std::fs::write(gestures_dir(blocked.path()).join("replies"), b"not a dir").unwrap();
    assert!(mint(blocked.path(), "seed").is_err());
}

#[test]
fn a_deposit_is_create_only_and_never_replayed_in_place() {
    let root = tempdir().unwrap();
    let path = deposit(root.path(), "g-1", &json!({"op": "balls"})).unwrap();
    assert!(path.is_file());
    let again = deposit(root.path(), "g-1", &json!({"op": "balls"}));
    assert_eq!(again.unwrap_err().kind(), std::io::ErrorKind::AlreadyExists);
    let bad = deposit(root.path(), "no/slash", &json!({}));
    assert_eq!(bad.unwrap_err().kind(), std::io::ErrorKind::InvalidInput);
}

#[test]
fn pending_lists_real_deposits_in_one_order_and_nothing_else() {
    let root = tempdir().unwrap();
    assert!(pending(root.path()).is_empty(), "no inbox is the empty set");
    deposit(root.path(), "b", &json!({})).unwrap();
    deposit(root.path(), "a", &json!({})).unwrap();
    let dir = gestures_dir(root.path());
    std::fs::write(dir.join(".c.json.tmp"), b"{}").unwrap(); // dotfile temp
    std::fs::write(dir.join(".dot.json"), b"{}").unwrap(); // dot-named non-id
    std::fs::write(dir.join("noise.txt"), b"x").unwrap(); // wrong extension
    std::fs::create_dir(dir.join("d.json")).unwrap(); // a directory, not a file
    let ids: Vec<String> = pending(root.path()).into_iter().map(|(id, _)| id).collect();
    assert_eq!(
        ids,
        ["a", "b"],
        "name-ordered (I9), temps and noise unlisted"
    );
}

#[test]
fn the_claim_is_the_rename_and_losing_the_race_is_benign() {
    let root = tempdir().unwrap();
    deposit(root.path(), "g-1", &json!({"op": "balls"})).unwrap();
    let held = claim(root.path(), "g-1").unwrap();
    assert!(held.path().is_file());
    assert!(
        pending(root.path()).is_empty(),
        "a claimed deposit is off the list"
    );
    assert!(
        claim(root.path(), "g-1").is_err(),
        "the loser gets the error"
    );
}

/// bl-d1f1: the claim is **held** — an OS file lock taken before the rename
/// and released only when the claimant lets go (or dies, which is the same
/// release to the kernel). While it is held, the claimed file reads as work in
/// flight; the moment it is dropped, as debris a sweep may answer.
#[test]
fn a_claim_is_lock_held_for_the_claimants_life_and_unheld_after() {
    let root = tempdir().unwrap();
    deposit(root.path(), "g-1", &json!({"op": "balls"})).unwrap();
    let held = claim(root.path(), "g-1").unwrap();
    assert!(
        !unheld(&held.path()),
        "a live claim is locked — never debris"
    );
    let path = held.path();
    drop(held);
    assert!(unheld(&path), "a dropped claim is debris, tellably");
    assert!(
        !unheld(&path.with_file_name("nope.json")),
        "a file that cannot be opened is nobody's debris"
    );
}

/// bl-d1f1: the lock is the first race, ahead of the rename — a rival holding
/// the deposit's lock means the claim is already someone's, and the loser
/// moves on exactly as a lost rename always meant.
#[test]
fn a_deposit_another_holds_the_lock_on_is_not_claimable() {
    let root = tempdir().unwrap();
    let path = deposit(root.path(), "g-1", &json!({"op": "balls"})).unwrap();
    let rival = std::fs::File::open(&path).unwrap();
    rival.try_lock().unwrap();
    let lost = claim(root.path(), "g-1").unwrap_err();
    assert_eq!(lost.kind(), std::io::ErrorKind::WouldBlock);
    assert!(
        path.is_file(),
        "the loser renamed nothing — the deposit stays where it was"
    );
}

/// bl-d1f1: a claimed id is spent. Re-depositing under it was the old doc's
/// own recovery advice ("re-deposit to re-run") and is exactly the unsafe
/// path — the first run is at best in doubt — so it refuses mechanically.
#[test]
fn a_claimed_id_is_spent_and_a_re_deposit_under_it_refuses() {
    let root = tempdir().unwrap();
    deposit(root.path(), "g-1", &json!({"op": "balls"})).unwrap();
    let held = claim(root.path(), "g-1").unwrap();
    drop(held);
    let again = deposit(root.path(), "g-1", &json!({"op": "balls"}));
    assert_eq!(again.unwrap_err().kind(), std::io::ErrorKind::AlreadyExists);
}

#[test]
fn claimed_lists_the_taken_deposits_and_an_empty_world_is_the_empty_set() {
    let root = tempdir().unwrap();
    assert!(claimed(root.path()).is_empty());
    deposit(root.path(), "g-1", &json!({"op": "balls"})).unwrap();
    let held = claim(root.path(), "g-1").unwrap();
    let ids: Vec<String> = claimed(root.path()).into_iter().map(|(id, _)| id).collect();
    assert_eq!(ids, ["g-1"]);
    drop(held);
}

#[test]
fn the_reply_file_is_the_done_marker() {
    let root = tempdir().unwrap();
    assert!(read_reply(root.path(), "g-1").is_none());
    write_reply(root.path(), "g-1", &json!({"ok": true})).unwrap();
    assert_eq!(read_reply(root.path(), "g-1"), Some(json!({"ok": true})));
    assert_eq!(
        reply_path(root.path(), "g-1"),
        gestures_dir(root.path()).join("replies").join("g-1.json")
    );
    // A torn/mangled reply file reads as not-yet, never a crash.
    std::fs::write(reply_path(root.path(), "g-2"), b"not json").unwrap();
    assert!(read_reply(root.path(), "g-2").is_none());
}
