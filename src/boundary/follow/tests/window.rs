//! **What the tool window promises the operator** (bl-5305): the third subject
//! of this lane's contract, beside what the tail promises ([`super`]) and how
//! the bytes are gathered ([`super::reading`]).
//!
//! A conversation administering a machine says almost nothing in prose while it
//! runs commands on it, so every beat here drives [`Follow::poll`] over a step
//! whose `response.json` never grows — the window is the only thing that can
//! have put an event on the frame.

use super::*;

/// One call's record landed under the followed step, exactly as litany lands
/// it: `input.json` immediately before the call is dispatched.
fn post(step: &Path, id: &str, name: &str, command: &str) -> std::path::PathBuf {
    let dir = step.join("tools").join(id);
    std::fs::create_dir_all(&dir).expect("call dir");
    std::fs::write(
        dir.join("input.json"),
        format!(r#"{{"id":"{id}","name":"{name}","input":{{"command":"{command}"}}}}"#),
    )
    .expect("input");
    dir
}

/// And the capture when it returns.
fn capture(dir: &Path, exit_code: i32) {
    std::fs::write(
        dir.join("output.json"),
        format!(
            r#"{{"stdout":"","stderr":"","exit_code":{exit_code},"started_at":"t0","ended_at":"t1"}}"#
        ),
    )
    .expect("output");
}

/// **The beat this ball exists for.** A command dispatched to an enrolled box
/// is on a frame as it is posted — naming the tool, the machine and the command
/// line — and its status is on the frame after the capture lands, while the
/// prose says nothing at all. Gated on the fold moving, neither frame exists.
#[test]
fn a_command_running_on_a_machine_is_on_a_frame_though_the_prose_says_nothing() {
    let (dir, _cell, mut follow) = flying();
    let file = response(dir.path(), 1);
    append(&file, "");
    let step = file.parent().expect("step dir").to_path_buf();
    let call = post(&step, "toolu_01", "box2_Bash", "apk add curl");

    let opened = ran(follow.poll()).expect("a frame, though no prose landed");
    assert_eq!(opened.len(), 1, "{opened:?}");
    assert_eq!(
        opened[0].name.as_deref(),
        Some("box2_Bash"),
        "the tool, and through its name the box it runs on"
    );
    assert!(
        opened[0]
            .input
            .as_deref()
            .is_some_and(|i| i.contains("apk add curl")),
        "and what it is about to run there: {:?}",
        opened[0].input
    );

    assert!(
        matches!(follow.poll(), Frame::Waiting),
        "a look with nothing new is nothing said — a frame is an append"
    );

    capture(&call, 1);
    let closed = ran(follow.poll()).expect("the capture is a frame too");
    assert_eq!(closed.len(), 1, "{closed:?}");
    assert_eq!(closed[0].tool_use, "toolu_01", "on the same identity");
    assert_eq!(closed[0].exit_code, Some(1), "with the status it earned");

    assert!(
        matches!(follow.poll(), Frame::Waiting),
        "and a closed call never speaks again"
    );
}

/// The window and the prose ride **one** frame when a look finds both: they are
/// two halves of one read, not two lanes that could interleave.
#[test]
fn prose_and_a_tool_event_from_one_look_are_one_frame() {
    let (dir, _cell, mut follow) = flying();
    let file = response(dir.path(), 1);
    append(&file, &text_delta("running it now"));
    let step = file.parent().expect("step dir").to_path_buf();
    post(&step, "toolu_01", "Bash", "uptime");

    let Frame::Ready(stream, tools) = follow.poll() else {
        panic!("one frame carries both halves");
    };
    assert_eq!(stream.text.as_deref(), Some("running it now"));
    assert_eq!(tools.len(), 1, "{tools:?}");
}

/// **The lane stays open while the tools run, and that is the half the ball
/// turns on.** A model call settles into `tool_use` blocks and the driver then
/// spends minutes running them: the conversation is `Live`, not `InFlight`.
/// Keyed on the model call, this read ended the moment the prose did — before
/// the first command was dispatched — so the operator lost the lane exactly
/// where the commands are. Keyed on the STEP, it holds, and the window is what
/// it carries.
#[test]
fn the_lane_holds_through_the_tool_phase_of_a_step() {
    let (dir, cell, mut follow) = flying();
    let file = response(dir.path(), 1);
    append(&file, &text_delta("On box2:"));
    assert!(
        matches!(follow.poll(), Frame::Ready(..)),
        "the call streams"
    );

    // The call settles into a tool block; the driver keeps the lease and runs
    // it. The derivation republishes the conversation as `Live`.
    crate::state::publish_snapshot(
        &cell,
        std::sync::Arc::new(crate::boundary::tests::snapshot(
            dir.path(),
            "alba",
            vec![crate::boundary::tests::agent(AGENT, AgentState::Live, 100)],
            vec![],
        )),
    );
    let step = file.parent().expect("step dir").to_path_buf();
    let call = post(&step, "toolu_01", "box2_Bash", "find / -name app.log");
    let opened = ran(follow.poll()).expect("the lane is still open");
    assert_eq!(opened.len(), 1, "and the command is on it: {opened:?}");

    capture(&call, 0);
    assert_eq!(
        ran(follow.poll()).map(|t| t.len()),
        Some(1),
        "and so is what it came back with"
    );

    // The driver lets go: nothing is being worked, and the stream is over.
    crate::state::publish_snapshot(
        &cell,
        std::sync::Arc::new(crate::boundary::tests::snapshot(
            dir.path(),
            "alba",
            vec![crate::boundary::tests::agent(
                AGENT,
                AgentState::Quiescent,
                100,
            )],
            vec![],
        )),
    );
    assert!(matches!(follow.poll(), Frame::Over));
}

/// A conversation with nothing in flight opens no window: the step's records
/// are the settled transcript's, which `Query::Transcript` already carries in
/// full, and this lane never opens a response file it is not following.
#[test]
fn a_settled_conversation_opens_no_window() {
    let dir = tempfile::tempdir().expect("tmp");
    let cell = seated(dir.path(), AgentState::Quiescent);
    let mut follow = Follow::new(cell, dir.path().to_path_buf(), AGENT.to_owned());
    let file = response(dir.path(), 1);
    append(&file, &text_delta("the settled answer"));
    post(
        file.parent().expect("step dir"),
        "toolu_01",
        "Bash",
        "uptime",
    );
    assert!(
        ran(follow.poll()).is_none(),
        "nothing is in flight, so there is nothing to follow"
    );
}
