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
    let claimed = claim(root.path(), "g-1").unwrap();
    assert!(claimed.is_file());
    assert!(
        pending(root.path()).is_empty(),
        "a claimed deposit is off the list"
    );
    assert!(
        claim(root.path(), "g-1").is_err(),
        "the loser gets the error"
    );
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
