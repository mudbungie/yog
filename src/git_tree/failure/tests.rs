//! The two shapes of a failed model call, and the clause a row says of each
//! (bl-9b88).

use super::{clause, failure};
use crate::git_tree::terminal::settled;
use crate::steps_view::records::STDERR_FILE;
use std::path::Path;

/// A settled tail whose last segment carries an in-band `error` — what brazen
/// writes when it reached the provider and the provider said no.
const FAILED: &[u8] = br#"{"type":"message_start","v":1,"role":"assistant"}
{"type":"error","status":401,"message":"Unauthorized"}
{"type":"end"}
"#;

/// A whole one, for the arm that must pay no syscall at all.
const WHOLE: &[u8] = br#"{"type":"message_start","v":1,"role":"assistant"}
{"type":"finish","reason":"stop"}
{"type":"end"}
"#;

fn read(step: &Path, response: &[u8]) -> Option<String> {
    failure(step, response, settled(response))
}

/// The in-band half: the `error` event verbatim, because the auth heuristic
/// scans the whole line — status code and reason phrase alike.
#[test]
fn an_in_band_error_is_the_event_line_itself() {
    let dir = tempfile::TempDir::new().unwrap();
    let said = read(dir.path(), FAILED).expect("the call failed");
    assert!(said.contains(r#""status":401"#), "{said}");
    assert!(said.contains("Unauthorized"), "{said}");
}

/// The out-of-band half — the live sighting's own shape (bl-9b88). The adapter
/// died before it reached the contract, so `response.json` is empty and the
/// only words anywhere are its `stderr.log`.
#[test]
fn an_adapter_that_died_before_the_contract_says_so_from_its_stderr() {
    let dir = tempfile::TempDir::new().unwrap();
    std::fs::write(
        dir.path().join(STDERR_FILE),
        "bz: no credential for provider row \"work\"\n",
    )
    .unwrap();
    let said = read(dir.path(), b"").expect("the call failed");
    assert!(said.contains("no credential for provider row"), "{said}");
}

/// A tail that framed cleanly is not a failure, whatever else is on disk — and
/// the `stderr.log` beside it is never opened, which is what keeps a healthy
/// conversation free.
#[test]
fn a_complete_tail_is_no_failure_and_reads_no_stderr() {
    let dir = tempfile::TempDir::new().unwrap();
    std::fs::write(dir.path().join(STDERR_FILE), "stale words\n").unwrap();
    assert_eq!(read(dir.path(), WHOLE), None);
}

/// A driver killed mid-call leaves an unterminated tail and no words: the
/// honest answer is that nothing on disk says why, never an invented sentence.
#[test]
fn a_killed_call_with_no_words_is_no_sentence() {
    let dir = tempfile::TempDir::new().unwrap();
    assert_eq!(read(dir.path(), b"{\"type\":\"delta\"}\n"), None);
}

/// The row's clause prefers the provider's `message` — an operator reads a
/// sentence, not a wire frame.
#[test]
fn the_clause_is_the_providers_message_when_the_evidence_is_an_event() {
    assert_eq!(
        clause(r#"{"type":"error","status":401,"message":"Unauthorized"}"#),
        "Unauthorized"
    );
}

/// An event naming only a status says itself: as much as is known, never less.
#[test]
fn an_event_with_no_message_says_itself() {
    let raw = r#"{"type":"error","status":503}"#;
    assert_eq!(clause(raw), raw);
    // …and so does one whose `message` is blank, which is a message in name only.
    let blank = r#"{"type":"error","message":"  "}"#;
    assert_eq!(clause(blank), blank);
}

/// Plain adapter stderr is not JSON at all: the first non-empty line of it is
/// the clause, capped the way a §11 preview is.
#[test]
fn plain_stderr_gives_its_first_line_capped() {
    assert_eq!(
        clause("\n\nbz: malformed config\n  at line 3\n"),
        "bz: malformed config"
    );
    let long = "x".repeat(400);
    assert_eq!(clause(&long).chars().count(), super::CLAUSE_CAP);
}

/// Evidence with nothing readable in it at all still answers a string, never a
/// panic — the general path with empty inputs.
#[test]
fn empty_evidence_is_an_empty_clause() {
    assert_eq!(clause("   \n"), "");
}
