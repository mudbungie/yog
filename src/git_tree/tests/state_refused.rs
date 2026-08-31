//! The **provider-refusal reading** the classifier answers beside the state
//! (bl-b43b): a conversation whose latest model call was refused at the
//! provider rung comes to rest `Stopped` exactly as an operator's own `/stop`
//! does — the badge set is frozen at four (§5.1 #9) — so what tells the two
//! apart is this fact, read off the same bytes in the same pass.
//!
//! Split from [`super::state_unit`] on the seam that module's siblings already
//! use (DESIGN §12, *"one read, two facts"*), and sharing its probe stubs and
//! response writers so no two suites can disagree about what a settled step
//! looks like.

use super::state_unit::{FINISH_END, lock, refusal, resp, write, writer};
use crate::git_tree::Probe;
use tempfile::tempdir;

/// A model call refused at the provider rung: the adapter speaks the failure
/// in band on stdout (§4.4), so the settled tail's last event before `end` is
/// an `error` whose text is auth-shaped.
const REFUSED_END: &[u8] = br#"{"type":"message_start","v":1,"role":"assistant"}
{"type":"error","kind":"http","status":401,"message":"invalid api key"}
{"type":"end"}
"#;

/// A refusal that is not about credentials — the same framing, a different
/// cause, and the heuristic must not claim it.
const RESET_END: &[u8] = br#"{"type":"message_start","v":1,"role":"assistant"}
{"type":"error","kind":"transport","message":"connection reset"}
{"type":"end"}
"#;

#[test]
fn a_rest_whose_latest_call_was_refused_says_so_beside_the_state() {
    let dir = tempdir().unwrap();
    let agent = "20260427T140000Z-rrrr";
    write(&resp(dir.path(), agent, "001"), REFUSED_END);
    assert!(refusal(
        dir.path(),
        agent,
        &lock(Probe::Free),
        &writer(Probe::Free)
    ));
}

#[test]
fn a_rest_that_failed_on_anything_else_is_not_a_refusal() {
    // The state is `Stopped` either way; only the *why* differs, and a wound
    // that names the wrong remedy is worse than one that names none.
    let dir = tempdir().unwrap();
    for (agent, tail) in [
        ("20260427T140000Z-tttt", RESET_END),
        ("20260427T140000Z-qqqq", FINISH_END),
    ] {
        write(&resp(dir.path(), agent, "001"), tail);
        assert!(
            !refusal(dir.path(), agent, &lock(Probe::Free), &writer(Probe::Free)),
            "{agent}"
        );
    }
}

#[test]
fn a_driver_at_work_over_a_refused_step_reads_nothing() {
    // Asked only at rest, for the truncation reading's reason exactly: a
    // driver holding the lease is itself the answer to "what now".
    let dir = tempdir().unwrap();
    let agent = "20260427T140000Z-dddd";
    write(&resp(dir.path(), agent, "001"), REFUSED_END);
    assert!(!refusal(
        dir.path(),
        agent,
        &lock(Probe::Held),
        &writer(Probe::Free)
    ));
}

#[test]
fn a_conversation_with_no_step_at_all_is_not_a_refusal() {
    // Nothing on disk says a provider said no — the general path with no
    // input, which is also the killed-tail reading beside it.
    let dir = tempdir().unwrap();
    assert!(!refusal(
        dir.path(),
        "20260427T140000Z-nnnn",
        &lock(Probe::Free),
        &writer(Probe::Free)
    ));
}
