//! Table tests for the §11 bottom in-flight strip ([`super::super::flight`],
//! bl-905f): exactly what each class states, the elapsed each derives from its
//! own structural start (bl-9dfb) or honestly omits, the two ways a conversation
//! reads as at rest, and the hover that explains the seat (bl-68ac).
//!
//! **The line is what the §11 header does not already say** (bl-3f70): the
//! class's own words moved to the header's chip, and the `<who>` segment is
//! painted only for a member that is not the conversation the strip sits under
//! — the pane's heading names that one. So every expectation here is the
//! characteristics alone.

use super::*;
use crate::git_tree::ToolCallState;
use crate::nav::convs::{Flight, STRIP_HOVER, age_label, conversation_flight, strip};

/// The frame's wall clock, as the shell mints it — every start below is stated
/// as an offset from this, so a test reads as "the call began N seconds ago".
const NOW: i64 = 1_800_000_000;

/// A root that already carries its §3.3 minted name. The strip does not paint
/// a root's name (bl-3f70 — the pane's heading does), so this is what a child
/// is built from and what proves the root's name stays *off* the line.
fn stamped(id: &str, name: &str, state: AgentState) -> Agent {
    let mut a = agent(id, state, 1);
    a.goal_name = Some(name.to_owned());
    a
}

/// A streaming root whose model call opened `ago` seconds back (`None` = a step
/// whose `request.json` yog could not stat).
fn streaming(text: &str, ago: Option<i64>) -> Agent {
    let mut a = stamped("r-0", "stench-pug", AgentState::InFlight);
    a.stream.text = Some(text.into());
    a.call_start_unix = ago.map(|s| NOW - s);
    a
}

/// The strip's line for one tool-holding root, `name`/`start` being the two
/// segments its `input.json` record feeds.
fn tool_line(name: Option<&str>, start: Option<i64>) -> Option<String> {
    let mut a = named_tool(
        "r-0",
        AgentState::Live,
        ToolCallState::InFlight,
        name,
        start,
    );
    a.goal_name = Some("stench-pug".into());
    strip(&[a], "r-0", NOW).map(|s| s.facts)
}

#[test]
fn inference_states_who_is_streaming_how_much_landed_and_how_long_it_has_run() {
    let s = strip(&[streaming("hello", Some(42))], "r-0", NOW).unwrap();
    assert_eq!(s.class, Flight::Inference);
    assert_eq!(s.facts, "5 chars streamed · 42s");
    // The root's own name is the pane heading's, two lines above (bl-3f70).
    assert!(!s.facts.contains("stench-pug"), "{}", s.facts);
}

#[test]
fn a_call_whose_first_token_has_not_arrived_says_so_rather_than_nothing() {
    // Zero is the general path with an empty tail, not a special case — the
    // honest thing to say about an open call that has answered nothing yet.
    let mut root = stamped("r-0", "stench-pug", AgentState::InFlight);
    root.call_start_unix = Some(NOW - 3);
    assert_eq!(
        strip(&[root], "r-0", NOW).unwrap().facts,
        "0 chars streamed · 3s"
    );
    // …and one character is one char, not "1 chars".
    assert_eq!(
        strip(&[streaming("h", Some(3))], "r-0", NOW).unwrap().facts,
        "1 char streamed · 3s"
    );
}

#[test]
fn a_step_with_no_readable_request_stamp_drops_the_elapsed_not_the_line() {
    // §5.1 #28: a missing structural start omits the segment. Everything the
    // snapshot does carry is still stated — partial is lawful, invented is not.
    assert_eq!(
        strip(&[streaming("hello", None)], "r-0", NOW)
            .unwrap()
            .facts,
        "5 chars streamed"
    );
}

#[test]
fn the_strip_names_the_member_working_never_the_conversation_it_sits_under() {
    // A quiescent root with a streaming child: the class is the
    // conversation's, but the characteristics are the child's — which is the
    // whole reason the seat names an agent at all. The elapsed is the child's
    // too: the start rides with whoever is doing the work.
    let root = stamped("r-0", "stench-pug", AgentState::Quiescent);
    let mut child = stamped("r-0-c-1", "thistle-vane", AgentState::InFlight);
    child.stream.text = Some("abc".into());
    child.call_start_unix = Some(NOW - 7200);
    let s = strip(&[root, child], "r-0", NOW).unwrap();
    assert_eq!(s.class, Flight::Inference);
    assert_eq!(s.facts, "thistle-vane · 3 chars streamed · 2h");
}

#[test]
fn tools_states_the_running_tool_by_name_and_how_long_it_has_been_running() {
    let mut a = named_tool(
        "r-0",
        AgentState::Live,
        ToolCallState::InFlight,
        Some("Bash"),
        Some(NOW - 420),
    );
    a.goal_name = Some("stench-pug".into());
    let s = strip(&[a], "r-0", NOW).unwrap();
    assert_eq!(s.class, Flight::Tools);
    assert_eq!(s.facts, "Bash · 7m");
}

