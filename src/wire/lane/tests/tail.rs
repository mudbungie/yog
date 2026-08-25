//! The frame's half of the lane, argued with no wire at all (REMOTE §3) — what
//! the two channels do about a subject that moved and a window that went away.
//!
//! Everything here drives [`pair`](super::super::pair) directly. That is the
//! point of the file: these are claims about the hand-off, and a listener
//! standing behind them would only make them slower to fail.

use super::*;

/// **The question is its own key.** A frame answering the conversation the seat
/// was watching a moment ago cannot land on the one it is watching now — the
/// subject change drops what landed, with nothing to say "stop".
#[test]
fn a_frame_for_the_conversation_just_left_lands_nowhere() {
    let (mut tail, end) = pair();
    let other = crate::boundary::codec::encode(&crate::boundary::Gesture::Ask(
        crate::boundary::Query::Follow {
            workspace: "alba".to_owned(),
            agent: "c-2".to_owned(),
        },
    ));
    // Standing on c-1, and a fold lands for it.
    watching(&mut tail);
    end.publish(
        &subject().to_string(),
        Some(crate::git_tree::Stream {
            text: Some("c-1 is talking".to_owned()),
            ..crate::git_tree::Stream::default()
        }),
    );
    assert!(
        declared(&mut tail).is_some(),
        "the fold is for this subject"
    );

    // The operator moves to c-2. What c-1 says next reaches nobody.
    tail.settle();
    tail.ask(&other);
    end.publish(
        &subject().to_string(),
        Some(crate::git_tree::Stream {
            text: Some("c-1 is still talking".to_owned()),
            ..crate::git_tree::Stream::default()
        }),
    );
    tail.settle();
    assert_eq!(tail.ask(&other), None, "and c-2 has said nothing yet");
}

/// **A lane whose window is gone stops asking.** The lane thread is not joined
/// on drop, so this is what ends it: the subject channel disconnecting is read
/// as no subject, which is the same resting state as a window that has nothing
/// open.
#[test]
fn a_lane_whose_frame_end_is_gone_follows_nothing() {
    let (mut tail, mut end) = pair();
    watching(&mut tail);
    assert!(end.standing().is_some(), "a live frame end has a subject");
    drop(tail);
    assert!(end.standing().is_none(), "and a dead one has none, forever");
}
