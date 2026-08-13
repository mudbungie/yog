//! The level trigger and the thread it runs on (VISION §4.9).
//!
//! **Level-triggered off the step spine, by yog's clock.** A tick checks an
//! armed agent only when its branch tip has moved past the sha its last check
//! named — so a quiet fleet costs nothing, and a fleet that never rests is
//! still checked at most once per tip per tick. The comparison reads the
//! last-checked sha off the ops rows themselves; there is no cursor anywhere.
//!
//! **This is the whole retry mechanism.** A check that fails writes a failure
//! row that names no sha, so the tip stays unchecked and the next tick simply
//! re-fires. That is the anti-reinvention law in code: no backoff, no attempt
//! counter, no queue.
//!
//! **Never the frame, and never the derivation worker either.** A check is an
//! HTTPS call measured in seconds; the worker's pass is a correctness floor
//! measured against the cheap-sweep cadence (§7.2), and a call inside it would
//! read as yog being late. So this is its own thread, in the shape the worker
//! and the gestures consumer already use: a stop flag, a park loop, a `Drop`
//! that joins. All the logic is [`SentryCtx::pass`], which a test drives
//! directly.
//!
//! Its period is the clock's own **full-sweep** cadence, re-read from
//! `cadence.yaml` each turn: the monitor ticks with the slowest thing yog does,
//! because a checkpoint is a step boundary and not a poll.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;
use std::time::Duration;

use super::{Caller, arming, check, row, window};
use crate::app::cadence::{self, CADENCE_YAML};
use crate::git_tree::Agent;
use crate::opslog;
use crate::state::SnapshotCell;
use crate::ui_state::Clock;

/// How many durable rows the level trigger reads back. Generous, because the
/// trail carries every yog action and a monitor row must not fall out of the
/// window between two checkpoints; a row that *does* age out costs one repeated
/// check, never a wrong verdict.
const TAIL: usize = 4000;

/// What the sentry thread needs: the yog state root (which holds
/// `cadence.yaml`, the policy file and `ops.jsonl`), the snapshot the worker
/// publishes, the clock, and the one effect — the model call.
pub struct SentryCtx {
    pub state_root: PathBuf,
    pub cell: SnapshotCell,
    pub clock: Arc<dyn Clock>,
    pub caller: Box<dyn Caller>,
}

impl SentryCtx {
    /// The tick period — the clock's full-sweep cadence, off the same file
    /// arming rides, so tuning the clock tunes the monitor with it.
    pub fn period(&self) -> Duration {
        cadence::parse(&self.settings()).full_sweep
    }

    /// One tick: fire at most one check, and only for an agent whose tip has
    /// moved. Returns whether a check was made — a test's assertion, and
    /// nothing the thread needs.
    pub fn pass(&self) -> bool {
        let settings = self.settings();
        if arming::armed(&settings).is_empty() {
            return false;
        }
        let snapshot = crate::state::latest_snapshot(&self.cell);
        let checks = row::of_entries(&opslog::tail(&self.state_root, TAIL));
        for workspace in &snapshot.workspaces {
            let key = crate::nav::ws_key(&workspace.path);
            let Some(watch) = arming::watch(&settings, &key) else {
                continue;
            };
            // A named policy file that is missing or empty is **unarmed**: the
            // prompt is the mechanism's policy, so severing it severs the
            // mechanism rather than falling back to a compiled-in opinion.
            let policy =
                std::fs::read_to_string(self.state_root.join(&watch.prompt)).unwrap_or_default();
            if policy.trim().is_empty() {
                continue;
            }
            let Some(tree) = snapshot.trees.get(&workspace.path) else {
                continue;
            };
            if let Some(agent) = tree
                .agents
                .iter()
                .find(|a| due(&checks, &workspace.path, a))
            {
                self.fire(&workspace.path, agent, &watch, &policy, &checks);
                return true;
            }
        }
        false
    }

    /// Make one check and leave exactly one row behind, whichever way it went.
    fn fire(
        &self,
        workspace: &Path,
        agent: &Agent,
        watch: &arming::Watch,
        policy: &str,
        checks: &[row::Check],
    ) {
        let standing = row::latest(checks, &crate::nav::ws_key(workspace), &agent.agent_id);
        let evidence = window::gather(
            workspace,
            &agent.agent_id,
            standing.as_ref().map(|c| c.sha.as_str()),
            &agent.tip_oid,
        );
        let request = check::request(&evidence, standing.map(|c| c.verdict));
        let ts = self.clock.stamp();
        let entry = match check::run(&*self.caller, workspace, watch, policy, &request) {
            Ok(answer) => row::entry(
                ts,
                &row::Check {
                    workspace: crate::nav::ws_key(workspace),
                    agent: agent.agent_id.clone(),
                    verdict: answer.reply.verdict,
                    sha: agent.tip_oid.clone(),
                    reason: answer.reply.reason,
                    model: watch.model.clone(),
                    input_tokens: answer.input_tokens,
                    output_tokens: answer.output_tokens,
                },
            ),
            Err(why) => row::failure(ts, workspace, &agent.agent_id, &why),
        };
        // A trail yog cannot write is not a reason to make the call again on
        // the next tick with no record; the level trigger will do that anyway.
        let _ = opslog::append(&self.state_root, &entry);
    }

    /// `cadence.yaml`'s bytes, or none. Total, exactly as the clock's own read
    /// is: an absent or unreadable file arms nothing and leaves the default
    /// period standing.
    fn settings(&self) -> String {
        std::fs::read_to_string(self.state_root.join(CADENCE_YAML)).unwrap_or_default()
    }
}

/// Has this agent's branch tip moved past the sha its last check named? An
/// agent never checked is due by the same rule with an empty baseline.
fn due(checks: &[row::Check], workspace: &Path, agent: &Agent) -> bool {
    row::latest(checks, &crate::nav::ws_key(workspace), &agent.agent_id).map(|c| c.sha)
        != Some(agent.tip_oid.clone())
}

/// The sentry thread. The worker's shutdown shape (§7.2): stop flag, unpark,
/// join.
pub struct Sentry {
    stop: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl Sentry {
    /// Run [`SentryCtx::pass`] forever, parked for the clock's full-sweep
    /// period between ticks.
    pub fn spawn(ctx: SentryCtx) -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&stop);
        let handle = std::thread::spawn(move || {
            while !flag.load(Ordering::Relaxed) {
                ctx.pass();
                std::thread::park_timeout(ctx.period());
            }
        });
        Self {
            stop,
            handle: Some(handle),
        }
    }
}

impl Drop for Sentry {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            handle.thread().unpark();
            let _ = handle.join();
        }
    }
}

#[cfg(test)]
mod tests;
