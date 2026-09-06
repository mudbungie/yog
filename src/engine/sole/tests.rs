//! The world's one-engine exclusion.

use super::{LOCK, take};
use tempfile::tempdir;

/// The first engine takes the world; a second on the same world is refused,
/// naming the file and the invariant. This is the ordinary operator accident —
/// a supervisor restart, a second terminal, a stale `nohup` — and before
/// bl-1d9b it produced a second gesture consumer and a second `inv-N`
/// namespace rather than a refusal.
#[test]
fn a_second_engine_on_one_world_is_refused() {
    let dir = tempdir().expect("tmp");
    let state = dir.path().join("state").join("yog");
    let first = take(&state).expect("the first engine takes the world");
    let refusal = take(&state).err().expect("the second is refused");
    assert!(refusal.contains("one world has one engine"), "{refusal}");
    assert!(
        refusal.contains(LOCK),
        "the refusal names the file: {refusal}"
    );
    // The exclusion is the *held* descriptor, so releasing it frees the world
    // for the next engine — a restart is a stop and a start, with nothing to
    // reap in between.
    drop(first);
    take(&state).expect("the world is free once its engine has gone");
}

/// Two worlds are two exclusions: an engine on a scratch world beside the
/// operator's own is exactly the arrangement every drive run uses, and the
/// lock must not reach across data roots.
#[test]
fn two_worlds_are_two_exclusions() {
    let dir = tempdir().expect("tmp");
    let one = take(&dir.path().join("a")).expect("world a");
    let two = take(&dir.path().join("b")).expect("world b");
    drop((one, two));
}

/// A state root that cannot be made — a file where the directory goes — is a
/// refusal naming the path, never a boot that proceeds unexcluded.
#[test]
fn a_state_root_that_cannot_be_made_refuses() {
    let dir = tempdir().expect("tmp");
    let blocked = dir.path().join("blocked");
    std::fs::write(&blocked, b"a file where the state root goes").expect("block it");
    let refusal = take(&blocked).err().expect("refused");
    assert!(refusal.contains("engine lock"), "{refusal}");
}
