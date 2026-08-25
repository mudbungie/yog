//! **A lane with nobody at the other end** (REMOTE §3) — the far end absent,
//! the subject absent, and an answer that is not a follow frame at all.
//!
//! All three are one claim: the turn says nothing landed, the seat holds no
//! tail, and what paints is the committed transcript's own fold. The lane is an
//! improvement that can fail, never a mechanism that can break the chat.

use super::*;

/// **A lane that cannot dial is the pull path, not a broken chat.** The turn
/// answers that nothing landed — which is what paces the re-ask — and the seat
/// holds no tail, so the committed transcript's own fold is what paints.
#[test]
fn a_lane_with_no_engine_lands_nothing_and_says_so() {
    let tmp = TempDir::new().expect("tmp");
    mint(tmp.path());
    let seat = crate::wire::client::Seat::open(&material(tmp.path(), Role::Window, NO_LISTENER))
        .expect("seat");
    let (mut tail, end) = pair();
    let mut lane = Lane::new(crate::wire::dial::Dial::of(seat), end, Arc::new(NoRepaint));
    watching(&mut tail);
    assert!(!lane.turn(), "no engine, no frames");
    assert_eq!(declared(&mut tail), None);
}

/// A lane nobody has aimed asks nothing at all — the resting state of a window
/// with no conversation open, and the same `false` a failed dial answers, so
/// the caller paces both the same way.
#[test]
fn a_lane_with_no_subject_asks_nothing() {
    let tmp = TempDir::new().expect("tmp");
    let (_listener, mut lane, _tail) = wired(&tmp, vec![frame("unread")]);
    assert!(!lane.turn(), "nothing declared, nothing asked");
}

/// **An answer of another kind ends the read.** A refusal, or a reply the codec
/// carried faithfully but this lane did not ask for, is a defect rather than a
/// state — so the lane hangs up and the seat falls back rather than painting
/// something it cannot read.
#[test]
fn an_answer_of_another_kind_ends_the_read() {
    let tmp = TempDir::new().expect("tmp");
    let (_listener, mut lane, mut tail) = wired(
        &tmp,
        vec![json!({"ok": false, "error": "no such conversation"})],
    );
    watching(&mut tail);
    assert!(!lane.turn(), "nothing landed");
    assert_eq!(declared(&mut tail), None);
}
