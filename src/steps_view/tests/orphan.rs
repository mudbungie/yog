//! The orphaned-mail state (bl-ace6): the derivation's arms — mail on the
//! tail with nobody driving fires, with the last driver's `driver.log`
//! words when there are any; a driven agent, a settled tail and an
//! absent transcript all stay silent.

use tempfile::tempdir;

use super::AGENT;
use crate::git_tree::AgentState;
use crate::steps_view::{ORPHANED_MAIL, Orphan, build};

/// Lay a transcript whose entries are exactly `names`, in that order.
fn write_messages(ws: &std::path::Path, names: &[&str]) {
    let dir = ws.join("agents").join(AGENT).join("messages");
    std::fs::create_dir_all(&dir).unwrap();
    for name in names {
        std::fs::write(dir.join(name), b"x").unwrap();
    }
}

/// The words a dead driver left behind — the shape lernie's own decline
/// writes (its `advance` erroring to the `driver.log` fd).
const DECLINE: &str = "lernie: branch tip is an assistant entry with tool_use unmatched";

#[test]
fn mail_with_no_driver_and_a_spoken_log_carries_the_words() {
    let dir = tempdir().unwrap();
    let ws = dir.path();
    write_messages(ws, &["001-user.md", "002-opus.json", "003-user.md"]);
    let log = ws.join("steps").join(AGENT);
    std::fs::create_dir_all(&log).unwrap();
    std::fs::write(log.join("driver.log"), DECLINE).unwrap();

    let view = build(ws, AGENT, AgentState::Stopped);
    let Orphan::Spoke(words) = &view.orphan else {
        panic!("expected Spoke, got {:?}", view.orphan);
    };
    assert!(words.contains("unmatched"), "{words}");
    let banner = view.orphan.banner();
    assert!(banner.contains(ORPHANED_MAIL), "{banner}");
    assert!(banner.contains("driver.log"), "{banner}");
    assert!(banner.contains("unmatched"), "{banner}");
}

#[test]
fn mail_with_no_driver_and_no_log_is_mute_and_says_so() {
    let dir = tempdir().unwrap();
    let ws = dir.path();
    write_messages(ws, &["001-user.md"]);

    let view = build(ws, AGENT, AgentState::Quiescent);
    assert_eq!(view.orphan, Orphan::Mute);
    let banner = view.orphan.banner();
    assert!(banner.contains(ORPHANED_MAIL), "{banner}");
    assert!(banner.contains("nothing on disk says why"), "{banner}");
}

#[test]
fn a_settled_tail_is_no_orphan() {
    let dir = tempdir().unwrap();
    let ws = dir.path();
    // Newest entry is model output: the mail was answered.
    write_messages(ws, &["001-user.md", "002-opus.json"]);
    let view = build(ws, AGENT, AgentState::Stopped);
    assert_eq!(view.orphan, Orphan::None);
    assert!(!view.orphan.orphaned());
    assert!(view.orphan.banner().is_empty());
}

#[test]
fn a_driven_agent_is_never_an_orphan() {
    // The relaunch gap's steady half: a driver holds the lock, so the
    // mail on the tail is about to be answered — never a false definite.
    let dir = tempdir().unwrap();
    let ws = dir.path();
    write_messages(ws, &["001-user.md"]);
    for state in [AgentState::Live, AgentState::InFlight] {
        assert_eq!(build(ws, AGENT, state).orphan, Orphan::None);
    }
}

#[test]
fn an_absent_transcript_is_no_orphan() {
    let dir = tempdir().unwrap();
    let view = build(dir.path(), AGENT, AgentState::Stopped);
    assert_eq!(view.orphan, Orphan::None);
}
