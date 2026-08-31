//! Probe-injected unit tests for the §3.5 classifier's tri-state mapping.
//!
//! Every row of the (lock × writer × framing) table is exercised with
//! injected [`Probe`] stubs — the `live`/`in_flight` states need a driver
//! holding the lock and cannot be reached against a dead fixture, so they are
//! proven here rather than in [`super::state_repo`]. Each case asserts both
//! the [`AgentState`], the DESIGN §10 uncertainty flag (`Unknown` degrades to
//! a framing-only reading that is flagged, never a false definite) and the
//! §4.4 output-limit reading beside it, which is `false` on every row here —
//! the reading's own cases live in [`super::state_truncated`] (bl-fb87).

use crate::git_tree::probe::{LockProbe, WriterProbe};
use crate::git_tree::state::{RESPONSE_FILE, classify};
use crate::git_tree::{AgentState, Probe, STEPS_DIR};
use std::path::{Path, PathBuf};
use tempfile::tempdir;

/// Stub probes returning a fixed tri-state answer — one per trait so lock and
/// writer observations vary independently across the mapping rows.
pub(super) struct LockStub(Probe);
impl LockProbe for LockStub {
    fn lock_state(&self, _dir: &Path) -> Probe {
        self.0
    }
}
pub(super) struct WriterStub(Probe);
impl WriterProbe for WriterStub {
    fn writer_state(&self, _path: &Path) -> Probe {
        self.0
    }
}

pub(super) fn lock(p: Probe) -> LockStub {
    LockStub(p)
}
pub(super) fn writer(p: Probe) -> WriterStub {
    WriterStub(p)
}

pub(super) fn write(path: &Path, contents: &[u8]) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(path, contents).unwrap();
}

/// The classification as a comparable triple: state, §10 uncertainty, and the
/// §4.4 output-limit reading.
pub(super) fn reading(
    dir: &Path,
    agent: &str,
    lock: &dyn LockProbe,
    writer: &dyn WriterProbe,
) -> (AgentState, bool, bool) {
    let read = classify(dir, agent, lock, writer);
    (read.state, read.uncertain, read.truncated)
}

/// The **provider-refusal** reading beside the state (bl-b43b) — its own
/// helper rather than a fourth element on [`reading`], so the mapping table
/// this module owns keeps the shape every one of its cases is written to.
pub(super) fn refusal(
    dir: &Path,
    agent: &str,
    lock: &dyn LockProbe,
    writer: &dyn WriterProbe,
) -> bool {
    classify(dir, agent, lock, writer).refused
}

pub(super) fn resp(dir: &Path, agent: &str, seq: &str) -> PathBuf {
    dir.join(format!("{STEPS_DIR}/{agent}/{seq}/{RESPONSE_FILE}"))
}

pub(super) const FINISH_END: &[u8] = br#"{"type":"message_start","v":1,"role":"assistant"}
{"type":"finish","reason":"stop"}
{"type":"end"}
"#;
const ERROR_END: &[u8] = br#"{"type":"message_start","v":1,"role":"assistant"}
{"type":"error","kind":"transport","message":"reset"}
{"type":"end"}
"#;

#[test]
fn lock_held_and_writer_held_is_in_flight() {
    let dir = tempdir().unwrap();
    let agent = "20260427T140000Z-aaaa";
    write(&resp(dir.path(), agent, "001"), FINISH_END);
    // Lock Held + writer Held → InFlight, certain.
    assert_eq!(
        reading(dir.path(), agent, &lock(Probe::Held), &writer(Probe::Held)),
        (AgentState::InFlight, false, false)
    );
}

#[test]
fn lock_held_and_writer_free_is_live() {
    let dir = tempdir().unwrap();
    let agent = "20260427T140000Z-bbbb";
    write(&resp(dir.path(), agent, "001"), FINISH_END);
    // Lock Held + writer Free → Live (between calls), certain.
    assert_eq!(
        reading(dir.path(), agent, &lock(Probe::Held), &writer(Probe::Free)),
        (AgentState::Live, false, false)
    );
}

