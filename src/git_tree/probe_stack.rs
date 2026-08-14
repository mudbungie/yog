//! The platform liveness probe stack, held across ticks (DESIGN §10, §15 Y11).
//!
//! [`GitTree::from_repo`](super::GitTree::from_repo) built fresh probes on every
//! call, throwing away any cache. The live UI instead constructs **one**
//! [`ProbeStack`] and re-[`derive`](ProbeStack::derive)s each workspace through
//! it, so the macOS 2 s TTL cache ([`probe_cache`](super::probe_cache), Y22)
//! survives between derivations and a streaming `response.json` append storm
//! collapses to ≤1 `lsof` per target per TTL. Linux carries the two stateless
//! `/proc` probes (§10: always present, always definite), so holding them is
//! free and the cache is not compiled at all.
//!
//! [`ProbeStack::invalidate_liveness`] is the §10 eager cache eviction: it drops
//! each named agent's lock-probe target so the next derive **observes** the
//! driver rather than trusting a within-TTL answer. Two callers ask for it, on
//! the two signals that can carry the two transitions, and their conditions are
//! complementary so neither pays for the other's case:
//!
//! - the §7.2 cheap sweep, for a workspace that **does** hold a Live/InFlight
//!   agent — only those can die *silently*, since a released flock emits no fs
//!   event and no watcher will ever mention it;
//! - the watcher-driven re-derivation, for a workspace that holds **none** — a
//!   change under a quiet workspace may be a driver arriving, and nothing else
//!   would notice for a whole TTL.
//!
//! A streaming `response.json` storm falls under neither (its workspace is
//! already known live), which is what leaves the collapse above intact. On Linux
//! the whole thing is a no-op.

use super::{Agent, GitTree, GitTreeError, REPO_DIR, cmd, enumerate};
use std::path::Path;

/// The liveness probes for this platform, constructed once via
/// [`platform`](ProbeStack::platform) and shared across every tick.
pub struct ProbeStack {
    #[cfg(not(target_os = "macos"))]
    lock: super::lock_probe::ProcFsLockProbe,
    #[cfg(not(target_os = "macos"))]
    writer: super::fd_probe::ProcFsProbe,
    #[cfg(target_os = "macos")]
    probe: super::probe_cache::TtlCache<
        super::lsof::LsofProbe<super::lsof::SystemLsof>,
        crate::ui_state::SystemClock,
    >,
}

impl ProbeStack {
    /// The two `/proc` probes (Linux, §10 — cheap, definite, no cache).
    #[cfg(not(target_os = "macos"))]
    pub fn platform() -> Self {
        Self {
            lock: super::lock_probe::ProcFsLockProbe::default(),
            writer: super::fd_probe::ProcFsProbe::default(),
        }
    }

    /// The single `lsof` probe behind the 2 s TTL cache (macOS, §10).
    #[cfg(target_os = "macos")]
    pub fn platform() -> Self {
        Self {
            probe: super::lsof::system_probe(),
        }
    }

    /// Derive one workspace's [`GitTree`] through the held probes — the config
    /// lineage log plus the classified agent set (§3.5 stateless re-read).
    pub fn derive(&self, workspace: &Path) -> Result<GitTree, GitTreeError> {
        let git_dir = workspace.join(REPO_DIR);
        let log = cmd::git_log_first_parent(&git_dir)?;
        let commits = log.into_iter().map(enumerate::build_node).collect();
        let agents = self.enumerate(workspace, &git_dir)?;
        Ok(GitTree { commits, agents })
    }

    #[cfg(not(target_os = "macos"))]
    fn enumerate(&self, workspace: &Path, git_dir: &Path) -> Result<Vec<Agent>, GitTreeError> {
        enumerate::enumerate_agents(workspace, git_dir, &self.lock, &self.writer)
    }

    #[cfg(target_os = "macos")]
    fn enumerate(&self, workspace: &Path, git_dir: &Path) -> Result<Vec<Agent>, GitTreeError> {
        // One `lsof`-backed probe answers both liveness questions.
        enumerate::enumerate_agents(workspace, git_dir, &self.probe, &self.probe)
    }

    /// Evict the cached lock observation for each agent's inbox dir so its next
    /// read recomputes — the §7.2 eager refresh on the targeted liveness
    /// re-probe.
    #[cfg(target_os = "macos")]
    pub fn invalidate_liveness(&self, workspace: &Path, agent_ids: &[String]) {
        for id in agent_ids {
            self.probe
                .invalidate(&workspace.join(super::INBOX_DIR).join(id));
        }
    }

    /// A no-op on Linux (stateless `/proc` probes are always definite, §10).
    #[cfg(not(target_os = "macos"))]
    pub fn invalidate_liveness(&self, _workspace: &Path, _agent_ids: &[String]) {}
}
