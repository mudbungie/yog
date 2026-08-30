//! The §4.4 **output-limit reading** the classifier answers beside the state
//! (bl-fb87): transport completion is not task completion, so a turn the
//! request's `max_tokens` cut off is `Stopped` **and** truncated, and §8.2's
//! Nudge gate is the one consumer.
//!
//! Split from [`super::state_unit`] — which owns the (lock × writer × framing)
//! mapping table — on the seam DESIGN §12 draws through the classifier itself:
//! *"one read, two facts"*. The probe stubs and the response-writing helpers
//! are that module's, shared rather than restated, so the two suites can never
//! disagree about what a settled step looks like.

use super::state_unit::{FINISH_END, lock, reading, resp, write, writer};
use crate::git_tree::{AgentState, Probe};
use tempfile::tempdir;

/// The bl-fb87 shape: thinking, no text, no `tool_use`, and a `finish` whose
/// canonical reason is `length` — every transport promise kept around a turn
/// the request's `max_tokens` cut off.
const LENGTH_END: &[u8] = br#"{"type":"message_start","v":1,"role":"assistant"}
{"type":"content_delta","index":0,"delta":{"thinking_delta":"hmm"}}
{"type":"finish","reason":"length"}
{"type":"end"}
"#;

#[test]
fn no_lock_and_output_limited_response_is_stopped_and_truncated() {
    // Transport completion is not task completion (bl-fb87): the tail frames
    // clean, so §4.4 reads it `Complete` and `rail::place` still pairs it with
    // the entry litany sealed — but the turn ran out of room, which is a
    // conversation stopped mid-utterance, not one at rest.
    let dir = tempdir().unwrap();
    let agent = "20260427T140000Z-llll";
    write(&resp(dir.path(), agent, "001"), LENGTH_END);
    assert_eq!(
        reading(dir.path(), agent, &lock(Probe::Free), &writer(Probe::Free)),
        (AgentState::Stopped, false, true)
    );
}

#[test]
fn a_driver_at_work_over_an_output_limited_step_reads_neither() {
    // The truncation reading is asked only at rest: a driver holding the lease
    // is itself the answer to "what now", and Nudge is already off for it.
    let dir = tempdir().unwrap();
    let agent = "20260427T140000Z-mmmm";
    write(&resp(dir.path(), agent, "001"), LENGTH_END);
    assert_eq!(
        reading(dir.path(), agent, &lock(Probe::Held), &writer(Probe::Free)),
        (AgentState::Live, false, false)
    );
}

#[test]
fn a_tool_use_finish_is_untouched_by_the_output_limit_reading() {
    // A turn that really did end with a call to make finishes `tool_use`, not
    // `length` — the canonical reason is one value and the provider names it,
    // so the continuation case needs no content sniffing to stay Quiescent.
    let dir = tempdir().unwrap();
    let agent = "20260427T140000Z-tttt";
    write(
        &resp(dir.path(), agent, "001"),
        br#"{"type":"finish","reason":"tool_use"}
{"type":"end"}
"#,
    );
    assert_eq!(
        reading(dir.path(), agent, &lock(Probe::Free), &writer(Probe::Free)),
        (AgentState::Quiescent, false, false)
    );
}

#[test]
fn an_output_limited_step_behind_a_newer_one_is_not_the_reading() {
    // Only the latest step settles the agent (§3.5): a truncated turn the
    // operator already carried on from is history, not the state.
    let dir = tempdir().unwrap();
    let agent = "20260427T140000Z-nnnn";
    write(&resp(dir.path(), agent, "001"), LENGTH_END);
    write(&resp(dir.path(), agent, "002"), FINISH_END);
    assert_eq!(
        reading(dir.path(), agent, &lock(Probe::Free), &writer(Probe::Free)),
        (AgentState::Quiescent, false, false)
    );
}
