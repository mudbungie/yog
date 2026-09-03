//! The §7.3 step wound: the derivation, its liveness gate, the Altitude-1
//! predicate, the **reason** read off the step's own `stderr.log` (bl-55d8),
//! and the sentence that actually reaches the paint output — the bl-8e07
//! real-substrate finding, driven from its on-disk shape. The wound's **other**
//! class, the §4.4 output limit, is [`super::truncation`] — a separate file
//! because it is a separate on-disk shape with its own fixtures, not because
//! either outgrew a cap.

use tempfile::tempdir;

use super::{AGENT, write_file};
use crate::git_tree::{AgentState, Framing};
use crate::steps_view::{NO_RESPONSE, Wound, build_aged, latest_wound};

/// The observed repro: litany wrote `request.json`, opened `response.json`,
/// and died on a substrate version skew — zero bytes, no `meta.json`.
fn write_repro(ws: &std::path::Path, seq: &str) {
    write_file(ws, seq, "request.json", br#"{"model":"opus"}"#);
    write_file(ws, seq, "response.json", b"");
}

/// The bl-55d8 falsifying run's step 002, byte-for-byte: the adapter refused
/// before it could say anything in band, and said why on stderr instead.
const BZ_REFUSAL: &str = "bz: no workspace in this environment — providers, sign-ins and the model \
cache belong to a workspace, and there is nothing shared to fall back to. Run this inside a yog \
workspace, or focus one in yog.";

#[test]
fn a_driver_that_wrote_nothing_is_a_wound_not_a_quiet_step() {
    let dir = tempdir().unwrap();
    let ws = dir.path();
    write_repro(ws, "001");

    let view = build_aged(ws, AGENT, AgentState::Stopped);
    let step = &view.steps[0];
    assert!(
        step.wound.wounded(),
        "zero-byte response + no meta is the wound"
    );
    // The figures that made the row read quiet are untouched — the wound is a
    // rendered fact beside them, not a different count.
    assert_eq!(step.attempts, 0);
    assert_eq!(step.tokens.total_tokens(), 0);
    assert_eq!(step.framing, Framing::Killed);
    // The §11 Altitude-1 banner keys on the same derivation.
    assert!(latest_wound(&view).wounded());
}

#[test]
fn an_absent_response_with_no_meta_is_the_same_wound() {
    let dir = tempdir().unwrap();
    let ws = dir.path();
    // Died before the model call opened the file at all — same fact.
    write_file(ws, "001", "request.json", b"{}");
    assert!(
        build_aged(ws, AGENT, AgentState::Stopped).steps[0]
            .wound
            .wounded()
    );
}

#[test]
fn a_step_with_bytes_or_a_settled_meta_is_not_a_wound() {
    let dir = tempdir().unwrap();
    let ws = dir.path();
    // 001 answered; 002 emitted nothing but settled (meta written, however
    // malformed — its bytes say the call returned).
    write_file(
        ws,
        "001",
        "response.json",
        b"{\"type\":\"finish\",\"reason\":\"stop\"}\n{\"type\":\"end\"}\n",
    );
    write_file(ws, "002", "response.json", b"");
    write_file(ws, "002", "meta.json", b"not json at all");
    // …and a settled step's stderr is nobody's business: a retried attempt may
    // have written to it before the next one succeeded, which is not a wound.
    write_file(ws, "002", "stderr.log", BZ_REFUSAL.as_bytes());
    let view = build_aged(ws, AGENT, AgentState::Stopped);
    assert_eq!(view.steps[0].wound, Wound::None);
    assert_eq!(view.steps[1].wound, Wound::None);
    assert!(!latest_wound(&view).wounded());
    // No steps at all: nothing to banner.
    assert_eq!(
        latest_wound(&build_aged(ws, "ghost", AgentState::Stopped)),
        Wound::None
    );
}

#[test]
fn a_live_drivers_newest_step_is_a_call_in_flight_not_a_wound() {
    let dir = tempdir().unwrap();
    let ws = dir.path();
    write_repro(ws, "001");
    // A driver holds the lock: the newest step is legitimately empty for the
    // moments before the first streamed event (§10 — never a false definite).
    for state in [AgentState::Live, AgentState::InFlight] {
        let view = build_aged(ws, AGENT, state);
        assert!(!view.steps[0].wound.wounded());
        assert!(!latest_wound(&view).wounded());
    }
    // Nobody driving: the same bytes are the wound.
    for state in [AgentState::Quiescent, AgentState::Stopped] {
        assert!(build_aged(ws, AGENT, state).steps[0].wound.wounded());
    }
}

#[test]
fn a_live_driver_excuses_only_its_newest_step() {
    let dir = tempdir().unwrap();
    let ws = dir.path();
    write_repro(ws, "001");
    write_repro(ws, "002");
    let view = build_aged(ws, AGENT, AgentState::InFlight);
    assert!(
        view.steps[0].wound.wounded(),
        "where the conversation died stays rendered after a resume"
    );
    assert!(!view.steps[1].wound.wounded());
}

/// bl-55d8, the whole of it: the operator's only signal was the absence of a
/// reply, and the reason was bytes in a file yog already reads past.
#[test]
fn the_wound_carries_the_adapters_own_words_off_the_steps_stderr_log() {
    let dir = tempdir().unwrap();
    let ws = dir.path();
    write_repro(ws, "001");
    write_file(ws, "001", "stderr.log", BZ_REFUSAL.as_bytes());

    let wound = latest_wound(&build_aged(ws, AGENT, AgentState::Stopped));
    assert_eq!(wound, Wound::Spoke(BZ_REFUSAL.to_owned()));
    let sentence = wound.banner();
    assert!(sentence.contains(NO_RESPONSE), "the class: {sentence}");
    assert!(
        sentence.contains("no workspace in this environment"),
        "the reason, in words, not a pointer at somewhere to look: {sentence}"
    );
    assert!(
        sentence.contains("stderr.log"),
        "where the rest is: {sentence}"
    );
}

/// A `stderr.log` that exists and is empty (the ordinary shape — litany opens
/// it every attempt) is not words, and neither is one of pure whitespace.
#[test]
fn a_wound_with_an_empty_stderr_log_says_so_rather_than_inventing_a_cause() {
    let dir = tempdir().unwrap();
    let ws = dir.path();
    write_repro(ws, "001");
    write_file(ws, "001", "stderr.log", b"");
    assert_eq!(
        build_aged(ws, AGENT, AgentState::Stopped).steps[0].wound,
        Wound::Mute
    );
    write_file(ws, "001", "stderr.log", b"  \n\n ");
    let wound = latest_wound(&build_aged(ws, AGENT, AgentState::Stopped));
    assert_eq!(wound, Wound::Mute);
    let sentence = wound.banner();
    assert!(
        sentence.contains(NO_RESPONSE),
        "still the class: {sentence}"
    );
    assert!(
        sentence.contains("nothing on disk says why"),
        "the honest end of the trail: {sentence}"
    );
    // A step with no `stderr.log` file at all reads the same way.
    let bare = tempdir().unwrap();
    write_repro(bare.path(), "001");
    assert_eq!(
        build_aged(bare.path(), AGENT, AgentState::Stopped).steps[0].wound,
        Wound::Mute
    );
    // Not a wound and no sentence — the caller gates on `wounded` and says the
    // framing instead.
    assert_eq!(Wound::None.banner(), "");
    assert_eq!(Wound::default(), Wound::None);
}

/// The banner quotes a *tail*, on the two bounds the crate already had: at most
/// `opslog::detached`'s 4 KiB of file, then `opslog::rows`' last three lines.
/// A chatty adapter cannot push a banner off the screen.
#[test]
fn a_chatty_adapter_is_quoted_by_its_tail_not_in_full() {
    let dir = tempdir().unwrap();
    let ws = dir.path();
    write_repro(ws, "001");
    let mut log = "noise line\n".repeat(600);
    log.push_str("one\ntwo\nthree\n");
    write_file(ws, "001", "stderr.log", log.as_bytes());
    let wound = latest_wound(&build_aged(ws, AGENT, AgentState::Stopped));
    assert_eq!(wound, Wound::Spoke("one\ntwo\nthree".to_owned()));
}

/// **The wound reaches a seat**, which is the whole point of the class: the
/// §8.5 answer carries the class token in place of the settled framing, so no
/// seat can render the quiet badge over a step that produced nothing.
#[test]
fn the_wound_reaches_the_answer_in_place_of_the_quiet_framing() {
    let dir = tempdir().unwrap();
    let ws = dir.path();
    write_repro(ws, "001");
    let answered = crate::steps_view::wire::steps(&build_aged(ws, AGENT, AgentState::Stopped));
    let row = &answered["rows"][0];
    assert_eq!(row["wound"], "no_response", "got:\n{answered:#}");
    assert_ne!(row["framing"], "complete", "got:\n{answered:#}");
    assert!(
        Wound::Mute.banner().contains(NO_RESPONSE),
        "and the sentence a seat renders names the class"
    );
}
