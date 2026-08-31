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
//!   `response.json` is *complete and whole* (§4.4, [`Settled::whole`]): last
//!   line `end`, last segment a `finish` with no `error`, and that `finish`
//!   naming a reason the model reached rather than ran out of room at. A
//!   finished-for-now agent awaiting a message (§2.4).
//! - [`AgentState::Stopped`] — no lock held and the latest step is *failed*
//!   (last segment carries an `error`, §2.10) or *killed* (closed with no
//!   trailing `end`, §2.9), or ended at the **output limit** (§4.4
//!   [`Ending::OutputLimit`], bl-fb87), or no step has run. Kill, crash, and
//!   explicit stop are indistinguishable on disk (§2.9); a failed step renders
//!   here too, per §3.5.
//!
//! The output-limit arm is bl-fb87's correction, and it is a §3.5 reading, not
//! a fifth state: transport completion is not task completion, so a tail that
//! framed cleanly around a turn the request's `max_tokens` cut off is a
//! conversation **stopped mid-utterance**, not one at rest. The coarse badge
//! vocabulary is unchanged (the bl-d816 ruling: the badge answers "needs me?",
//! the workspace pane answers "why"), and the *why* rides beside it as
//! [`Liveness::truncated`] — the fact §8.2's Nudge gate reads, because linked
//! litany derives `NothingDue` from exactly this shape and a control that
//! fires and does nothing is QUALITY H4's theater.
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
use super::terminal::{Ending, Settled, settled};
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

impl AgentState {
    /// **Is a driver at work on this agent?** (§3.5) — the one reading of the
    /// two live states, and the gate every derivation that must not accuse a
    /// running driver asks: the §7.3 step wound (a driver at work is still
    /// filling its newest step), the orphaned-mail state, and the §8.1
    /// detached-launch verdict ([`crate::opslog::launch::stillborn`]). It lives on the
    /// state itself because it is a reading of the state and nothing else —
    /// three copies of `matches!(…, Live | InFlight)` is one fact with three
    /// homes, and the third was about to be written.
    pub fn driven(self) -> bool {
        matches!(self, Self::Live | Self::InFlight)
    }
}

/// One agent's §3.5 classification: the three readings the two liveness
/// observations plus the latest step's settled tail yield, gathered in one
/// pass so the response file is read once.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Liveness {
    pub state: AgentState,
    /// A probe returned [`Probe::Unknown`] (DESIGN §10): the state is the best
    /// framing-only reading, not a definite one, and the renderer marks it.
    pub uncertain: bool,
    /// The latest turn was cut off at the **output limit** (§4.4
    /// [`Ending::OutputLimit`], bl-fb87). Read only **at rest**: a driver
    /// holding the lease is itself the answer to "what now", and the step it
    /// is filling has no settled tail to read.
    pub truncated: bool,
    /// The latest turn was **refused at the provider rung** (bl-b43b): the same
    /// bytes, the same pass, the same at-rest gate as
    /// [`truncated`](Self::truncated) — one more reading of the one file this
    /// classification reads, never a second syscall and never a second
    /// mid-write state of it.
    pub refused: bool,
}

/// Classify `agent_id` in `workspace` from the two liveness observations
/// plus the latest step's settled tail.
pub(super) fn classify(
    workspace: &Path,
    agent_id: &str,
    lock: &dyn LockProbe,
    writer: &dyn WriterProbe,
) -> Liveness {
    let inbox_dir = workspace.join(INBOX_DIR).join(agent_id);
    match lock.lock_state(&inbox_dir) {
        // A driver holds the lock: the agent is live; the writer refines it
        // to `in_flight` (and may itself be unable to observe → uncertain).
        Probe::Held => {
            let (state, uncertain) = live_substate(workspace, agent_id, writer);
            Liveness {
                state,
                uncertain,
                truncated: false,
                refused: false,
            }
        }
        // No driver: the terminal-only reading rules (§4.4) settle the file.
        Probe::Free => at_rest(workspace, agent_id, false),
        // Can't tell whether a driver exists (DESIGN §10): degrade to the
        // same framing-only reading, but flag it uncertain — never a false
        // definite `live`/`stopped`.
        Probe::Unknown => at_rest(workspace, agent_id, true),
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
/// complete-and-whole latest `response.json` is `Quiescent`, anything else
/// `Stopped`, and an output-limited tail says so beside the state.
fn at_rest(workspace: &Path, agent_id: &str, uncertain: bool) -> Liveness {
    let (settled, refused) = latest_settled(workspace, agent_id);
    Liveness {
        state: if settled.whole() {
            AgentState::Quiescent
        } else {
            AgentState::Stopped
        },
        uncertain,
        truncated: settled.ending == Ending::OutputLimit,
        refused,
    }
}

fn latest_response_path(workspace: &Path, agent_id: &str) -> Option<PathBuf> {
    let steps = workspace.join(STEPS_DIR).join(agent_id);
    Some(latest_step_dir(&steps)?.join(RESPONSE_FILE))
}

/// The latest step's §4.4 settled reading **and whether it was refused at the
/// provider rung** (bl-b43b) — two readings off one read, for the reason this
/// module already gathers three: reading the file twice could catch two
/// different mid-write states of it. Absence and an unreadable file both read
/// as the *killed* tail they honestly are, and as no refusal — nothing on disk
/// says a provider said no.
fn latest_settled(workspace: &Path, agent_id: &str) -> (Settled, bool) {
    let Some(path) = latest_response_path(workspace, agent_id) else {
        return (Settled::KILLED, false);
    };
    match std::fs::read(&path) {
        Ok(bytes) => (
            settled(&bytes),
            crate::login::auth::classify(&bytes).offered(),
        ),
        Err(_) => (Settled::KILLED, false),
    }
}
