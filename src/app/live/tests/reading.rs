//! How the follower gathers bytes: partial-write tolerance, the one reset rule,
//! idleness, and the real thread.
//!
//! Split from [`super`] at §12's per-file budget; the fixtures are the parent's.

use super::super::{LiveTail, overlay};
use super::{
    AGENT, CountingRepaint, Harness, append, painted, response, rigged, text_delta, thinking_delta,
};
use crate::git_tree::{Delta, Stream};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

#[test]
fn a_half_written_line_waits_for_its_newline() {
    let h = Harness::new();
    let (mut rig, mut follower) = rigged(&h);
    let file = response(&h, AGENT, 1);
    // The harness is mid-write: the line has no terminator yet.
    let whole = text_delta("atomic");
    let (head, tail) = whole.split_at(whole.len() / 2);
    append(&file, head);
    assert!(!follower.pass(), "nothing whole arrived");
    rig.refresh();
    assert_eq!(painted(&rig, &h.ws, AGENT).text, None);
    // The rest lands and the line folds — never half a JSON object.
    append(&file, tail);
    assert!(follower.pass());
    rig.refresh();
    assert_eq!(painted(&rig, &h.ws, AGENT).text.as_deref(), Some("atomic"));
}

#[test]
fn an_idle_stream_publishes_nothing_so_a_quiet_call_costs_no_repaints() {
    let h = Harness::new();
    let (_rig, mut follower) = rigged(&h);
    let file = response(&h, AGENT, 1);
    append(&file, &text_delta("said"));
    assert!(follower.pass(), "the first bytes are news");
    assert!(!follower.pass(), "an unchanged file is not");
    assert!(!follower.pass());
    // Bytes that say nothing the operator can see are not news either: the
    // offset advances, the tail does not, and no frame is asked for.
    append(&file, "{\"type\":\"message_start\"}\n");
    assert!(
        !follower.pass(),
        "an event outside the delta seam wakes nobody"
    );
}

#[test]
fn focus_moving_drops_the_tail_and_opens_the_new_conversation() {
    let h = Harness::new();
    h.build_more("c-2", "second");
    let (mut rig, mut follower) = rigged(&h);
    append(&response(&h, AGENT, 1), &text_delta("first conversation"));
    follower.pass();
    rig.refresh();
    assert_eq!(
        painted(&rig, &h.ws, AGENT).text.as_deref(),
        Some("first conversation")
    );

    // The operator navigates. The accumulator is dropped whole — the tail is a
    // preview of the *focused* conversation and of nothing else — and the one
    // left behind reverts to the derivation's own fold, on the sweep's cadence.
    append(&response(&h, "c-2", 1), &text_delta("second conversation"));
    rig.focus_agent(&h.ws, "c-2");
    rig.refresh();
    assert!(follower.pass());
    rig.refresh();
    assert_eq!(
        painted(&rig, &h.ws, "c-2").text.as_deref(),
        Some("second conversation")
    );
    assert_eq!(
        painted(&rig, &h.ws, AGENT).text,
        None,
        "the conversation just left is back to what the derivation says"
    );
}

#[test]
fn a_new_step_starts_the_accumulator_over() {
    let h = Harness::new();
    let (mut rig, mut follower) = rigged(&h);
    append(&response(&h, AGENT, 1), &text_delta("step one's answer"));
    follower.pass();
    rig.refresh();
    assert_eq!(
        painted(&rig, &h.ws, AGENT).text.as_deref(),
        Some("step one's answer")
    );
    // The step commits and the next opens: a different stream, so the previous
    // text is dropped rather than prefixed onto this one.
    append(&response(&h, AGENT, 2), &text_delta("step two's"));
    assert!(follower.pass());
    rig.refresh();
    assert_eq!(
        painted(&rig, &h.ws, AGENT).text.as_deref(),
        Some("step two's")
    );
}

#[test]
fn a_truncated_file_is_re_read_from_the_start() {
    let h = Harness::new();
    let (mut rig, mut follower) = rigged(&h);
    let file = response(&h, AGENT, 1);
    append(&file, &text_delta("aaaaaaaaaaaaaaaaaaaa"));
    follower.pass();
    // The file shrinks under the follower (a replaced step record): its offset
    // is past the end, so it starts over rather than reading from nowhere.
    std::fs::write(&file, text_delta("b")).expect("truncate");
    assert!(follower.pass());
    rig.refresh();
    assert!(
        painted(&rig, &h.ws, AGENT)
            .text
            .as_deref()
            .is_some_and(|t| t.ends_with('b')),
        "the bytes that are there now are what folded"
    );
}