#[test]
fn lock_held_and_writer_unknown_is_live_uncertain() {
    // Lock Held (a driver is definitely present) but the writer backend
    // cannot observe: the agent is Live, yet the in_flight refinement is
    // undetectable → uncertain (DESIGN §10).
    let dir = tempdir().unwrap();
    let agent = "20260427T140000Z-uuuu";
    write(&resp(dir.path(), agent, "001"), FINISH_END);
    assert_eq!(
        reading(
            dir.path(),
            agent,
            &lock(Probe::Held),
            &writer(Probe::Unknown)
        ),
        (AgentState::Live, true, false)
    );
}

#[test]
fn lock_held_with_no_response_is_live() {
    // A driver that acquired the lock but has not opened a response.json yet
    // (pre-first-call) is Live, not InFlight.
    let dir = tempdir().unwrap();
    let agent = "20260427T140000Z-cccc";
    std::fs::create_dir_all(dir.path().join(STEPS_DIR).join(agent)).unwrap();
    assert_eq!(
        reading(dir.path(), agent, &lock(Probe::Held), &writer(Probe::Held)),
        (AgentState::Live, false, false)
    );
}

#[test]
fn no_lock_and_complete_response_is_quiescent() {
    let dir = tempdir().unwrap();
    let agent = "20260427T140000Z-dddd";
    write(&resp(dir.path(), agent, "001"), FINISH_END);
    assert_eq!(
        reading(dir.path(), agent, &lock(Probe::Free), &writer(Probe::Free)),
        (AgentState::Quiescent, false, false)
    );
}

#[test]
fn no_lock_and_failed_response_is_stopped() {
    let dir = tempdir().unwrap();
    let agent = "20260427T140000Z-eeee";
    write(&resp(dir.path(), agent, "001"), ERROR_END);
    assert_eq!(
        reading(dir.path(), agent, &lock(Probe::Free), &writer(Probe::Free)),
        (AgentState::Stopped, false, false)
    );
}

#[test]
fn no_lock_and_no_response_is_stopped() {
    let dir = tempdir().unwrap();
    assert_eq!(
        reading(
            dir.path(),
            "no-such-agent",
            &lock(Probe::Free),
            &writer(Probe::Free)
        ),
        (AgentState::Stopped, false, false)
    );
}

#[test]
fn lock_unknown_and_complete_response_is_quiescent_uncertain() {
    // Lock backend cannot observe: fall back to framing (complete →
    // quiescent) but flag uncertainty, never a false definite (§10).
    let dir = tempdir().unwrap();
    let agent = "20260427T140000Z-qqqq";
    write(&resp(dir.path(), agent, "001"), FINISH_END);
    assert_eq!(
        reading(
            dir.path(),
            agent,
            &lock(Probe::Unknown),
            &writer(Probe::Free)
        ),
        (AgentState::Quiescent, true, false)
    );
}

#[test]
fn lock_unknown_and_incomplete_response_is_stopped_uncertain() {
    let dir = tempdir().unwrap();
    let agent = "20260427T140000Z-ssss";
    write(&resp(dir.path(), agent, "001"), ERROR_END);
    assert_eq!(
        reading(
            dir.path(),
            agent,
            &lock(Probe::Unknown),
            &writer(Probe::Free)
        ),
        (AgentState::Stopped, true, false)
    );
}

#[test]
fn classify_reads_latest_step_only() {
    let dir = tempdir().unwrap();
    let agent = "20260427T140000Z-ffff";
    write(&resp(dir.path(), agent, "001"), FINISH_END);
    // Latest step is mid-stream (no terminal) → not complete → Stopped with
    // no lock.
    write(
        &resp(dir.path(), agent, "002"),
        b"{\"type\":\"content_delta\",\"index\":0,\"delta\":{\"text_delta\":\"go\"}}\n",
    );
    assert_eq!(
        reading(dir.path(), agent, &lock(Probe::Free), &writer(Probe::Free)),
        (AgentState::Stopped, false, false)
    );
}
