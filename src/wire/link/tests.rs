//! The frame's half: declaring, landing, forgetting — and never waiting.

use super::*;
use crate::boundary::reply::Reply;
use serde_json::json;

fn clients(workspace: &str) -> Value {
    json!({"op": "clients", "workspace": workspace})
}

/// One frame, in the order [`AppModel::refresh`](crate::AppModel::refresh) runs
/// it: settle first (take what landed, declare what the last frame asked), then
/// render, which is where the question is asked. A surface that stops painting
/// therefore stops asking, with nothing to say so.
fn frame(link: &mut Link, question: &Value) -> Option<Landed> {
    link.settle();
    link.ask(question)
}

/// The first frame asks and gets nothing — which is the resting state of every
/// surface for one cadence period, and is not an error.
#[test]
fn a_question_nobody_has_answered_yet_lands_nothing() {
    let (mut link, mut end) = pair();
    assert!(frame(&mut link, &clients("home")).is_none());
    assert!(
        end.standing().is_empty(),
        "not declared until the next frame"
    );
    assert!(frame(&mut link, &clients("home")).is_none());
    assert_eq!(end.standing(), vec![clients("home")], "declared");
}

/// The whole round trip: the frame declares, the asker publishes, the next
/// frame paints it.
#[test]
fn an_answer_lands_and_the_next_frame_reads_it() {
    let (mut link, end) = pair();
    frame(&mut link, &clients("home"));
    frame(&mut link, &clients("home"));
    assert!(end.publish(&clients("home"), Ok(Reply::Attention(Vec::new()))));
    assert_eq!(
        frame(&mut link, &clients("home")),
        Some(Ok(Reply::Attention(Vec::new()))),
        "and it survives every frame that keeps asking"
    );
    assert!(frame(&mut link, &clients("home")).is_some());
}

/// A standing set is sent only when it **changes** — one compare per frame,
/// which is what a frame is allowed to cost.
#[test]
fn an_unchanged_declaration_is_not_re_sent() {
    let (mut link, mut end) = pair();
    for _ in 0..4 {
        frame(&mut link, &clients("home"));
    }
    assert_eq!(end.standing(), vec![clients("home")], "sent once");
    // Nothing new arrived, so the asker keeps asking what it was last told.
    assert_eq!(end.standing(), vec![clients("home")]);
}

/// Forgetting has no verb: a question the frame stops declaring stops being
/// asked, and its answer is dropped with it.
#[test]
fn a_question_nobody_declares_is_forgotten() {
    let (mut link, mut end) = pair();
    frame(&mut link, &clients("home"));
    frame(&mut link, &clients("home"));
    end.publish(&clients("home"), Ok(Reply::Attention(Vec::new())));
    assert!(frame(&mut link, &clients("home")).is_some());
    // The frame paints a different workspace now, so it declares that instead.
    assert!(frame(&mut link, &clients("other")).is_none());
    frame(&mut link, &clients("other"));
    assert_eq!(end.standing(), vec![clients("other")], "the new one only");
    assert!(
        frame(&mut link, &clients("home")).is_none(),
        "the old answer went with the old question"
    );
}

/// A refusal is an answer: the frame paints what it was told rather than
/// carrying a case for which layer said so.
#[test]
fn a_refusal_lands_like_any_other_answer() {
    let (mut link, end) = pair();
    frame(&mut link, &clients("nowhere"));
    frame(&mut link, &clients("nowhere"));
    end.publish(&clients("nowhere"), Err("unknown workspace".to_owned()));
    assert_eq!(
        frame(&mut link, &clients("nowhere")),
        Some(Err("unknown workspace".to_owned()))
    );
}

/// **A link nobody answers** is the posture of a box whose mint failed, and it
/// is the same code path: the frame asks, nothing lands, the surface says so.
#[test]
fn a_default_link_answers_nothing_and_does_not_fail() {
    let mut link = Link::default();
    for _ in 0..3 {
        assert!(frame(&mut link, &clients("home")).is_none());
    }
}

/// A window that has gone away is a publish that fails, which is how the asker
/// learns to stop.
#[test]
fn a_publish_to_a_dropped_window_fails() {
    let (link, end) = pair();
    drop(link);
    assert!(!end.publish(&clients("home"), Ok(Reply::Attention(Vec::new()))));
}
