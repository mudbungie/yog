//! Pure-function unit tests for the detect + cmd layers.

use crate::git_tree::GitTreeError;
use crate::git_tree::cmd::{parse_log, parse_step_commits};
use crate::git_tree::detect::{PREVIEW_MAX, payload_headline, truncate_preview};

#[test]
fn truncate_preview_passes_short_input_through() {
    assert_eq!(truncate_preview("hi"), "hi");
}

#[test]
fn truncate_preview_collapses_whitespace_and_trims() {
    assert_eq!(truncate_preview("  a\n\tb  "), "a  b");
}

#[test]
fn truncate_preview_cuts_long_input_with_ellipsis() {
    let long = "x".repeat(PREVIEW_MAX + 20);
    let out = truncate_preview(&long);
    let last = out.chars().last().unwrap();
    assert_eq!(last, '…');
    assert_eq!(out.chars().count(), PREVIEW_MAX);
}

#[test]
fn payload_headline_is_the_goals_first_line() {
    assert_eq!(payload_headline("hi v03"), "hi v03");
}

/// The §3.3 ladder's second rung at its source: the identity stamp comes off
/// before anything collapses, and what is left is the payload's headline — so a
/// stamped conversation never previews as its own identity line.
#[test]
fn payload_headline_strips_the_identity_stamp_and_keeps_the_headline() {
    assert_eq!(
        payload_headline("You are stench-pug.\n\nBall bl-1: fix\n\nthe body runs on"),
        "Ball bl-1: fix"
    );
    // Unstamped (foreign / hand-typed): the goal is its own payload, and its
    // first non-blank line is the headline.
    assert_eq!(
        payload_headline("\n\nwire the gate\nsecond line"),
        "wire the gate"
    );
    // A stamp and nothing else previews as nothing — the row falls to rung three.
    assert_eq!(payload_headline("You are stench-pug."), "");
}

/// An empty `goal.md` is a goal that says nothing, not an absent one — absence
/// is the read's own `None` (see the `goal.md`-removed repo test).
#[test]
fn payload_headline_of_an_empty_goal_is_empty() {
    assert_eq!(payload_headline(""), "");
}

#[test]
fn parse_log_errors_on_line_missing_subject_separator() {
    // No `\x00` between the parent column and the subject — every log
    // entry must include the merge subject for v0.3.1 detection.
    let err = parse_log(b"only-one-token\n").unwrap_err();
    assert!(matches!(err, GitTreeError::LogFormat(_)), "{err:?}");
}

#[test]
fn parse_log_errors_on_line_missing_timestamp() {
    let err = parse_log(b"only-one-token\x00subject\n").unwrap_err();
    assert!(matches!(err, GitTreeError::LogFormat(_)), "{err:?}");
}

#[test]
fn parse_log_errors_on_non_numeric_timestamp() {
    let err = parse_log(b"abc notanumber\x00subject\n").unwrap_err();
    assert!(matches!(err, GitTreeError::LogFormat(_)), "{err:?}");
}

#[test]
fn parse_log_parses_commit_line() {
    let out = parse_log(b"abc 100\x00scaffold\n").unwrap();
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].oid, "abc");
    assert_eq!(out[0].timestamp, 100);
    assert_eq!(out[0].subject, "scaffold");
}

#[test]
fn parse_log_keeps_spaces_in_subject() {
    let out = parse_log(b"abc 100\x00Merge branch 'foo'\n").unwrap();
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].subject, "Merge branch 'foo'");
}

#[test]
fn parse_step_commits_parses_valid_lines() {
    let out = parse_step_commits(b"abc 100\x00dispatch [x]\ndef 200\x00delivery: user\n").unwrap();
    assert_eq!(out.len(), 2);
    assert_eq!(out[0].oid, "abc");
    assert_eq!(out[0].timestamp_unix, 100);
    assert_eq!(out[0].subject, "dispatch [x]");
    assert_eq!(out[1].timestamp_unix, 200);
    assert_eq!(out[1].subject, "delivery: user");
}

#[test]
fn parse_step_commits_errors_on_missing_subject_separator() {
    // No `\x00` between the timestamp column and the subject.
    let err = parse_step_commits(b"abc 100\n").unwrap_err();
    assert!(matches!(err, GitTreeError::LogFormat(_)), "{err:?}");
}

#[test]
fn parse_step_commits_errors_on_missing_timestamp() {
    let err = parse_step_commits(b"abc\x00subject\n").unwrap_err();
    assert!(matches!(err, GitTreeError::LogFormat(_)), "{err:?}");
}

#[test]
fn parse_step_commits_errors_on_non_numeric_timestamp() {
    let err = parse_step_commits(b"abc notanumber\x00subject\n").unwrap_err();
    assert!(matches!(err, GitTreeError::LogFormat(_)), "{err:?}");
}

#[test]
fn error_display_for_log_format() {
    let e = GitTreeError::LogFormat("oops".into());
    let msg = e.to_string();
    assert!(msg.contains("malformed"));
}