#[test]
fn nothing_focused_and_nothing_opened_are_the_same_empty_answer() {
    let h = Harness::new();
    let (_clock, mut rig) = h.model();
    let mut follower = rig.follower();
    // No conversation focused: nothing to follow.
    rig.refresh();
    assert!(!follower.pass());
    // A conversation focused whose step file does not exist yet — the general
    // path with empty input, not a wait state.
    rig.focus_agent(&h.ws, AGENT);
    rig.refresh();
    assert!(!follower.pass());
    // One lands and is published; then the focus moves to a conversation with
    // no step record at all — the same empty answer as no focus, reached by a
    // different road, and the tail retiring is a change the face has to see.
    append(&response(&h, AGENT, 1), &text_delta("hi"));
    assert!(follower.pass());
    rig.focus_agent(&h.ws, "never-ran");
    rig.refresh();
    assert!(follower.pass(), "the tail retiring is news too");
    assert!(!follower.pass(), "and then there is nothing to say");
    // And leaving the conversation entirely is that same resting state.
    rig.focus_workspace(&crate::naming::leaf(&h.ws));
    rig.refresh();
    assert!(!follower.pass());
}

#[test]
fn a_tail_for_an_agent_the_snapshot_does_not_carry_writes_nothing() {
    let h = Harness::new();
    let (_clock, rig) = h.model();
    let mut snap = (**rig.model.derivation()).clone();
    // Neither the workspace nor the agent is on the roster: the conversation
    // was deleted, or has not been enumerated. Minting a row for it is the
    // §3.4 pending echo's job, never the tail's.
    overlay(
        &mut snap,
        &LiveTail {
            ws: "/nowhere".into(),
            agent: AGENT.to_owned(),
            stream: Stream::default(),
        },
    );
    overlay(
        &mut snap,
        &LiveTail {
            ws: h.ws.clone(),
            agent: "no-such-agent".to_owned(),
            stream: Stream::default(),
        },
    );
    assert_eq!(
        snap.trees.get(&h.ws).map(|t| t.agents.len()),
        Some(1),
        "no row minted, none removed"
    );
}

#[test]
fn the_spawned_follower_runs_the_real_thread_and_wakes_the_face() {
    let h = Harness::new();
    let (mut rig, follower) = rigged(&h);
    let file = response(&h, AGENT, 1);
    let count = Arc::new(AtomicUsize::new(0));
    let thread = follower.spawn(CountingRepaint(Arc::clone(&count)));
    append(&file, &text_delta("landed"));
    // The one thing a hand-driven pass cannot prove: the loop picks the bytes
    // up on its own and asks the face to paint them.
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while count.load(Ordering::Relaxed) == 0 && std::time::Instant::now() < deadline {
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert!(
        count.load(Ordering::Relaxed) >= 1,
        "the follower requested a repaint when characters landed"
    );
    drop(thread); // clean stop + join
    rig.refresh();
    assert_eq!(painted(&rig, &h.ws, AGENT).text.as_deref(), Some("landed"));
}

#[test]
fn absorbing_a_suffix_is_the_same_fold_as_reading_the_whole_file() {
    // The follower's whole licence to read incrementally: `fold(a).absorb(
    // fold(b)) == fold(a ++ b)` on any line boundary. Without this equality the
    // incremental read would be a second parser wearing the first one's name.
    let head = thinking_delta("mm");
    let whole = format!("{head}{}{}", text_delta("ab"), text_delta("cd"));
    let mut resumed = crate::git_tree::fold_stream(head.as_bytes());
    resumed.absorb(crate::git_tree::fold_stream(
        whole.get(head.len()..).expect("suffix").as_bytes(),
    ));
    assert_eq!(resumed, crate::git_tree::fold_stream(whole.as_bytes()));
    assert_eq!(resumed.text.as_deref(), Some("abcd"));
    assert_eq!(resumed.thinking.as_deref(), Some("mm"));
    assert_eq!(resumed.last_delta, Some(Delta::Text));
    // A suffix that said nothing leaves the accumulator exactly as it was.
    let mut held = resumed.clone();
    held.absorb(Stream::default());
    assert_eq!(held, resumed);
}
