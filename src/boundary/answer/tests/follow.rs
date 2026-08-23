//! The follow-class read at the chokepoint every intake shares (bl-73e7).

use super::*;

/// **The follow-class read, answered where every intake shares one answer**
/// (REMOTE §3, bl-73e7). Most intakes cannot hold a connection open — a
/// deposit, a `yog gesture` at a terminal — so what they get is the tail as of
/// now, in one frame. It is a true answer of the same question and not a
/// degraded one: it is `live_tail`, the very fold `Query::Transcript` puts on
/// its tail and the very fold the held read's frames carry.
#[test]
fn a_follow_answered_once_is_the_tail_the_transcript_folds() {
    let mut row = agent("c-1", AgentState::InFlight, 100);
    row.stream = crate::git_tree::Stream {
        text: Some("half a thought".to_owned()),
        thinking: None,
        last_delta: Some(crate::git_tree::Delta::Text),
    };
    let d = deps(snapshot(&ws(), "alba", vec![row.clone()], vec![]));
    let asked = Query::Follow {
        workspace: "alba".to_owned(),
        agent: "c-1".to_owned(),
    };
    let Ok(Reply::Follow(stream)) = answer(&asked, &d, &ui(), 0) else {
        panic!("a follow answers a fold");
    };
    assert_eq!(stream, row.stream, "the snapshot's own tail, unmoved");

    // A settled conversation has no tail, and says so as an empty fold rather
    // than as a refusal: nothing is being written, which is a reading.
    let settled = deps(snapshot(
        &ws(),
        "alba",
        vec![agent("c-1", AgentState::Quiescent, 100)],
        vec![],
    ));
    let Ok(Reply::Follow(nothing)) = answer(&asked, &settled, &ui(), 0) else {
        panic!("a follow answers a fold");
    };
    assert_eq!(nothing, crate::git_tree::Stream::default());
}
