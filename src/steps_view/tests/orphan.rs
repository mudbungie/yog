//! The orphaned-tail state's **mail** shape (bl-ace6): the derivation's arms
//! — mail on the tail with nobody driving fires, with the last driver's
//! `driver.log` words when there are any; a driven agent, a settled tail and
//! an absent transcript all stay silent.
//!
//! The state's other shape — a tool window an executor died inside (bl-abba)
//! — is [`super::window`], split at §12's budget on the seam the two ball
//! bodies already draw. The fixture writers both shapes need live here, since
//! this is the file that laid the transcript down first.

use tempfile::tempdir;

use super::AGENT;
use crate::git_tree::AgentState;
use crate::steps_view::{ORPHANED_MAIL, Orphan, Tail, build_aged};

/// Lay a transcript whose entries are exactly `names`, in that order.
pub(super) fn write_messages(ws: &std::path::Path, names: &[&str]) {
    let dir = messages(ws);
    for name in names {
        std::fs::write(dir.join(name), b"x").unwrap();
    }
}

/// The agent's `messages/` directory, created.
pub(super) fn messages(ws: &std::path::Path) -> std::path::PathBuf {
    let dir = ws.join("agents").join(AGENT).join("messages");
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// The words a dead driver left behind — the shape litany's own decline
/// writes (its `advance` erroring to the `driver.log` fd).
pub(super) const DECLINE: &str = "litany: branch tip is an assistant entry with tool_use unmatched";

/// Lay the dead driver's words beside the steps.
pub(super) fn write_driver_log(ws: &std::path::Path, words: &str) {
    let log = ws.join("steps").join(AGENT);
    std::fs::create_dir_all(&log).unwrap();
    std::fs::write(log.join("driver.log"), words).unwrap();
}

#[test]
fn mail_with_no_driver_and_a_spoken_log_carries_the_words() {
    let dir = tempdir().unwrap();
    let ws = dir.path();
    write_messages(ws, &["001-user.md", "002-opus.json", "003-user.md"]);
    write_driver_log(ws, DECLINE);

    let view = build_aged(ws, AGENT, AgentState::Stopped);
    let Orphan::Spoke(Tail::Mail, words) = &view.orphan else {
        panic!("expected Spoke(Mail), got {:?}", view.orphan);
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

    let view = build_aged(ws, AGENT, AgentState::Quiescent);
    assert_eq!(view.orphan, Orphan::Mute(Tail::Mail));
    let banner = view.orphan.banner();
    assert!(banner.contains(ORPHANED_MAIL), "{banner}");
    assert!(banner.contains("nothing on disk says why"), "{banner}");
}

#[test]
fn a_settled_tail_is_no_orphan() {
    let dir = tempdir().unwrap();
    let ws = dir.path();
    // Newest entry is model output whose bytes yog cannot classify: the mail
    // was answered, and bytes nobody can read assert nothing.
    write_messages(ws, &["001-user.md", "002-opus.json"]);
    let view = build_aged(ws, AGENT, AgentState::Stopped);
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
        assert_eq!(build_aged(ws, AGENT, state).orphan, Orphan::None);
    }
}

#[test]
fn an_absent_transcript_is_no_orphan() {
    let dir = tempdir().unwrap();
    let view = build_aged(dir.path(), AGENT, AgentState::Stopped);
    assert_eq!(view.orphan, Orphan::None);
}

/// A `messages/` directory that exists and holds nothing has no tail at all
/// — a conversation authored but never spoken to. The general path with an
/// empty listing, not a bootstrap case.
#[test]
fn an_empty_transcript_directory_is_no_orphan() {
    let dir = tempdir().unwrap();
    let ws = dir.path();
    messages(ws);
    assert_eq!(
        build_aged(ws, AGENT, AgentState::Stopped).orphan,
        Orphan::None
    );
}
