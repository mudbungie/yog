//! The ops row is the monitor's only durable, so its round trip is the whole
//! contract: what a check writes must read back as the same standing verdict
//! and the same last-checked sha, and nothing else on the trail may be mistaken
//! for one.

use super::*;
use crate::monitor::Verdict;
use std::path::Path;

fn check(agent: &str, verdict: Verdict, sha: &str) -> Check {
    Check {
        workspace: "/ws".to_owned(),
        agent: agent.to_owned(),
        verdict,
        sha: sha.to_owned(),
        reason: "because".to_owned(),
        model: "haiku".to_owned(),
        input_tokens: Some(120),
        output_tokens: Some(9),
    }
}

#[test]
fn a_check_round_trips_through_its_durable_line() {
    let written = check("a-1", Verdict::Drifting, "abc123");
    let line = entry("42".to_owned(), &written);
    assert_eq!(line.exit, 0, "a completed check is not a failure");
    assert_eq!(line.origin, Origin::World);
    assert_eq!(
        of_entries(std::slice::from_ref(&line)),
        vec![written.clone()]
    );
    // And through the render side's pre-joined argv, which must be lossless.
    assert_eq!(of_rows(&[OpRow::from(&line)]), vec![written]);
}

#[test]
fn an_unreported_counter_stays_absent_rather_than_zero() {
    let mut written = check("a-1", Verdict::Aligned, "abc123");
    written.input_tokens = None;
    written.output_tokens = None;
    let line = entry("42".to_owned(), &written);
    assert!(line.argv.contains(&ABSENT.to_owned()));
    assert_eq!(of_entries(&[line]), vec![written]);
}

#[test]
fn nothing_else_on_the_trail_reads_as_a_check() {
    let other = OpEntry {
        ts: "1".to_owned(),
        argv: vec!["bl".to_owned(), "close".to_owned(), "bl-1".to_owned()],
        cwd: "/p".to_owned(),
        exit: 0,
        stdout: String::new(),
        stderr: String::new(),
        origin: Origin::Balls,
    };
    assert!(of_entries(&[other]).is_empty());
    // Right pseudo-binary, wrong arity, and an unreadable verdict token.
    let short = OpEntry {
        argv: vec![YOG_MONITOR.to_owned(), "aligned".to_owned()],
        ..OpEntry::default()
    };
    assert!(of_entries(&[short]).is_empty());
    let unreadable = OpEntry {
        argv: vec![YOG_MONITOR.to_owned(); 7],
        ..OpEntry::default()
    };
    assert!(of_entries(&[unreadable]).is_empty());
    // Right arity, wrong pseudo-binary: a future yog's seven-token row is not
    // a check either, and reading it as one would invent a verdict.
    let foreign = OpEntry {
        argv: vec!["yog-something".to_owned(); 7],
        ..OpEntry::default()
    };
    assert!(of_entries(&[foreign]).is_empty());
    // And a flag row, which is a different assertion with a different argv[0].
    let flag = flagged("1".to_owned(), Path::new("/ws"), "a-1", "look at this");
    assert!(of_entries(std::slice::from_ref(&flag)).is_empty());
    assert_eq!(flag.exit, 0, "raising attention is not a failure");
    assert_eq!(flag.origin, Origin::Conversation);
    assert_eq!(flag.stdout, "look at this");
}

#[test]
fn a_failed_check_names_no_sha_so_the_tip_stays_unchecked() {
    let line = failure("7".to_owned(), Path::new("/ws"), "a-1", "no credentials");
    assert!(of_entries(std::slice::from_ref(&line)).is_empty());
    assert_eq!(line.exit, crate::opslog::SYNTHETIC_EXIT);
    assert!(line.stderr.contains("a-1") && line.stderr.contains("no credentials"));
    assert_eq!(line.origin, Origin::World, "no banner for the monitor");
}

#[test]
fn the_latest_row_per_agent_is_the_standing_verdict() {
    let rows = vec![
        check("a-1", Verdict::Aligned, "old"),
        check("a-2", Verdict::Diverged, "other"),
        check("a-1", Verdict::Drifting, "new"),
    ];
    let found = latest(&rows, "/ws", "a-1").expect("checked");
    assert_eq!(
        (found.verdict, found.sha.as_str()),
        (Verdict::Drifting, "new")
    );
    assert_eq!(
        latest(&rows, "/elsewhere", "a-1"),
        None,
        "keyed by workspace"
    );
    assert_eq!(latest(&rows, "/ws", "a-3"), None, "never checked");
}

#[test]
fn a_conversations_verdict_is_the_worst_of_its_members() {
    let rows = vec![
        check("a-1", Verdict::Aligned, "x"),
        check("a-1-b", Verdict::Diverged, "y"),
        check("a-1-c", Verdict::Drifting, "z"),
    ];
    let members: Vec<String> = ["a-1", "a-1-b", "a-1-c"]
        .iter()
        .map(|s| (*s).to_owned())
        .collect();
    assert_eq!(
        worst(&rows, "/ws", &members).expect("checked").verdict,
        Verdict::Diverged,
        "a diverged child is the conversation's fact"
    );
    assert_eq!(worst(&rows, "/ws", &["nobody".to_owned()]), None);
}

#[test]
fn a_runaway_reason_is_clipped_and_flattened() {
    let mut written = check("a-1", Verdict::Aligned, "abc");
    written.reason = format!("line one\nline two{}", "x".repeat(REASON_MAX));
    let line = entry("1".to_owned(), &written);
    assert_eq!(line.stdout.chars().count(), REASON_MAX);
    assert!(!line.stdout.contains('\n'), "one row, one line");
}
