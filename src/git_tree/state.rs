//! Agent-state classifier (ARCH §3.5 / §7.1, terminal rules §4.4).
//!
//! The four live-view states are derived from the executor lock, the
//! agent's latest-step `response.json`, and nothing stored (PRINCIPLES
//! "Single source of truth"):
//!
//! - [`AgentState::Live`] — a driver holds the agent's inbox-directory lock
//!   (§2.11): someone is stepping the branch.
//! - [`AgentState::InFlight`] — the `live` sub-state where a model call is
//!   in flight *right now*: the latest step's `response.json` fd is still
//!   open (§3.5, §4.4). The harness holds that fd across every retry
//!   attempt and backoff sleep, so a mid-retry `end` segment is still
//!   in_flight, never stopped.
//! - [`AgentState::Quiescent`] — no lock held and the latest step's
//!   `response.json` is *complete* (§4.4): last line `end`, last segment a
//!   `finish` with no `error`. A finished-for-now agent awaiting a message
//!   (§2.4).
//! - [`AgentState::Stopped`] — no lock held and the latest step is *failed*
//!   (last segment carries an `error`, §2.10) or *killed* (closed with no
//!   trailing `end`, §2.9), or no step has run. Kill, crash, and explicit
//!   stop are indistinguishable on disk (§2.9); a failed step renders here
//!   too, per §3.5.
//!
//! The two observations are deliberately not collapsed (§2.11): the lock is
//! *is-anyone-driving*; the open `response.json` fd is
//! *is-a-model-call-in-flight-right-now*.
//!
//! Each observation is a tri-state [`Probe`] (DESIGN §10): a backend that
//! cannot look (macOS `lsof` missing) returns `Unknown`. On `Unknown`,
//! [`classify`] degrades to the framing-only reading (quiescent/stopped) and
//! returns an **uncertainty flag** so the renderer marks it "live?" — never a
//! false definite. The Linux procfs backends never return `Unknown`, so the
//! flag is always `false` here.

use std::path::{Path, PathBuf};

use super::probe::{LockProbe, Probe, WriterProbe};
use super::streaming::latest_step_dir;
use super::terminal::last_segment_complete;
use super::{INBOX_DIR, STEPS_DIR};

pub(super) const RESPONSE_FILE: &str = "response.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentState {
    /// The executor holds the agent's inbox-directory lock (§2.11) but no
    /// model call's `response.json` fd is open — a driver between steps or
    /// running a tool.
    Live,
    /// `live` refined: a model call is in flight right now — the latest
    /// step's `response.json` fd is still open (§3.5, §4.4).
    InFlight,
    /// No lock held; the latest step's `response.json` is a clean,
    /// complete model call (§4.4) — awaiting a message (§2.4).
    Quiescent,
    /// No lock held; the latest step failed, was killed, or never ran
    /// (§2.9, §2.10). Kill/crash/explicit stop are indistinguishable here.
    Stopped,
}

/// Classify `agent_id` in `workspace` from the two liveness observations
/// plus the latest step's terminal framing. Returns the [`AgentState`] and an
/// **uncertainty flag** — `true` when a probe returned [`Probe::Unknown`]
/// (DESIGN §10), meaning the state is the best framing-only reading, not a
/// definite one.
pub(super) fn classify(
    workspace: &Path,
    agent_id: &str,
    lock: &dyn LockProbe,
    writer: &dyn WriterProbe,
) -> (AgentState, bool) {
    let inbox_dir = workspace.join(INBOX_DIR).join(agent_id);
    match lock.lock_state(&inbox_dir) {
        // A driver holds the lock: the agent is live; the writer refines it
        // to `in_flight` (and may itself be unable to observe → uncertain).
        Probe::Held => live_substate(workspace, agent_id, writer),
        // No driver: the terminal-only reading rules (§4.4) settle the file.
        Probe::Free => (framing_state(workspace, agent_id), false),
        // Can't tell whether a driver exists (DESIGN §10): degrade to the
        // same framing-only reading, but flag it uncertain — never a false
        // definite `live`/`stopped`.
        Probe::Unknown => (framing_state(workspace, agent_id), true),
    }
}

/// Under the lock: `InFlight` iff a writer still holds the latest step's
/// `response.json` open (a model call right now), else plain `Live`. A writer
/// that cannot observe ([`Probe::Unknown`]) leaves the state `Live` (the lock
/// is definitely held) but flags uncertainty — the in_flight refinement is
/// undetectable.
fn live_substate(workspace: &Path, agent_id: &str, writer: &dyn WriterProbe) -> (AgentState, bool) {
    match latest_response_path(workspace, agent_id) {
        Some(path) => match writer.writer_state(&path) {
            Probe::Held => (AgentState::InFlight, false),
            Probe::Free => (AgentState::Live, false),
            Probe::Unknown => (AgentState::Live, true),
        },
        // Lock held but no step yet (pre-first-call): plainly `Live`.
        None => (AgentState::Live, false),
    }
}

/// No (observable) lock: the §4.4 terminal-only rules settle the file — a
/// complete latest `response.json` is `Quiescent`, anything else `Stopped`.
fn framing_state(workspace: &Path, agent_id: &str) -> AgentState {
    if latest_response_complete(workspace, agent_id) {
        AgentState::Quiescent
    } else {
        AgentState::Stopped
    }
}

fn latest_response_path(workspace: &Path, agent_id: &str) -> Option<PathBuf> {
    let steps = workspace.join(STEPS_DIR).join(agent_id);
    Some(latest_step_dir(&steps)?.join(RESPONSE_FILE))
}

/// Is the latest step's `response.json` *complete* (§4.4)? Reads the file
/// once; absence or an unreadable file is not complete.
fn latest_response_complete(workspace: &Path, agent_id: &str) -> bool {
    let Some(path) = latest_response_path(workspace, agent_id) else {
        return false;
    };
    match std::fs::read(&path) {
        Ok(bytes) => last_segment_complete(&bytes),
        Err(_) => false,
    }
}
