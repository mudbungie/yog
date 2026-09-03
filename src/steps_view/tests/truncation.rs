//! The §7.3 wound's **output-limit** class (bl-fb87): a step whose transport
//! framed cleanly around a turn the request's `max_tokens` cut off.
//!
//! Its sibling is [`super::wound`], the no-response class. The two are disjoint
//! on disk — one has no response bytes at all, the other a settled tail — and
//! what they share is the carrier and the seat: one badge word, one banner
//! sentence, said where the step is rendered.

use tempfile::tempdir;

use super::{AGENT, write_file};
use crate::git_tree::{AgentState, Framing};
use crate::steps_view::{OUTPUT_LIMIT, Wound, build_aged, latest_wound};

/// The bl-fb87 shape on disk: the model thought, said nothing, and the request
/// ran out of `max_tokens`. Every transport promise kept; the turn unfinished.
const THINKING_ONLY_LENGTH: &[u8] = br#"{"type":"message_start","v":1,"role":"assistant"}
{"type":"content_delta","index":0,"delta":{"thinking_delta":"weighing it up"}}
{"type":"usage","input_tokens":900,"output_tokens":4096}
{"type":"finish","reason":"length"}
{"type":"end"}
"#;

/// A step that framed clean is still a wound when the turn it framed was cut
/// off: the transport reading and the turn reading are two facts, and the row
/// says the one the operator needs.
#[test]
fn an_output_limited_step_is_a_wound_though_its_framing_is_complete() {
    let dir = tempdir().unwrap();
    let ws = dir.path();
    write_file(ws, "001", "response.json", THINKING_ONLY_LENGTH);
    write_file(ws, "001", "meta.json", br#"{"commit":"abc"}"#);

    let view = build_aged(ws, AGENT, AgentState::Stopped);
    let step = &view.steps[0];
    assert_eq!(step.wound, Wound::OutputLimit);
    assert_eq!(
        step.framing,
        Framing::Complete,
        "the transport reading is untouched — the sealed entry really is there"
    );
    assert_eq!(step.attempts, 1);
    assert_eq!(step.tokens.total_tokens(), 4_996);
    assert!(latest_wound(&view).wounded());
}

/// The partial answer survives: the wound marks the turn truncated, it does
/// not withhold what the model managed to say.
#[test]
fn a_partial_text_turn_keeps_its_text_and_is_still_marked_truncated() {
    let dir = tempdir().unwrap();
    let ws = dir.path();
    write_file(
        ws,
        "001",
        "response.json",
        br#"{"type":"content_delta","index":0,"delta":{"text_delta":"the answer is 4"}}
{"type":"finish","reason":"length"}
{"type":"end"}
"#,
    );
    assert_eq!(
        build_aged(ws, AGENT, AgentState::Stopped).steps[0].wound,
        Wound::OutputLimit
    );
    assert_eq!(
        crate::git_tree::stream_from_disk(ws, AGENT).text.as_deref(),
        Some("the answer is 4"),
        "the §5.1 #10 fold is untouched — the fragment still reaches the glass"
    );
}

/// Every other settled shape is left exactly where it was: a clean stop, a
/// turn that ended with a tool call to make, a refusal, a failed segment and a
/// tail with no clean end are none of them this wound.
#[test]
fn no_other_settled_shape_becomes_the_output_limit_wound() {
    let dir = tempdir().unwrap();
    let ws = dir.path();
    for (seq, reason) in [("001", "stop"), ("002", "tool_use"), ("003", "refusal")] {
        write_file(
            ws,
            seq,
            "response.json",
            format!("{{\"type\":\"finish\",\"reason\":\"{reason}\"}}\n{{\"type\":\"end\"}}\n")
                .as_bytes(),
        );
    }
    write_file(
        ws,
        "004",
        "response.json",
        b"{\"type\":\"error\",\"kind\":\"http\"}\n{\"type\":\"end\"}\n",
    );
    write_file(
        ws,
        "005",
        "response.json",
        b"{\"type\":\"content_delta\"}\n",
    );
    let view = build_aged(ws, AGENT, AgentState::Stopped);
    for step in &view.steps {
        assert_eq!(step.wound, Wound::None, "step {}", step.seq);
    }
    assert_eq!(view.steps[3].framing, Framing::Failed);
    assert_eq!(view.steps[4].framing, Framing::Killed);
}

/// The reason is rendered, in the badge's own seat and in the banner — the
/// whole point of the class: a mechanism that stays false while an explanation
/// is painted beside it is worse than no explanation (bl-fb87).
#[test]
fn the_output_limit_wound_says_what_ended_the_turn_and_how_to_carry_on() {
    let dir = tempdir().unwrap();
    let ws = dir.path();
    write_file(ws, "001", "response.json", THINKING_ONLY_LENGTH);

    let sentence = latest_wound(&build_aged(ws, AGENT, AgentState::Stopped)).banner();
    assert!(sentence.contains(OUTPUT_LIMIT), "the class: {sentence}");
    assert!(
        sentence.contains("output budget ran out"),
        "what happened, in words: {sentence}"
    );
    assert!(
        sentence.contains("send a message"),
        "the recovery is the gesture that exists: {sentence}"
    );
    assert!(
        sentence.contains("Nudge cannot"),
        "and the control that vanished says why: {sentence}"
    );
    assert!(
        !sentence.contains("  "),
        "one sentence, not a source continuation leaking its indent: {sentence}"
    );

    // The answer carries BOTH, and that is the class's whole shape: the
    // transport framed cleanly — so the framing is `complete` and honest — and
    // the wound is what says the turn was cut off inside that clean frame. A
    // seat says the wound where it would otherwise say the framing; the server
    // states the two facts and does not choose between them.
    let answered = crate::steps_view::wire::steps(&build_aged(ws, AGENT, AgentState::Stopped));
    let row = &answered["rows"][0];
    assert_eq!(row["wound"], "output_limit", "got:\n{answered:#}");
    assert_eq!(row["framing"], "complete", "got:\n{answered:#}");
}

/// The liveness gate is the class-blind one it always was: a driver at work on
/// the newest step excuses it, and only it.
#[test]
fn a_driver_at_work_excuses_the_newest_output_limited_step_only() {
    let dir = tempdir().unwrap();
    let ws = dir.path();
    write_file(ws, "001", "response.json", THINKING_ONLY_LENGTH);
    write_file(ws, "002", "response.json", THINKING_ONLY_LENGTH);
    let view = build_aged(ws, AGENT, AgentState::InFlight);
    assert_eq!(view.steps[0].wound, Wound::OutputLimit);
    assert_eq!(view.steps[1].wound, Wound::None);
}