#[test]
fn the_tool_name_and_the_tool_elapsed_are_independently_omitted() {
    // Two segments off one record: an unparsable name loses the name only, an
    // unstattable record loses the elapsed only. `toolu_01abc…` names nothing
    // to an operator, so the class's own words carry it.
    assert_eq!(tool_line(None, Some(NOW - 9)), Some("9s".to_owned()));
    assert_eq!(tool_line(Some("Bash"), None), Some("Bash".to_owned()));
    // Neither name nor stamp, on the conversation's own root: nothing this seat
    // can add that the header's chip does not already say, so there is no strip
    // rather than a bare glyph (bl-3f70).
    assert_eq!(tool_line(None, None), None);
}

#[test]
fn subagents_counts_the_children_and_counts_them_in_english() {
    let root = stamped("r-0", "stench-pug", AgentState::Quiescent);
    let one = [root.clone(), agent("r-0-c-1", AgentState::Live, 2)];
    let s = strip(&one, "r-0", NOW).unwrap();
    assert_eq!(s.class, Flight::Subagents);
    assert_eq!(s.facts, "1 child running");
    let two = [
        root,
        agent("r-0-c-1", AgentState::Live, 2),
        agent("r-0-c-2", AgentState::Live, 3),
        // A settled sibling is not running and is not counted.
        agent("r-0-c-3", AgentState::Stopped, 4),
    ];
    assert_eq!(strip(&two, "r-0", NOW).unwrap().facts, "2 children running");
}

#[test]
fn subagents_shows_no_elapsed_even_when_the_children_carry_starts() {
    // The class is by construction the between-steps window: the child holds a
    // driver while running neither a model call nor a tool, so its latest
    // step's start times something that already ended. The refusal belongs to
    // the class, not to a missing fact — hence a child that *does* carry one
    // and still shows none.
    let root = stamped("r-0", "stench-pug", AgentState::Quiescent);
    let mut child = agent("r-0-c-1", AgentState::Live, 2);
    child.call_start_unix = Some(NOW - 60);
    assert_eq!(
        strip(&[root, child], "r-0", NOW).unwrap().facts,
        "1 child running"
    );
}

#[test]
fn the_elapsed_label_is_the_row_age_label_not_a_second_spelling() {
    // One home for "how long" (§11): the strip's segment must be exactly what
    // the list row would print for the same span, across every bucket.
    for ago in [0_i64, 42, 420, 7200, 200_000] {
        let s = strip(&[streaming("x", Some(ago))], "r-0", NOW).unwrap();
        assert_eq!(s.facts, format!("1 char streamed · {}", age_label(ago)));
    }
    // Clock skew (a start stamped ahead of the frame) clamps to `0s` there
    // too — the strip inherits the clamp rather than restating it.
    let mut ahead = streaming("x", Some(0));
    ahead.call_start_unix = Some(NOW + 90);
    assert_eq!(
        strip(&[ahead], "r-0", NOW).unwrap().facts,
        "1 char streamed · 0s"
    );
}

#[test]
fn nothing_in_flight_is_no_strip_at_all() {
    // §7.2: `None` is both "paint nothing" and "schedule nothing" — the seat
    // has one decision, so it cannot drift into an idle repaint loop.
    let settled = [
        agent("r-0", AgentState::Quiescent, 1),
        agent("r-0-c-1", AgentState::Stopped, 2),
    ];
    assert_eq!(strip(&settled, "r-0", NOW), None);
    // An id that roots no conversation here, and an empty world.
    assert_eq!(strip(&settled, "ghost", NOW), None);
    assert_eq!(strip(&[], "r-0", NOW), None);
}

#[test]
fn the_strip_is_the_third_seat_of_one_derivation_not_a_second_one() {
    // Priority is resolved once, in `flight`: a streaming root that also holds
    // a running tool under a live child reads inference at every seat — and
    // times the model call, not the tool.
    let root = streaming("hi", Some(5));
    let child = named_tool(
        "r-0-c-1",
        AgentState::Live,
        ToolCallState::InFlight,
        Some("Read"),
        Some(NOW - 3600),
    );
    let agents = [root, child];
    let s = strip(&agents, "r-0", NOW).unwrap();
    assert_eq!(s.class, conversation_flight(&agents, "r-0").unwrap());
    assert_eq!(s.class, Flight::Inference);
    assert_eq!(s.facts, "2 chars streamed · 5s");
}

#[test]
fn the_seat_explains_itself_on_hover() {
    // bl-68ac: a surface the operator did not ask for says what it is.
    assert!(!STRIP_HOVER.is_empty());
    assert!(STRIP_HOVER.contains("in flight"));
}
