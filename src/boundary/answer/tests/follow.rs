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
    let Ok(Reply::Follow(frame)) = answer(&asked, &d, &ui(), 0) else {
        panic!("a follow answers a fold");
    };
    let stream = frame.stream;
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
    assert_eq!(
        nothing,
        crate::boundary::reply::FollowFrame::default(),
        "no tail, and no window open on it either"
    );
}

/// **The one-shot answer carries the window through the tool phase too**
/// (bl-5305). A conversation running a command is `Live`, not `InFlight`: its
/// prose is settled and the committed transcript carries it, and what is
/// happening *now* is the call. An answer gated on the narrower liveness would
/// hand a seat an empty frame at exactly the moment it asked.
#[test]
fn a_follow_answered_once_carries_the_window_while_the_tools_run() {
    // A real directory, because this is the one follow reading whose subject is
    // the workspace's own bytes rather than the snapshot's derivations.
    let dir = tempfile::tempdir().expect("tmp");
    let ws = dir.path().join("alba");
    let step = ws.join("steps").join("c-1").join("001");
    let call = step.join("tools").join("toolu_01");
    std::fs::create_dir_all(&call).expect("call dir");
    std::fs::write(
        call.join("input.json"),
        br#"{"id":"toolu_01","name":"box2_Bash","input":{"command":"service nginx restart"}}"#,
    )
    .expect("input");
    let d = deps(snapshot(
        &ws,
        "alba",
        vec![agent("c-1", AgentState::Live, 100)],
        vec![],
    ));
    let asked = Query::Follow {
        workspace: "alba".to_owned(),
        agent: "c-1".to_owned(),
    };
    let answered = answer(&asked, &d, &ui(), 0);
    let Ok(Reply::Follow(frame)) = answered else {
        panic!("a follow answers a frame: {answered:?}");
    };
    assert_eq!(frame.stream, crate::git_tree::Stream::default(), "settled");
    assert_eq!(frame.tools.len(), 1, "{:?}", frame.tools);
    assert_eq!(frame.tools[0].name.as_deref(), Some("box2_Bash"));
}
